extern crate sdl2;
extern crate unicode_segmentation;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use unicode_segmentation::UnicodeSegmentation;

use std::env;

macro_rules! rect(($x:expr, $y:expr, $w:expr, $h:expr) => (sdl2::rect::Rect::new($x as i32, $y as i32, $w as u32, $h as u32)));

const WINDOW_WIDTH: u32 = 1200;
const WINDOW_HEIGHT: u32 = 1000;

const BG_COLOR: Color = Color{r: 25, g: 25, b: 25, a: 255};
const BAR_COLOR: Color = Color{r: 15, g: 15, b: 15, a: 255};
const SELECT_COLOR: Color = Color{r: 180, g: 180, b: 180, a: 100};

const FONT_SIZE: u16 = 20;

mod utils;
mod text;
mod cursor;

struct Select {
    x1: usize,
    y1: usize,
    x2: usize,
    y2: usize,
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

    let lines: Vec<String> = utils::read_file(&args[1]).split("\n").map(|x| x.to_owned()).collect();

    canvas.set_draw_color(BG_COLOR);

    let mut text = text::Text::new(font, lines);
    let mut number_w: u32 = 0;

    let mut cursor = cursor::Cursor::new(0, 0, &texture_creator);
    let mut selected = Select{x1: 0, y1: 0, x2: 0, y2: 0};

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },

                Event::KeyDown { keycode: Some(Keycode::Left), .. } => {
                    cursor.left(&text.raw);
                    text.needs_update = true;
                },

                Event::KeyDown { keycode: Some(Keycode::Right), .. } => {
                    cursor.right(&text.raw);
                    text.needs_update = true;
                },

                Event::KeyDown { keycode: Some(Keycode::Up), .. } => {
                    cursor.up(&text.raw, &canvas);
                    text.needs_update = true;
                },

                Event::KeyDown { keycode: Some(Keycode::Down), .. } => {
                    cursor.down(&text.raw, &canvas);
                    text.needs_update = true;
                },

                Event::KeyDown { keycode: Some(Keycode::Return), .. } => {
                    text.raw[cursor.get_absolute_y()].insert_str(cursor.x as usize, "\n");

                    let halves: Vec<String> = text.raw[cursor.get_absolute_y()].split("\n").map(|x| x.to_owned()).collect();

                    text.raw[cursor.get_absolute_y()] = halves[0].clone();

                    text.raw.insert((cursor.get_absolute_y()+1) as usize, halves[1].clone());

                    cursor.x = 0;
                    cursor.wanted_x = 0;
                    cursor.y += 1;
                    text.needs_update = true;
                },

                Event::KeyDown { keycode: Some(Keycode::Backspace), .. } => {
                    if cursor.x > 0 {
                        cursor.left(&text.raw);
                        text.raw[cursor.get_absolute_y()].remove(cursor.x as usize);
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
                    }
                    text.needs_update = true;
                },

                Event::KeyDown { keycode: Some(Keycode::F5), .. } => {
                    utils::save_file(&args[1], &text.raw);
                },

                Event::TextInput { text: input, .. } => {
                    text.raw[cursor.get_absolute_y()].insert_str(cursor.x as usize, &input);
                    cursor.x += input.len() as u32;
                    text.needs_update = true;
                },

                Event::MouseWheel { y: dir, .. } => {
                    cursor.scroll_screen(&canvas, &text.raw, dir);
                    text.needs_update = true;
                },

                Event::MouseButtonDown { mouse_btn: button, x, y, .. } => {
                    match button {
                        sdl2::mouse::MouseButton::Left => {
                            cursor.move_to(x, y, number_w, &texture_creator, &mut text);
                        },
                        _ => {},
                    }
                },

                Event::MouseButtonUp { mouse_btn: button, x, y, .. } => {
                    match button {
                        sdl2::mouse::MouseButton::Left => {
                            let (old_x, old_y) = (cursor.x, cursor.get_absolute_y());

                            cursor.move_to(x, y, number_w, &texture_creator, &mut text);

                            if old_x < cursor.x {
                                selected.x1 = old_x as usize;
                                selected.y1 = old_y as usize;

                                selected.x2 = cursor.x as usize;
                                selected.y2 = cursor.get_absolute_y() as usize;
                                println!["{}", &text.raw[cursor.get_absolute_y()][old_x as usize..cursor.x as usize]];
                            }
                            else {
                                selected.x2 = old_x as usize;
                                selected.y2 = old_y as usize;

                                selected.x1 = cursor.x as usize;
                                selected.y1 = cursor.get_absolute_y() as usize;
                                println!["{}", &text.raw[cursor.get_absolute_y()][cursor.x as usize..old_x as usize]];
                            }

                            text.needs_update = true;
                        },
                        _ => {},
                    }
                },

                Event::Window { win_event: event, .. } => {
                    match event {
                        sdl2::event::WindowEvent::SizeChanged(_, _) => text.needs_update = true,
                        _ => {},
                    }
                },

                _ => {}
            }
        }

        if !text.needs_update {
            let display_index = canvas.window().display_index().unwrap();
            let display_mode = video_subsystem.current_display_mode(display_index).unwrap();
            let frame_time = 1.0/display_mode.refresh_rate as f64;
            let duration = std::time::Duration::new(frame_time as u64, ((frame_time - (frame_time as usize) as f64)*1_000_000_000.0) as u32);
            std::thread::sleep(duration);

            continue;
        }
        canvas.clear();

        let screen_limit = if text.raw.len() < (cursor.screen_y + canvas.window().size().1/FONT_SIZE as u32) as usize {
            text.raw.len()
        }
        else {
            (cursor.screen_y + canvas.window().size().1/FONT_SIZE as u32) as usize
        };

        for i in (cursor.screen_y as usize)..screen_limit {
            let mut x = 0;
            let y = FONT_SIZE*((i-cursor.screen_y as usize) as u16);

            let number = format!["{:1$} ", i+1, utils::number_of_digits(text.raw.len())];
            let mut n_iter = number.graphemes(true);
            let mut n = n_iter.next();
            while n != None {
                let texture = text.get_bold_char(n.unwrap(), &texture_creator);
                let texture_info = texture.query();

                canvas.copy(texture, None, Some(rect![x, y, texture_info.width, texture_info.height])).unwrap();
                x += texture_info.width;

                n = n_iter.next()
            }

            number_w = x;

            let line = text.raw[i].clone();
            let mut c_iter = line.graphemes(true);
            let mut c = c_iter.next();
            while c != None {
                let texture = text.get_normal_char(c.unwrap(), &texture_creator);
                let texture_info = texture.query();

                canvas.copy(texture, None, Some(rect![x, y, texture_info.width, texture_info.height])).unwrap();
                x += texture_info.width;

                c = c_iter.next()
            }
        }

        let (w_width, w_height) = canvas.window().size();
        {
            text.font.set_style(sdl2::ttf::STYLE_NORMAL);
            let (half, _) = text.raw[cursor.get_absolute_y()].split_at(cursor.x as usize);
            let (x, _) = text.font.size_of(half).unwrap();
            canvas.copy(&cursor.texture, None, Some(rect![x+number_w-2, cursor.y*(FONT_SIZE as u32), 4, FONT_SIZE])).unwrap();
        }

        {
            canvas.set_draw_color(BAR_COLOR);
            let _ = canvas.fill_rect(rect![0, w_height - FONT_SIZE as u32, w_width, FONT_SIZE]);
            canvas.set_draw_color(BG_COLOR);
        }

        {
            let lines_ui = format!["{}/{}", cursor.get_absolute_y()+1, text.raw.len()];
            let mut n_iter = lines_ui.graphemes(true);
            let mut n = n_iter.next();
            let mut x = w_width-text.font.size_of(&lines_ui).unwrap().0-10;
            while n != None {
                let texture = text.get_normal_char(n.unwrap(), &texture_creator);
                let texture_info = texture.query();

                canvas.copy(texture, None, Some(rect![x, w_height-2-FONT_SIZE as u32, texture_info.width, texture_info.height])).unwrap();

                n = n_iter.next();

                x += texture_info.width;
            }
        }

        {
            canvas.set_draw_color(SELECT_COLOR);
            let (half, _) = text.raw[selected.y1].split_at(selected.x1);
            let (x1, _) = text.font.size_of(half).unwrap();

            let (half, _) = text.raw[selected.y2].split_at(selected.x2);
            let (x2, _) = text.font.size_of(half).unwrap();

            let _ = canvas.fill_rect(rect![x1+number_w-2, selected.y1*FONT_SIZE as usize, x2-x1, (selected.y2-selected.y1+1)*FONT_SIZE as usize]);
            canvas.set_draw_color(BG_COLOR);
        }

        canvas.present();

        text.needs_update = false;
    }
}
