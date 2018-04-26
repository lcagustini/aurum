extern crate sdl2;
extern crate unicode_segmentation;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use unicode_segmentation::UnicodeSegmentation;

use std::collections::HashMap;
use std::io::prelude::*;
use std::env;

macro_rules! rect(($x:expr, $y:expr, $w:expr, $h:expr) => (sdl2::rect::Rect::new($x as i32, $y as i32, $w as u32, $h as u32)));

const WINDOW_WIDTH: u32 = 1200;
const WINDOW_HEIGHT: u32 = 1000;
const BG_COLOR: Color = Color{r: 25, g: 25, b: 25, a: 255};
const BAR_COLOR: Color = Color{r: 255, g: 25, b: 25, a: 255};
const FONT_SIZE: u16 = 20;

struct Text<'ttf, 'a> {
    font: sdl2::ttf::Font<'ttf, 'a>,
    rendered: Option<Vec<sdl2::render::Texture<'a>>>,
    lines_rendered: Option<Vec<sdl2::render::Texture<'a>>>,
    raw: Vec<String>,
    character_cache: HashMap<String, sdl2::render::Texture<'a>>
}
impl<'ttf, 'a> Text<'ttf, 'a> {
    fn new(font: sdl2::ttf::Font<'ttf, 'a>, raw: Vec<String>) -> Text<'ttf, 'a> {
        Text { font: font, rendered: None, lines_rendered: None, raw: raw, character_cache: HashMap::new() }
    }

    fn get_char(&mut self, character: &str, texture_creator: &'a sdl2::render::TextureCreator<sdl2::video::WindowContext>) -> &sdl2::render::Texture {
        if !self.character_cache.contains_key(character) {
            self.font.set_style(sdl2::ttf::STYLE_NORMAL);

            let surface = self.font.render(character).blended(Color::RGBA(255, 255, 255, 255)).unwrap();
            let texture = texture_creator.create_texture_from_surface(&surface).unwrap();

            self.character_cache.insert(character.to_owned(), texture);
        }

        self.character_cache.get(character).unwrap()
    }

    fn render_text(&mut self, texture_creator: &'a sdl2::render::TextureCreator<sdl2::video::WindowContext>) {
        let mut rendered: Vec<sdl2::render::Texture> = Vec::new();
        let mut lines_rendered: Vec<sdl2::render::Texture> = Vec::new();

        for i in 0..self.raw.len() {
            let line_number = format!["{:1$} ", i+1, number_of_digits(self.raw.len())];
            let line = &self.raw[i as usize];

            {
                self.font.set_style(sdl2::ttf::STYLE_NORMAL);

                let surface = if line.len() == 0 {
                    self.font.render(" ").blended(Color::RGBA(255, 255, 255, 255)).unwrap()
                }
                else {
                    self.font.render(&line).blended(Color::RGBA(255, 255, 255, 255)).unwrap()
                };
                let texture = texture_creator.create_texture_from_surface(&surface).unwrap();

                rendered.push(texture);
            }{
                self.font.set_style(sdl2::ttf::STYLE_BOLD);

                let surface = self.font.render(&line_number).blended(Color::RGBA(255, 255, 255, 255)).unwrap();
                let texture = texture_creator.create_texture_from_surface(&surface).unwrap();

                lines_rendered.push(texture);
            }
        }

        self.rendered = Some(rendered);
        self.lines_rendered = Some(lines_rendered);
    }

    fn render_line(&mut self, line: usize, texture_creator: &'a sdl2::render::TextureCreator<sdl2::video::WindowContext>) {
        let line_str = &self.raw[line];

        self.font.set_style(sdl2::ttf::STYLE_NORMAL);

        let surface = if line_str.len() == 0 {
            self.font.render(" ").blended(Color::RGBA(255, 255, 255, 255)).unwrap()
        }
        else {
            self.font.render(&line_str).blended(Color::RGBA(255, 255, 255, 255)).unwrap()
        };
        let texture = texture_creator.create_texture_from_surface(&surface).unwrap();

        match self.rendered {
            Some(ref mut r) => r[line] = texture,
            None => return,
        };
    }
}

