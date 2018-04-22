extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use std::io::prelude::*;
use std::env;

macro_rules! rect(($x:expr, $y:expr, $w:expr, $h:expr) => (sdl2::rect::Rect::new($x as i32, $y as i32, $w as u32, $h as u32)));

const WINDOW_WIDTH: u32 = 1200;
const WINDOW_HEIGHT: u32 = 1000;
const BG_COLOR: Color = Color{r: 25, g: 25, b: 25, a: 255};
const FONT_SIZE: u16 = 20;

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

    fn up(&mut self, text: &Vec<String>) {
        if self.y > 0 {
            self.y -= 1;
            self.x = if self.wanted_x > text[self.get_absolute_cursor()].len() as u32 {
                text[self.get_absolute_cursor()].len() as u32
            }
            else {
                self.wanted_x
            };
        }
    }

    fn down(&mut self, text: &Vec<String>) {
        if self.y < (text.len() as u32)-1 && self.y < (WINDOW_HEIGHT/FONT_SIZE as u32)-1 {
            self.y += 1;
            self.x = if self.wanted_x > text[self.get_absolute_cursor()].len() as u32 {
                text[self.get_absolute_cursor()].len() as u32
            }
            else {
                self.wanted_x
            };
        }
    }

    fn left(&mut self, text: &Vec<String>) {
        if self.x > 0 {
            self.x -= 1;
            while !text[self.get_absolute_cursor()].is_char_boundary(self.x as usize) {
                self.x -= 1;
            }
            self.wanted_x = self.x;
        }
    }

    fn right(&mut self, text: &Vec<String>) {
        if self.x < text[self.get_absolute_cursor()].len() as u32 {
            self.x += 1;
            while !text[self.get_absolute_cursor()].is_char_boundary(self.x as usize) {
                self.x += 1;
            }
            self.wanted_x = self.x;
        }
    }

    fn get_absolute_cursor(&self) -> usize {
        (self.y + self.screen_y) as usize
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
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();

    let mut font = ttf_context.load_font("roboto.ttf", FONT_SIZE).unwrap();
    font.set_style(sdl2::ttf::STYLE_NORMAL);

    canvas.set_draw_color(BG_COLOR);

    let mut cursor = Cursor::new(0, 0, &texture_creator);
    let mut lines: Vec<String> = read_file(&args[1]).split("\n").map(|x| x.to_owned()).collect();
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        let begin_loop_instant = std::time::Instant::now();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown { keycode: Some(Keycode::Left), .. } => {
                    cursor.left(&lines);
                },
                Event::KeyDown { keycode: Some(Keycode::Right), .. } => {
                    cursor.right(&lines);
                },
                Event::KeyDown { keycode: Some(Keycode::Up), .. } => {
                    cursor.up(&lines);
                },
                Event::KeyDown { keycode: Some(Keycode::Down), .. } => {
                    cursor.down(&lines);
                },
                Event::KeyDown { keycode: Some(Keycode::Return), .. } => {
                    lines[cursor.get_absolute_cursor()].insert_str(cursor.x as usize, "\n");

                    let halves: Vec<String> = lines[cursor.get_absolute_cursor()].split("\n").map(|x| x.to_owned()).collect();

                    lines[cursor.get_absolute_cursor()] = halves[0].clone();

                    lines.insert((cursor.get_absolute_cursor()+1) as usize, halves[1].clone());

                    cursor.x = 0;
                    cursor.wanted_x = 0;
                    cursor.y += 1;
                },
                Event::KeyDown { keycode: Some(Keycode::Backspace), .. } => {
                    if cursor.x > 0 {
                        cursor.left(&lines);
                        lines[cursor.get_absolute_cursor()].remove(cursor.x as usize);
                    }
                    else if cursor.x == 0 && cursor.get_absolute_cursor() > 0 {
                        let line = lines[cursor.get_absolute_cursor()].clone();

                        cursor.x = lines[cursor.get_absolute_cursor()-1].len() as u32;
                        cursor.wanted_x = cursor.x;

                        lines[cursor.get_absolute_cursor()-1].push_str(&line);

                        lines.remove(cursor.get_absolute_cursor());

                        if cursor.y == 0 {
                            cursor.screen_y -= 1;
                        }
                        else {
                            cursor.y -= 1;
                        }
                    }
                },
                Event::KeyDown { keycode: Some(Keycode::F5), .. } => {
                    save_file(&args[1], &lines);
                },
                Event::TextInput { text: input, .. } => {
                    lines[cursor.get_absolute_cursor()].insert_str(cursor.x as usize, &input);
                    cursor.x += input.len() as u32;
                },
                Event::MouseWheel { y: dir, .. } => {
                    if (cursor.screen_y > 0 && dir > 0) || (cursor.screen_y < (lines.len()-1) as u32 && dir < 0) {
                        cursor.screen_y = if dir > 0 { cursor.screen_y - 1 } else { cursor.screen_y + 1 };
                        if (cursor.y > 0 && dir < 0) || (cursor.y < (WINDOW_HEIGHT/FONT_SIZE as u32)-1 && dir > 0) {
                            if dir > 0 {
                                cursor.down(&lines);
                            }
                            else {
                                cursor.up(&lines);
                            };
                        }
                        else {
                            cursor.x = if cursor.wanted_x > lines[cursor.get_absolute_cursor()].len() as u32 {
                                lines[cursor.get_absolute_cursor()].len() as u32
                            }
                            else {
                                cursor.wanted_x
                            };
                        }
                    }
                },
                _ => {}
            }
        }

        canvas.clear();
        for i in cursor.screen_y..(lines.len() as u32) {
            let with_line_number = format!["{:n$} {}", i, lines[i as usize], n = number_of_digits(lines.len())];
            let surface = match font.render(&with_line_number).blended(Color::RGBA(255, 255, 255, 255)) {
                Ok(s) => s,
                Err(_) => continue,
            };
            let texture = texture_creator.create_texture_from_surface(&surface).unwrap();
            let sdl2::render::TextureQuery { width, height, .. } = texture.query();

            let y = FONT_SIZE*((i-cursor.screen_y) as u16);
            canvas.copy(&texture, None, Some(rect![0, y, width, height])).unwrap();
            if y >= (WINDOW_HEIGHT as u16) {
                break
            }
        }
        let line_number = format!["{:n$} ", cursor.get_absolute_cursor(), n = number_of_digits(lines.len())];
        let (line_number_x, _) = font.size_of(&line_number).unwrap();
        let (half, _) = lines[cursor.get_absolute_cursor()].split_at(cursor.x as usize);
        let (x, _) = font.size_of(half).unwrap();
        canvas.copy(&cursor.texture, None, Some(rect![x+line_number_x, cursor.y*(FONT_SIZE as u32), 4, FONT_SIZE])).unwrap();

        canvas.present();

        let frame_time = std::time::Instant::now().duration_since(begin_loop_instant);
        println!["FPS: {}", 1_000_000_000/frame_time.subsec_nanos()];
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
    while(n != 0) {
        n /= 10;
        i += 1;
    }
    i
}
