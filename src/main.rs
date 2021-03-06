extern crate sdl2;
extern crate unicode_segmentation;
extern crate nfd;
#[macro_use] extern crate serde_derive;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use unicode_segmentation::UnicodeSegmentation;

use std::env;

macro_rules! rect(($x:expr, $y:expr, $w:expr, $h:expr) => (sdl2::rect::Rect::new($x as i32, $y as i32, $w as u32, $h as u32)));
macro_rules! color(($a:expr) => (Color::RGB($a[0] as u8, $a[1] as u8, $a[2] as u8)));
macro_rules! color_a(($a:expr, $alpha:expr) => (Color::RGBA($a[0] as u8, $a[1] as u8, $a[2] as u8, $alpha)));

const WHITE: sdl2::pixels::Color = sdl2::pixels::Color {r: 255, g: 255, b: 255, a: 255};

mod utils;
mod text;
mod cursor;
mod select;
mod undo;
mod search;
mod editor;
mod syntax;
mod autocomplete;
mod config;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let ttf_context = sdl2::ttf::init().unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();
    let window = video_subsystem.window("Aurum", 1200, 1000)
        .position_centered()
        .allow_highdpi()
        .resizable()
        .build()
        .unwrap();

    let canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();

    let config = config::Config::load_config("config.json");

    let mut editor = editor::Editor::create(canvas, &ttf_context, &config);

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => {
                    break 'running
                },

                Event::KeyDown { keycode: Some(Keycode::Left), .. } => {
                    editor.cursor.left(&editor.text.raw);
                    editor.completion_engine.list_mode = false;
                    editor.text.needs_update = true;
                },

                Event::KeyDown { keycode: Some(Keycode::Right), .. } => {
                    editor.cursor.right(&editor.text.raw);
                    editor.completion_engine.list_mode = false;
                    editor.text.needs_update = true;
                },

                Event::KeyDown { keycode: Some(Keycode::Up), .. } => {
                    editor.cursor.up(&editor.text.raw, &editor.canvas, &config);
                    editor.completion_engine.list_mode = false;
                    editor.text.needs_update = true;
                },

                Event::KeyDown { keycode: Some(Keycode::Down), .. } => {
                    editor.cursor.down(&editor.text.raw, &editor.canvas, &config);
                    editor.completion_engine.list_mode = false;
                    editor.text.needs_update = true;
                },

                Event::KeyDown { keycode: Some(Keycode::N), keymod, .. } => {
                    if keymod.contains(sdl2::keyboard::Mod::LCTRLMOD) {
                        editor = editor::Editor::create(editor.canvas, &ttf_context, &config);
                    }
                },

                Event::KeyDown { keycode: Some(Keycode::O), keymod, .. } => {
                    if keymod.contains(sdl2::keyboard::Mod::LCTRLMOD) {
                        let dir = editor.text.get_text_dir();
                        let result = nfd::open_file_dialog(None, Some(&dir)).unwrap();
                        match result {
                            nfd::Response::Okay(file_path) => {
                                editor.text.raw = utils::read_file(&file_path).split("\n").map(|x| x.to_owned()).collect();
                                editor.text.file_path = file_path;

                                editor.cursor.x = 0;
                                editor.cursor.wanted_x = 0;
                                editor.cursor.screen_y = 0;
                                editor.cursor.y = 0;

                                editor.undo_handler.clear_states();
                                utils::update_timer(&mut editor);

                                let text_type = editor.text.get_text_type();
                                if text_type != "?" {
                                    let path = format!["{}/langs/{}/syntax.json", env::current_dir().unwrap().display(), text_type];
                                    editor.syntax_handler = syntax::SyntaxHandler::parse_syntax_file(&path);
                                }
                                else {
                                    editor.syntax_handler = None;
                                }

                                editor.text.needs_update = true;
                            },

                            _ => ()
                        }
                    }
                },

                Event::KeyDown { keycode: Some(Keycode::S), keymod, .. } => {
                    if keymod.contains(sdl2::keyboard::Mod::LCTRLMOD) {
                        if editor.text.file_path != "" {
                            utils::save_file(&editor.text.file_path, &editor.text.raw);
                        }
                        else {
                            let result = nfd::open_save_dialog(None, None).unwrap();
                            match result {
                                nfd::Response::Okay(file_path) => {
                                    utils::save_file(&file_path, &editor.text.raw);
                                    editor.text.file_path = file_path;

                                    let text_type = editor.text.get_text_type();
                                    if text_type != "?" {
                                        let path = format!["{}/langs/{}/syntax.json", env::current_dir().unwrap().display(), text_type];
                                        editor.syntax_handler = syntax::SyntaxHandler::parse_syntax_file(&path);
                                    }
                                    else {
                                        editor.syntax_handler = None;
                                    }

                                    editor.text.needs_update = true;
                                },

                                _ => ()
                            }
                        }
                    }
                },

                Event::KeyDown { keycode: Some(Keycode::Z), keymod, .. } => {
                    let mut ctrl_shift = sdl2::keyboard::Mod::LCTRLMOD;
                    ctrl_shift.insert(sdl2::keyboard::Mod::LSHIFTMOD);

                    if keymod.contains(ctrl_shift) {
                        editor.undo_handler.restore_next_state(&mut editor.cursor, &mut editor.text);
                        editor.text.needs_update = true;
                    }
                    else if keymod.contains(sdl2::keyboard::Mod::LCTRLMOD) {
                        editor.undo_handler.restore_previous_state(&mut editor.cursor, &mut editor.text);
                        editor.text.needs_update = true;
                    }
                },


                Event::KeyDown { keycode: Some(Keycode::P), keymod, .. } => {
                    if keymod.contains(sdl2::keyboard::Mod::LCTRLMOD) {
                        if editor.completion_engine.list_mode {
                            editor.completion_engine.selected_word += 1;
                            if editor.completion_engine.selected_word >= editor.completion_engine.completion_list.len() {
                                editor.completion_engine.selected_word = 0;
                            }
                        }
                        else {
                            editor.completion_engine.complete(&editor.text.raw, &editor.cursor);
                            if editor.completion_engine.completion_list.len() == 1 {
                                let complete = &editor.completion_engine.completion_list[0][editor.completion_engine.cur_word.len()..];

                                editor.text.raw[editor.cursor.get_absolute_y()].insert_str(editor.cursor.x as usize, &complete);
                                editor.cursor.x += complete.len() as u32;
                            }
                            else {
                                editor.completion_engine.selected_word = 0;
                                editor.completion_engine.list_mode = true;
                            }
                        }

                        editor.text.needs_update = true;
                    }
                },

                Event::KeyDown { keycode: Some(Keycode::F), keymod, .. } => {
                    if keymod.contains(sdl2::keyboard::Mod::LCTRLMOD) {
                        editor.search_handler.active = !editor.search_handler.active;
                        editor.search_handler.search_string.clear();

                        editor.selected.reset_selection();

                        editor.text.needs_update = true;
                    }
                },

                Event::KeyDown { keycode: Some(Keycode::C), keymod, .. } => {
                    if keymod.contains(sdl2::keyboard::Mod::LCTRLMOD) {
                        if !editor.search_handler.active {
                            let text = editor.selected.get_selected_text(&editor.text);
                            video_subsystem.clipboard().set_clipboard_text(&text).unwrap();
                        }
                    }
                },

                Event::KeyDown { keycode: Some(Keycode::V), keymod, .. } => {
                    if keymod.contains(sdl2::keyboard::Mod::LCTRLMOD) {
                        if !editor.search_handler.active {
                            let input: Vec<String> = video_subsystem.clipboard().clipboard_text().unwrap().split("\n").map(|x| x.to_owned()).collect();
                            let len = input.len();

                            for (i, line) in input.iter().enumerate() {
                                editor.text.raw[editor.cursor.get_absolute_y()].insert_str(editor.cursor.x as usize, &line);

                                if i < len-1 {
                                    editor.text.raw.insert(editor.cursor.get_absolute_y()+1, "".to_owned());
                                    editor.cursor.x += line.len() as u32;
                                    editor.cursor.down(&editor.text.raw, &editor.canvas, &config);
                                }
                            }
                            utils::update_timer(&mut editor);
                            editor.text.needs_update = true;
                        }
                    }
                },

                Event::KeyDown { keycode: Some(Keycode::Return), .. } => {
                    if editor.search_handler.active {
                        match editor.search_handler.next_string_pos() {
                            Some((x, y)) => {
                                while editor.cursor.get_absolute_y() > y as usize {
                                    editor.cursor.up(&editor.text.raw, &editor.canvas, &config);
                                }

                                while editor.cursor.get_absolute_y() < y as usize {
                                    editor.cursor.down(&editor.text.raw, &editor.canvas, &config);
                                }

                                while editor.cursor.x > x {
                                    editor.cursor.left(&editor.text.raw);
                                }

                                while editor.cursor.x < x {
                                    editor.cursor.right(&editor.text.raw);
                                }
                            },
                            None => (),
                        }
                    }
                    else {
                        if editor.completion_engine.list_mode {
                            let complete = &editor.completion_engine.completion_list[editor.completion_engine.selected_word][editor.completion_engine.cur_word.len()..];

                            editor.text.raw[editor.cursor.get_absolute_y()].insert_str(editor.cursor.x as usize, &complete);
                            editor.cursor.x += complete.len() as u32;

                            editor.completion_engine.list_mode = false;
                        }
                        else {
                            editor.text.raw[editor.cursor.get_absolute_y()].insert_str(editor.cursor.x as usize, "\n");

                            let halves: Vec<String> = editor.text.raw[editor.cursor.get_absolute_y()].split("\n").map(|x| x.to_owned()).collect();

                            editor.text.raw[editor.cursor.get_absolute_y()] = halves[0].clone();

                            let mut space_amount = halves[0].len() - halves[0].trim_start().len();
                            let mut space_string = "".to_owned();

                            if editor.text.raw[editor.cursor.get_absolute_y()].trim().ends_with("{") ||
                                editor.text.raw[editor.cursor.get_absolute_y()].trim().ends_with(":") ||
                                editor.text.raw[editor.cursor.get_absolute_y()].trim().ends_with("(") {
                                space_amount += 4;
                            }

                            for _ in 0..space_amount {
                                space_string.push(' ');
                            };

                            space_string.push_str(&halves[1]);
                            editor.text.raw.insert((editor.cursor.get_absolute_y()+1) as usize, space_string);

                            editor.cursor.x = space_amount as u32;
                            editor.cursor.wanted_x = 0;
                            editor.cursor.y += 1;
                        }

                        utils::update_timer(&mut editor);
                        editor.selected.reset_selection();
                    }
                    editor.text.needs_update = true;
                },

                Event::KeyDown { keycode: Some(Keycode::Backspace), .. } => {
                    if editor.search_handler.active {
                        editor.search_handler.search_string.pop();
                        editor.search_handler.find_search_string(&editor.text.raw);
                    }
                    else {
                        // FIXME: This crashes sometimes
                        if editor.selected.y1 != editor.selected.y2 || editor.selected.x1 != editor.selected.x2 {
                            let mut new_line = editor.text.raw[editor.selected.y1][..editor.selected.x1].to_owned();
                            new_line.push_str(&editor.text.raw[editor.selected.y2][editor.selected.x2..]);

                            if editor.selected.y1 == editor.selected.y2 {
                                editor.text.raw[editor.selected.y1] = new_line;
                            }
                            else {
                                for i in (editor.selected.y1..=editor.selected.y2).rev() {
                                    editor.text.raw.remove(i);
                                }
                                editor.text.raw.insert(editor.selected.y1, new_line);
                            }

                            editor.cursor.y = editor.selected.y1 as u32 - editor.cursor.screen_y;
                            editor.cursor.x = editor.selected.x1 as u32;
                            editor.cursor.wanted_x = editor.selected.x1 as u32;

                            utils::update_timer(&mut editor);
                        }
                        else {
                            if editor.cursor.x > 0 {
                                let amount =
                                    if editor.text.raw[editor.cursor.get_absolute_y()][..editor.cursor.x as usize].ends_with("    ") {
                                        4
                                    }
                                    else {
                                        1
                                    };

                                for _ in 0..amount {
                                    editor.cursor.left(&editor.text.raw);
                                    editor.text.raw[editor.cursor.get_absolute_y()].remove(editor.cursor.x as usize);
                                }
                                if editor.completion_engine.list_mode {
                                    editor.completion_engine.complete(&editor.text.raw, &editor.cursor);
                                }
                            }
                            else if editor.cursor.x == 0 && editor.cursor.get_absolute_y() > 0 {
                                let line = editor.text.raw[editor.cursor.get_absolute_y()].clone();

                                editor.cursor.x = editor.text.raw[editor.cursor.get_absolute_y()-1].len() as u32;
                                editor.cursor.wanted_x = editor.cursor.x;

                                editor.text.raw[editor.cursor.get_absolute_y()-1].push_str(&line);

                                editor.text.raw.remove(editor.cursor.get_absolute_y());

                                if editor.cursor.y == 0 {
                                    editor.cursor.screen_y -= 1;
                                }
                                else {
                                    editor.cursor.y -= 1;
                                }

                                editor.completion_engine.list_mode = false;
                            }
                        }

                        editor.selected.reset_selection();
                    }
                    editor.text.needs_update = true;
                },

                Event::KeyDown { keycode: Some(Keycode::Tab), .. } => {
                    if !editor.search_handler.active {
                        let input = "    ";
                        editor.text.raw[editor.cursor.get_absolute_y()].insert_str(editor.cursor.x as usize, input);
                        editor.cursor.x += input.len() as u32;

                        editor.completion_engine.list_mode = false;

                        editor.text.needs_update = true;
                    }
                },

                Event::TextInput { text: input, .. } => {
                    if editor.search_handler.active {
                        editor.search_handler.search_string.push_str(&input);
                        editor.search_handler.find_search_string(&editor.text.raw);
                    }
                    else {
                        editor.text.raw[editor.cursor.get_absolute_y()].insert_str(editor.cursor.x as usize, &input);
                        editor.cursor.x += input.len() as u32;

                        if editor.completion_engine.list_mode {
                            editor.completion_engine.complete(&editor.text.raw, &editor.cursor);
                            if editor.completion_engine.completion_list.len() == 0 {
                                editor.completion_engine.list_mode = false;
                            }
                        }

                        editor.selected.reset_selection();

                        editor.char_timer += 1;
                    }
                    editor.text.needs_update = true;
                },

                Event::MouseWheel { y: dir, .. } => {
                    editor.cursor.scroll_screen(&editor.canvas, &editor.text.raw, dir, &config);
                    editor.text.needs_update = true;
                },

                Event::MouseButtonDown { mouse_btn: button, x, y, .. } => {
                    match button {
                        sdl2::mouse::MouseButton::Left => {
                            editor.cursor.move_to(x, y, &texture_creator, &mut editor.text, &config);

                            editor.selected.old_x = editor.cursor.x as usize;
                            editor.selected.old_y = editor.cursor.get_absolute_y();

                            editor.selected.y1 = editor.cursor.get_absolute_y() as usize;
                            editor.selected.x1 = editor.cursor.x as usize;

                            editor.selected.y2 = editor.cursor.get_absolute_y() as usize;
                            editor.selected.x2 = editor.cursor.x as usize;
                        },
                        _ => {},
                    }
                },

                Event::MouseButtonUp { mouse_btn: button, x, y, .. } => {
                    match button {
                        sdl2::mouse::MouseButton::Left => {
                            editor.cursor.move_to(x, y, &texture_creator, &mut editor.text, &config);

                            if editor.selected.old_y < editor.cursor.get_absolute_y() || (editor.selected.old_y == editor.cursor.get_absolute_y() && editor.selected.old_x < editor.cursor.x as usize) {
                                editor.selected.y1 = editor.selected.old_y as usize;
                                editor.selected.x1 = editor.selected.old_x as usize;

                                editor.selected.y2 = editor.cursor.get_absolute_y() as usize;
                                editor.selected.x2 = editor.cursor.x as usize;
                            }
                            else {
                                editor.selected.y1 = editor.cursor.get_absolute_y() as usize;
                                editor.selected.x1 = editor.cursor.x as usize;

                                editor.selected.y2 = editor.selected.old_y as usize;
                                editor.selected.x2 = editor.selected.old_x as usize;
                            }

                            editor.text.needs_update = true;
                        },
                        _ => {},
                    }
                },

                Event::MouseMotion { mousestate, x, y, .. } => {
                    if mousestate.left() {
                        {
                            let (_, w_height) = editor.canvas.window().size();
                            if y > (w_height - 3*config.font_size as u32) as i32 {
                                editor.cursor.scroll_screen(&editor.canvas, &editor.text.raw, -1, &config);
                            }
                            if y < 3*config.font_size as i32 {
                                editor.cursor.scroll_screen(&editor.canvas, &editor.text.raw, 1, &config);
                            }
                        }

                        {
                            let new_y =
                                if (y/config.font_size as i32) as u32 + editor.cursor.screen_y < editor.text.raw.len() as u32 {
                                    (y/config.font_size as i32) as usize + editor.cursor.screen_y as usize
                                }
                                else {
                                    editor.text.raw.len() - 1
                                };
                            let new_x = {
                                let mut width = 0;
                                let mut len = 0;
                                let line = editor.text.raw[new_y].clone();
                                let mut c_iter = line.graphemes(true);
                                let mut c = c_iter.next();
                                while c != None {
                                    if ((x-editor.cursor.number_w as i32)-width as i32) < config.font_size as i32/2 {
                                        break;
                                    }

                                    let cur = c.unwrap();
                                    let texture = editor.text.get_normal_char(&cur, &texture_creator, &::WHITE);
                                    let texture_info = texture.query();

                                    width += texture_info.width;
                                    len += cur.len() as u32;
                                    c = c_iter.next();
                                }
                                len
                            };

                            if editor.selected.old_y < new_y || (editor.selected.old_y == new_y && editor.selected.old_x < new_x as usize) {
                                editor.selected.y1 = editor.selected.old_y as usize;
                                editor.selected.x1 = editor.selected.old_x as usize;

                                editor.selected.y2 = new_y;
                                editor.selected.x2 = new_x as usize;
                            }
                            else {
                                editor.selected.y1 = new_y;
                                editor.selected.x1 = new_x as usize;

                                editor.selected.y2 = editor.selected.old_y as usize;
                                editor.selected.x2 = editor.selected.old_x as usize;
                            }
                        }

                        editor.text.needs_update = true;
                    }
                },

                Event::Window { win_event: event, .. } => {
                    match event {
                        sdl2::event::WindowEvent::SizeChanged(_, _) | sdl2::event::WindowEvent::FocusGained |
                            sdl2::event::WindowEvent::Exposed | sdl2::event::WindowEvent::Moved(_, _)=> editor.text.needs_update = true,
                        _ => {},
                    }
                },

                _ => {}
            }
        }

        if !editor.text.needs_update {
            if editor.char_timer > 60 {
                utils::update_timer(&mut editor);
            }

            let display_index = editor.canvas.window().display_index().unwrap();
            let display_mode = video_subsystem.current_display_mode(display_index).unwrap();
            let frame_time = 1.0/display_mode.refresh_rate as f64;
            let duration = std::time::Duration::new(frame_time as u64, ((frame_time - (frame_time as usize) as f64)*1_000_000_000.0) as u32);
            std::thread::sleep(duration);

            continue;
        }
        editor.canvas.set_draw_color(config.bg_color);
        editor.canvas.clear();

        let (w_width, w_height) = editor.canvas.window().size();

        //Draw Lines
        {
            let screen_limit =
                if editor.text.raw.len() < (editor.cursor.screen_y + editor.canvas.window().size().1/editor.text.font_size as u32) as usize {
                    editor.text.raw.len()
                }
                else {
                    (editor.cursor.screen_y + editor.canvas.window().size().1/editor.text.font_size as u32) as usize
                };

            let digits = utils::number_of_digits(editor.text.raw.len());
            {
                let max_number = format!["{:1$} ", editor.text.raw.len(), digits];

                editor.text.font.set_style(sdl2::ttf::FontStyle::BOLD);
                let (x, _) = editor.text.font.size_of(&max_number).unwrap();
                editor.cursor.number_w = x;
                editor.text.font.set_style(sdl2::ttf::FontStyle::NORMAL);

                editor.canvas.set_draw_color(config.bar_color);
                editor.canvas.fill_rect(rect![0, 0, x, w_height]).unwrap();
            }

            for i in (editor.cursor.screen_y as usize)..screen_limit {
                //Draw line number
                let mut x = 0;
                let y = editor.text.font_size*((i-editor.cursor.screen_y as usize) as u16);

                let number = format!["{:1$} ", i+1, digits];
                let mut n_iter = number.graphemes(true);
                let mut n = n_iter.next();
                while n != None {
                    let texture = editor.text.get_bold_char(n.unwrap(), &texture_creator, &config.line_number_color);
                    let texture_info = texture.query();

                    editor.canvas.copy(texture, None, Some(rect![x, y, texture_info.width, texture_info.height])).unwrap();
                    x += texture_info.width;

                    n = n_iter.next()
                }

                //Draw line text
                let line = editor.text.raw[i].clone();

                let mut colors = syntax::SyntaxHandler::get_line_color(&line, &editor, &config).into_iter();

                let mut c_iter = line.graphemes(true);
                let mut c = c_iter.next();
                while c != None {
                    let texture = editor.text.get_normal_char(c.unwrap(), &texture_creator, &colors.next().unwrap());
                    let texture_info = texture.query();

                    editor.canvas.copy(texture, None, Some(rect![x, y, texture_info.width, texture_info.height])).unwrap();
                    x += texture_info.width;

                    c = c_iter.next()
                }
            }
        }

        //Draw text selection
        {
            editor.canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
            if editor.selected.x1 != editor.selected.x2 || editor.selected.y1 != editor.selected.y2 {
                let (half, _) = editor.text.raw[editor.selected.y1].split_at(editor.selected.x1);
                let (x1, _) = editor.text.font.size_of(half).unwrap();

                let (half, _) =
                    if editor.selected.x2 < editor.text.raw[editor.selected.y2].len() {
                        editor.text.raw[editor.selected.y2].split_at(editor.selected.x2)
                    }
                    else {
                        (&editor.text.raw[editor.selected.y2][..], "")
                    };
                let (x2, _) = editor.text.font.size_of(half).unwrap();

                editor.canvas.set_draw_color(config.select_color);
                if editor.selected.y1 == editor.selected.y2 {
                    editor.canvas.fill_rect(rect![x1+editor.cursor.number_w, (editor.selected.y1 as isize - editor.cursor.screen_y as isize)*editor.text.font_size as isize, x2-x1, editor.text.font_size]).unwrap();
                }
                else {
                    for i in editor.selected.y1..=editor.selected.y2 {
                        let mut start = editor.cursor.number_w;
                        let mut end = editor.cursor.number_w;
                        let (all, _) = editor.text.font.size_of(&editor.text.raw[i][..]).unwrap();

                        if i == editor.selected.y1 {
                            start += x1;
                            end += all;
                        }
                        else if i == editor.selected.y2 {
                            end += x2;
                        }
                        else {
                            end += all;
                        }
                        editor.canvas.fill_rect(rect![start, (i as isize - editor.cursor.screen_y as isize)*editor.text.font_size as isize, end-start, editor.text.font_size]).unwrap();
                    }
                }
            }
            editor.canvas.set_blend_mode(sdl2::render::BlendMode::None);
        }

        //Draw search highlight
        {
            editor.canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
            if editor.search_handler.active {
                for (x, y) in &editor.search_handler.found_places {
                    if y.clone() >= editor.cursor.screen_y {
                        let (half, _) = editor.text.raw[y.clone() as usize].split_at(x.clone() as usize);
                        let (x1, _) = editor.text.font.size_of(half).unwrap();
                        let (w, _) = editor.text.font.size_of(&editor.search_handler.search_string).unwrap();

                        editor.canvas.set_draw_color(config.search_color);
                        editor.canvas.fill_rect(rect![x1+editor.cursor.number_w, (y-editor.cursor.screen_y)*editor.text.font_size as u32, w, editor.text.font_size]).unwrap();
                    }
                }
            }
            editor.canvas.set_blend_mode(sdl2::render::BlendMode::None);
        }

        //Draw autocomplete options
        {
            if editor.completion_engine.list_mode {
                let x;
                {
                    let (half, _) = editor.text.raw[editor.cursor.get_absolute_y()].split_at(editor.cursor.x as usize);
                    let (temp_x, _) = editor.text.font.size_of(half).unwrap();

                    x = temp_x + editor.cursor.number_w;
                }
                let mut y = editor.cursor.y*(editor.text.font_size as u32);

                editor.canvas.set_draw_color(config.bar_color);
                editor.canvas.fill_rect(rect![x, y, 200, editor.completion_engine.completion_list.len()*(editor.text.font_size as usize)]).unwrap();

                for word in &editor.completion_engine.completion_list {
                    let mut c_x = x;
                    let mut iter = word.graphemes(true);
                    for c in iter {
                        let texture = editor.text.get_normal_char(c, &texture_creator, &WHITE);
                        let texture_info = texture.query();

                        editor.canvas.copy(texture, None, Some(rect![c_x, y, texture_info.width, texture_info.height])).unwrap();
                        c_x += texture_info.width;
                    }
                    y += editor.text.font_size as u32;
                }
            }
        }

        //Draw cursor
        {
            editor.text.font.set_style(sdl2::ttf::FontStyle::NORMAL);
            let (half, _) = editor.text.raw[editor.cursor.get_absolute_y()].split_at(editor.cursor.x as usize);
            let (x, _) = editor.text.font.size_of(half).unwrap();

            let texture = texture_creator.create_texture_from_surface(&editor.cursor.surface).unwrap();

            editor.canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
            editor.canvas.set_draw_color(sdl2::pixels::Color::RGBA(255, 255, 255, 15));
            editor.canvas.fill_rect(rect!(editor.cursor.number_w, editor.cursor.y * editor.text.font_size as u32, w_width - editor.cursor.number_w, editor.text.font_size)).unwrap();
            editor.canvas.set_blend_mode(sdl2::render::BlendMode::None);

            if editor.completion_engine.list_mode {
                editor.canvas.copy(&texture, None, Some(rect![1+x+editor.cursor.number_w, (editor.cursor.y + editor.completion_engine.selected_word as u32)*(editor.text.font_size as u32), config.cursor_width, editor.text.font_size])).unwrap();
            }
            else {
                editor.canvas.copy(&texture, None, Some(rect![1+x+editor.cursor.number_w, editor.cursor.y*(editor.text.font_size as u32), config.cursor_width, editor.text.font_size])).unwrap();
            }
        }

        //Draw statusbar
        {
            editor.canvas.set_draw_color(config.bar_color);
            editor.canvas.fill_rect(rect![0, w_height - editor.text.font_size as u32, w_width, editor.text.font_size]).unwrap();

            //Right aligned
            {
                let lines_ui = format!["{}/{}: {}", editor.cursor.get_absolute_y()+1, editor.text.raw.len(), editor.cursor.x+1];
                let mut n_iter = lines_ui.graphemes(true);
                let mut n = n_iter.next();
                let mut x = w_width-editor.text.font.size_of(&lines_ui).unwrap().0-10;
                while n != None {
                    let f_s = editor.text.font_size;

                    let texture = editor.text.get_normal_char(n.unwrap(), &texture_creator, &WHITE);
                    let texture_info = texture.query();

                    editor.canvas.copy(&texture, None, Some(rect![x, w_height-f_s as u32, texture_info.width, texture_info.height])).unwrap();

                    n = n_iter.next();

                    x += texture_info.width;
                }
            }
            //Left aligned
            {
                let lines_ui =
                    if editor.search_handler.active {
                        let index =
                            if editor.search_handler.cur_index == 0 {
                                editor.search_handler.found_places.len()
                            }
                            else {
                                editor.search_handler.cur_index
                            };
                        format!["Search: {} [{}/{}]", &editor.search_handler.search_string, index, editor.search_handler.found_places.len()]
                    }
                    else {
                        format!["{}: {}", &utils::get_lang_name(&editor.text), &editor.text.file_path]
                    };

                let mut n_iter = lines_ui.graphemes(true);
                let mut n = n_iter.next();
                let mut x = 10;
                while n != None {
                    let f_s = editor.text.font_size;

                    let texture = editor.text.get_normal_char(n.unwrap(), &texture_creator, &WHITE);
                    let texture_info = texture.query();

                    editor.canvas.copy(texture, None, Some(rect![x, w_height-f_s as u32, texture_info.width, texture_info.height])).unwrap();

                    n = n_iter.next();

                    x += texture_info.width;
                }
            }
        }

        editor.canvas.present();

        editor.text.needs_update = false;
    }
}