struct Cursor<'r> {
    x: u32,
    y: u32,
    wanted_x: u32,
    screen_y: u32,
    texture: sdl2::render::Texture<'r>,
}
impl<'r> Cursor<'r> {
    fn new(x: u32, y: u32, texture_creator: &'r sdl2::render::TextureCreator<sdl2::video::WindowContext>) -> Cursor<'r> {
        let mut cursor_surface = sdl2::surface::Surface::new((6*FONT_SIZE/10) as u32, FONT_SIZE as u32, sdl2::pixels::PixelFormatEnum::RGBA8888).unwrap();
        cursor_surface.fill_rect(rect![0, 0, 4, FONT_SIZE], sdl2::pixels::Color::RGBA(255, 255, 255, 128)).unwrap();

        Cursor{ x: x, y: y, wanted_x: x, screen_y: 0, texture: texture_creator.create_texture_from_surface(cursor_surface).unwrap() }
    }

    fn up(&mut self, text: &Vec<String>, canvas: &sdl2::render::Canvas<sdl2::video::Window>) {
        if self.y > 0 {
            self.y -= 1;
            self.x = if self.wanted_x > text[self.get_absolute_y()].len() as u32 {
                text[self.get_absolute_y()].len() as u32
            }
            else {
                self.wanted_x
            };
        }
        else if self.get_absolute_y() > 0 {
            self.scroll_screen(canvas, text, 1);
            self.up(text, canvas);
        }
    }

    fn down(&mut self, text: &Vec<String>, canvas: &sdl2::render::Canvas<sdl2::video::Window>) {
        if self.get_absolute_y() < text.len()-1 && self.y < (canvas.window().size().1/FONT_SIZE as u32)-2 {
            self.y += 1;
            self.x = if self.wanted_x > text[self.get_absolute_y()].len() as u32 {
                text[self.get_absolute_y()].len() as u32
            }
            else {
                self.wanted_x
            };
        }
        else if self.get_absolute_y() < text.len()-1 {
            self.scroll_screen(canvas, text, -1);
            self.down(text, canvas);
        }
    }

    fn left(&mut self, text: &Vec<String>) {
        if self.x > 0 {
            self.x -= 1;
            while !text[self.get_absolute_y()].is_char_boundary(self.x as usize) {
                self.x -= 1;
            }
            self.wanted_x = self.x;
        }
    }

    fn right(&mut self, text: &Vec<String>) {
        if self.x < text[self.get_absolute_y()].len() as u32 {
            self.x += 1;
            while !text[self.get_absolute_y()].is_char_boundary(self.x as usize) {
                self.x += 1;
            }
            self.wanted_x = self.x;
        }
    }

    fn get_absolute_y(&self) -> usize {
        (self.y + self.screen_y) as usize
    }

    fn scroll_screen(&mut self, canvas: &sdl2::render::Canvas<sdl2::video::Window>, text: &Vec<String>, dir: i32) {
        if (self.screen_y > 0 && dir > 0) || (self.screen_y < (text.len()-1) as u32 && dir < 0) {
            self.screen_y = if dir > 0 { self.screen_y - 1 } else { self.screen_y + 1 };
            if (self.y > 0 && dir < 0) || (self.y < (canvas.window().size().1/FONT_SIZE as u32)-2 && dir > 0) {
                if dir > 0 {
                    self.down(text, canvas);
                }
                else {
                    self.up(text, canvas);
                };
            }
            else {
                self.x = if self.wanted_x > text[self.get_absolute_y()].len() as u32 {
                    text[self.get_absolute_y()].len() as u32
                }
                else {
                    self.wanted_x
                };
            }
        }
    }
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let ttf_context = sdl2::ttf::init().unwrap();

    let args: Vec<String> = env::args().collect();

    let window = video_subsystem.window("Aurum", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .allow_highdpi()
        .resizable()
        .build()
        .unwrap();

    let mut font = ttf_context.load_font("roboto.ttf", FONT_SIZE).unwrap();
    font.set_style(sdl2::ttf::STYLE_NORMAL);

    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();

    let lines: Vec<String> = read_file(&args[1]).split("\n").map(|x| x.to_owned()).collect();

    canvas.set_draw_color(BG_COLOR);

    let mut text = Text::new(font, lines);
    text.render_text(&texture_creator);

    let mut cursor = Cursor::new(0, 0, &texture_creator);
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        let begin_loop_instant = std::time::Instant::now();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },

                Event::KeyDown { keycode: Some(Keycode::Left), .. } => {
                    cursor.left(&text.raw);
                },

                Event::KeyDown { keycode: Some(Keycode::Right), .. } => {
                    cursor.right(&text.raw);
                },

                Event::KeyDown { keycode: Some(Keycode::Up), .. } => {
                    cursor.up(&text.raw, &canvas);
                },

                Event::KeyDown { keycode: Some(Keycode::Down), .. } => {
                    cursor.down(&text.raw, &canvas);
                },

                Event::KeyDown { keycode: Some(Keycode::Return), .. } => {
                    text.raw[cursor.get_absolute_y()].insert_str(cursor.x as usize, "\n");

                    let halves: Vec<String> = text.raw[cursor.get_absolute_y()].split("\n").map(|x| x.to_owned()).collect();

                    text.raw[cursor.get_absolute_y()] = halves[0].clone();

                    text.raw.insert((cursor.get_absolute_y()+1) as usize, halves[1].clone());

                    cursor.x = 0;
                    cursor.wanted_x = 0;
                    cursor.y += 1;

                    //text.render_text(&texture_creator);
                },

                Event::KeyDown { keycode: Some(Keycode::Backspace), .. } => {
                    if cursor.x > 0 {
                        cursor.left(&text.raw);
                        text.raw[cursor.get_absolute_y()].remove(cursor.x as usize);

                        text.render_line(cursor.get_absolute_y(), &texture_creator);
                    }
                    else if cursor.x == 0 && cursor.get_absolute_y() > 0 {
                        let line = text.raw[cursor.get_absolute_y()].clone();

                        cursor.x = text.raw[cursor.get_absolute_y()-1].len() as u32;
                        cursor.wanted_x = cursor.x;

                        text.raw[cursor.get_absolute_y()-1].push_str(&line);

                        text.raw.remove(cursor.get_absolute_y());

                        if cursor.y == 0 {
                            cursor.screen_y -= 1;
                        }
                        else {
                            cursor.y -= 1;
                        }

                        //text.render_text(&texture_creator);
                    }
                },

                Event::KeyDown { keycode: Some(Keycode::F5), .. } => {
                    save_file(&args[1], &text.raw);
                },

                Event::TextInput { text: input, .. } => {
                    text.raw[cursor.get_absolute_y()].insert_str(cursor.x as usize, &input);
                    cursor.x += input.len() as u32;

                    //text.render_line(cursor.get_absolute_y(), &texture_creator);
                },

                Event::MouseWheel { y: dir, .. } => {
                    cursor.scroll_screen(&canvas, &text.raw, dir);
                },

                _ => {}
            }
        }

        canvas.clear();

        for i in (cursor.screen_y as usize)..text.raw.len() {
            let line = text.raw[i].clone();
            let mut iter = line.graphemes(true);
            let mut x = 0;
            let y = FONT_SIZE*((i-cursor.screen_y as usize) as u16);

            let mut c = iter.next();
            while c != None {
                let texture = text.get_char(c.unwrap(), &texture_creator);
                let texture_info = texture.query();

                canvas.copy(texture, None, Some(rect![x, y, texture_info.width, texture_info.height])).unwrap();
                x += texture_info.width;

                c = iter.next()
            }
        }

        /*
        match text.rendered {
            Some(ref rendered) => {
                match text.lines_rendered {
                    Some(ref lines_rendered) => {
                        for i in cursor.screen_y..(text.raw.len() as u32) {
                            let texture = &rendered[i as usize];
                            let line_texture = &lines_rendered[i as usize];

                            let texture_info = texture.query();
                            let line_texture_info = line_texture.query();

                            let y = FONT_SIZE*((i-cursor.screen_y) as u16);
                            canvas.copy(&line_texture, None, Some(rect![0, y, line_texture_info.width, line_texture_info.height])).unwrap();
                            canvas.copy(&texture, None, Some(rect![line_texture_info.width, y, texture_info.width, texture_info.height])).unwrap();
                            if y >= (canvas.window().size().1 as u16)-2*FONT_SIZE {
                                break
                            }
                        }

                        let line_number_info = &lines_rendered[cursor.get_absolute_y()].query();
                        let (half, _) = text.raw[cursor.get_absolute_y()].split_at(cursor.x as usize);
                        text.font.set_style(sdl2::ttf::STYLE_NORMAL);
                        let (x, _) = text.font.size_of(half).unwrap();
                        canvas.copy(&cursor.texture, None, Some(rect![x+line_number_info.width, cursor.y*(FONT_SIZE as u32), 4, FONT_SIZE])).unwrap();
                    },
                    _ => {},
                };
            },
            _ => {},
        };
        */

        canvas.set_draw_color(BAR_COLOR);
        let (w_width, w_height) = canvas.window().size();
        let _ = canvas.fill_rect(rect![0, w_height - FONT_SIZE as u32, w_width, FONT_SIZE]);
        canvas.set_draw_color(BG_COLOR);

        canvas.present();

        let frame_time = std::time::Instant::now().duration_since(begin_loop_instant);
        //println!["FPS: {}", 1_000_000_000/frame_time.subsec_nanos()];
    }
}

fn read_file(path: &str) -> String {
    let mut file = std::fs::File::open(path).unwrap();
    let mut s = String::new();
    let _ = file.read_to_string(&mut s);

    s
}

fn save_file(path: &str, buffer: &Vec<String>) {
    let mut file = std::fs::File::create(path).unwrap();
    let mut s = String::new();

    for l in buffer.iter() {
        s.push_str(l);
        s.push_str("\n");
    }

    let result = file.write(&s.into_bytes());
    match result {
        Ok(_) => {},
        Err(e) => println!["{}", e],
    }
}

fn number_of_digits(n: usize) -> usize {
    let mut i = 0;
    let mut n = n;
    while n != 0 {
        n /= 10;
        i += 1;
    }
    i
}
