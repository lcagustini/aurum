extern crate sdl2;
extern crate unicode_segmentation;
extern crate nfd;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use unicode_segmentation::UnicodeSegmentation;

use std::env;

macro_rules! rect(($x:expr, $y:expr, $w:expr, $h:expr) => (sdl2::rect::Rect::new($x as i32, $y as i32, $w as u32, $h as u32)));

const BG_COLOR: Color = Color{r: 25, g: 25, b: 25, a: 255};
const BAR_COLOR: Color = Color{r: 15, g: 15, b: 15, a: 255};
const SELECT_COLOR: Color = Color{r: 255, g: 255, b: 255, a: 255};
const SEARCH_COLOR: Color = Color{r: 255, g: 255, b: 30, a: 255};

const FONT_SIZE: u16 = 20;

mod utils;
mod text;
mod cursor;
mod select;
mod undo;

struct SearchHandler {
    active: bool,
    search_string: String,

    cur_index: usize,
    found_places: Vec<(u32, u32)>
}
impl SearchHandler {
    fn new() -> SearchHandler {
        SearchHandler{active: false, search_string: "".to_owned(), cur_index: 0, found_places: Vec::new()}
    }

    fn find_search_string(&mut self, text: &Vec<String>) {
        self.found_places.clear();
        self.cur_index = 0;
        for (y, line) in text.iter().enumerate() {
            let r = line.find(&self.search_string);
            match r {
                Some(x) => self.found_places.push((x as u32, y as u32)),
                _ => ()
            }
        }
    }

    fn next_string_pos(&mut self) -> Option<(u32, u32)> {
        if self.found_places.len() > 0 {
            let ret = self.found_places[self.cur_index];
            self.cur_index += 1;
            if self.cur_index == self.found_places.len() {
                self.cur_index = 0;
            }
            return Some(ret);
        }
        return None
    }
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let ttf_context = sdl2::ttf::init().unwrap();

    let args: Vec<String> = env::args().collect();

    let window = video_subsystem.window("Aurum", 1200, 1000)
        .position_centered()
        .allow_highdpi()
        .resizable()
        .build()
        .unwrap();

    let mut font = ttf_context.load_font("roboto.ttf", FONT_SIZE).unwrap();
    font.set_style(sdl2::ttf::STYLE_NORMAL);

    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();

    let lines: Vec<String> =
        if args.len() > 1 {
            utils::read_file(&args[1]).split("\n").map(|x| x.to_owned()).collect()
        }
        else {
            vec!["".to_owned()]
        };

    let mut text = text::Text::new(font, lines, args);
    let mut cursor = cursor::Cursor::new(0, 0, &texture_creator);
    let mut selected = select::SelectHandler{x1: 0, y1: 0, x2: 0, y2: 0};
    let mut undo_handler = undo::UndoHandler::new(&cursor, &text);
    let mut search_handler = SearchHandler::new();

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => {
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

                Event::KeyDown { keycode: Some(Keycode::O), keymod, .. } => {
                    if keymod.contains(sdl2::keyboard::LCTRLMOD) {
                        let dir = text.get_text_dir();
                        let result = nfd::open_file_dialog(None, Some(&dir)).unwrap();
                        match result {
                            nfd::Response::Okay(file_path) => {
                                text.raw = utils::read_file(&file_path).split("\n").map(|x| x.to_owned()).collect();
                                text.file_path = file_path;

                                cursor.x = 0;
                                cursor.y = 0;

                                undo_handler.clear_states();
                                undo_handler.create_state(&cursor, &text);

                                text.needs_update = true;
                            },

                            _ => ()
                        }
                    }
                },

                Event::KeyDown { keycode: Some(Keycode::S), keymod, .. } => {
                    if keymod.contains(sdl2::keyboard::LCTRLMOD) {
                        if text.file_path != "" {
                            utils::save_file(&text.file_path, &text.raw);
                        }
                        else {
                            let result = nfd::open_save_dialog(None, None).unwrap();
                            match result {
                                nfd::Response::Okay(file_path) => {
                                    utils::save_file(&file_path, &text.raw);
                                    text.file_path = file_path;

                                    text.needs_update = true;
                                },

                                _ => ()
                            }
                        }
                    }
                },

                Event::KeyDown { keycode: Some(Keycode::Z), keymod, .. } => {
                    let mut ctrl_shift = sdl2::keyboard::LCTRLMOD;
                    ctrl_shift.insert(sdl2::keyboard::LSHIFTMOD);

                    if keymod.contains(ctrl_shift) {
                        undo_handler.restore_next_state(&mut cursor, &mut text);
                        text.needs_update = true;
                    }
                    else if keymod.contains(sdl2::keyboard::LCTRLMOD) {
                        undo_handler.restore_previous_state(&mut cursor, &mut text);
                        text.needs_update = true;
                    }
                },

                Event::KeyDown { keycode: Some(Keycode::F), keymod, .. } => {
                    if keymod.contains(sdl2::keyboard::LCTRLMOD) {
                        search_handler.active = !search_handler.active;
                        search_handler.search_string.clear();

                        text.needs_update = true;
                    }
                },

                Event::KeyDown { keycode: Some(Keycode::C), keymod, .. } => {
                    if keymod.contains(sdl2::keyboard::LCTRLMOD) {
                        if !search_handler.active {
                            let text = selected.get_selected_text(&text);
                            video_subsystem.clipboard().set_clipboard_text(&text).unwrap();
                        }
                    }
                },

                Event::KeyDown { keycode: Some(Keycode::V), keymod, .. } => {
                    if keymod.contains(sdl2::keyboard::LCTRLMOD) {
                        if !search_handler.active {
                            let input: Vec<String> = video_subsystem.clipboard().clipboard_text().unwrap().split("\n").map(|x| x.to_owned()).collect();

                            for line in input {
                                text.raw[cursor.get_absolute_y()].insert_str(cursor.x as usize, &line);
                                text.raw.insert(cursor.get_absolute_y()+1, "".to_owned());
                                cursor.x += line.len() as u32;
                                cursor.down(&text.raw, &canvas);
                            }
                            undo_handler.create_state(&cursor, &text);
                            text.needs_update = true;
                        }
                    }
                },

                Event::KeyDown { keycode: Some(Keycode::Return), .. } => {
                    if search_handler.active {
                        match search_handler.next_string_pos() {
                            Some((x, y)) => {
                                while cursor.get_absolute_y() > y as usize {
                                    cursor.up(&text.raw, &canvas);
                                }

                                while cursor.get_absolute_y() < y as usize {
                                    cursor.down(&text.raw, &canvas);
                                }

                                while cursor.x > x {
                                    cursor.left(&text.raw);
                                }

                                while cursor.x < x {
                                    cursor.right(&text.raw);
                                }
                            },
                            None => (),
                        }
                    }
                    else {
                        text.raw[cursor.get_absolute_y()].insert_str(cursor.x as usize, "\n");

                        let halves: Vec<String> = text.raw[cursor.get_absolute_y()].split("\n").map(|x| x.to_owned()).collect();

                        text.raw[cursor.get_absolute_y()] = halves[0].clone();

                        text.raw.insert((cursor.get_absolute_y()+1) as usize, halves[1].clone());

                        cursor.x = 0;
                        cursor.wanted_x = 0;
                        cursor.y += 1;

                        undo_handler.create_state(&cursor, &text);
                    }
                    text.needs_update = true;
                },

                Event::KeyDown { keycode: Some(Keycode::Backspace), .. } => {
                    if search_handler.active {
                        search_handler.search_string.pop();
                    }
                    else {
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
                    }
                    text.needs_update = true;
                },

                Event::KeyDown { keycode: Some(Keycode::Tab), .. } => {
                    if !search_handler.active {
                        let input = "    ";
                        text.raw[cursor.get_absolute_y()].insert_str(cursor.x as usize, input);
                        cursor.x += input.len() as u32;
                        text.needs_update = true;
                    }
                },

                Event::TextInput { text: input, .. } => {
                    if search_handler.active {
                        search_handler.search_string.push_str(&input);
                        search_handler.find_search_string(&text.raw);
                    }
                    else {
                        text.raw[cursor.get_absolute_y()].insert_str(cursor.x as usize, &input);
                        cursor.x += input.len() as u32;
                    }
                    text.needs_update = true;
                },

                Event::MouseWheel { y: dir, .. } => {
                    cursor.scroll_screen(&canvas, &text.raw, dir);
                    text.needs_update = true;
                },

                Event::MouseButtonDown { mouse_btn: button, x, y, .. } => {
                    match button {
                        sdl2::mouse::MouseButton::Left => {
                            let n_w = cursor.number_w;
                            cursor.move_to(x, y, n_w, &texture_creator, &mut text);
                        },
                        _ => {},
                    }
                },

                Event::MouseButtonUp { mouse_btn: button, x, y, .. } => {
                    match button {
                        sdl2::mouse::MouseButton::Left => {
                            let (old_x, old_y) = (cursor.x, cursor.get_absolute_y());

                            {
                                let n_w = cursor.number_w;
                                cursor.move_to(x, y, n_w, &texture_creator, &mut text);
                            }

                            if old_x < cursor.x {
                                selected.x1 = old_x as usize;
                                selected.x2 = cursor.x as usize;
                            }
                            else {
                                selected.x2 = old_x as usize;
                                selected.x1 = cursor.x as usize;
                            }

                            if old_y < cursor.get_absolute_y() {
                                selected.y1 = old_y as usize;
                                selected.y2 = cursor.get_absolute_y() as usize;
                            }
                            else {
                                selected.y1 = cursor.get_absolute_y() as usize;
                                selected.y2 = old_y as usize;
                            }

                            text.needs_update = true;
                        },
                        _ => {},
                    }
                },

                Event::Window { win_event: event, .. } => {
                    match event {
                        sdl2::event::WindowEvent::SizeChanged(_, _) | sdl2::event::WindowEvent::FocusGained |
                            sdl2::event::WindowEvent::Exposed | sdl2::event::WindowEvent::Moved(_, _)=> text.needs_update = true,
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
        canvas.set_draw_color(BG_COLOR);
        canvas.clear();

        let (w_width, w_height) = canvas.window().size();

        //Draw Lines
        {
            let screen_limit = if text.raw.len() < (cursor.screen_y + canvas.window().size().1/text.font_size as u32) as usize {
                text.raw.len()
            }
            else {
                (cursor.screen_y + canvas.window().size().1/text.font_size as u32) as usize
            };
            for i in (cursor.screen_y as usize)..screen_limit {
                let mut x = 0;
                let y = text.font_size*((i-cursor.screen_y as usize) as u16);

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

                cursor.number_w = x;

                let line = text.raw[i].clone();
                let mut c_iter = line.graphemes(true);
                let mut c = c_iter.next();
                for _ in 0..cursor.screen_x {
                    c = c_iter.next();
                }
                while c != None {
                    let texture = text.get_normal_char(c.unwrap(), &texture_creator);
                    let texture_info = texture.query();

                    canvas.copy(texture, None, Some(rect![x, y, texture_info.width, texture_info.height])).unwrap();
                    x += texture_info.width;

                    c = c_iter.next()
                }
            }
        }

        //Draw text selection
        // TODO: transparent selection box
        {
            if selected.x1 != selected.x2 || selected.y1 != selected.y2 {
                let (half, _) = text.raw[selected.y1].split_at(selected.x1);
                let (x1, _) = text.font.size_of(half).unwrap();

                let (half, _) =
                    if selected.x2 < text.raw[selected.y2].len() {
                        text.raw[selected.y2].split_at(selected.x2)
                    }
                    else {
                        (&text.raw[selected.y2][..], "")
                    };
                let (x2, _) = text.font.size_of(half).unwrap();

                canvas.set_draw_color(SELECT_COLOR);
                if selected.y1 == selected.y2 {
                    canvas.draw_rect(rect![x1+cursor.number_w, selected.y1*text.font_size as usize, x2-x1, text.font_size]).unwrap();
                }
                else {
                    for i in selected.y1..=selected.y2 {
                        let mut start = cursor.number_w;
                        let mut end = cursor.number_w;
                        let (all, _) = text.font.size_of(&text.raw[i][..]).unwrap();

                        if i == selected.y1 {
                            start += x1;
                            end += all;
                        }
                        else if i == selected.y2 {
                            end += x2;
                        }
                        else {
                            end += all;
                        }
                        canvas.draw_rect(rect![start, i*text.font_size as usize, end-start, text.font_size]).unwrap();
                    }
                }
            }
        }

        //Draw search highlight
        {
            if search_handler.active {
                for (x, y) in &search_handler.found_places {
                    if y.clone() >= cursor.screen_y {
                        let (half, _) = text.raw[y.clone() as usize].split_at(x.clone() as usize);
                        let (x1, _) = text.font.size_of(half).unwrap();
                        let (w, _) = text.font.size_of(&search_handler.search_string).unwrap();

                        canvas.set_draw_color(SEARCH_COLOR);
                        canvas.draw_rect(rect![x1+cursor.number_w, (y-cursor.screen_y)*text.font_size as u32, w, text.font_size]).unwrap();
                    }
                }
            }
        }

        //Draw cursor
        {
            text.font.set_style(sdl2::ttf::STYLE_NORMAL);
            let (half, _) = text.raw[cursor.get_absolute_y()].split_at(cursor.x as usize);
            let (x, _) = text.font.size_of(half).unwrap();
            canvas.copy(&cursor.texture, None, Some(rect![x+cursor.number_w, cursor.y*(text.font_size as u32), 4, text.font_size])).unwrap();
        }

        //Draw statusbar
        {
            canvas.set_draw_color(BAR_COLOR);
            canvas.fill_rect(rect![0, w_height - text.font_size as u32, w_width, text.font_size]).unwrap();

            //Right aligned
            {
                let lines_ui = format!["{}/{}: {}", cursor.get_absolute_y()+1, text.raw.len(), cursor.x+1];
                let mut n_iter = lines_ui.graphemes(true);
                let mut n = n_iter.next();
                let mut x = w_width-text.font.size_of(&lines_ui).unwrap().0-10;
                while n != None {
                    let f_s = text.font_size;

                    let texture = text.get_normal_char(n.unwrap(), &texture_creator);
                    let texture_info = texture.query();

                    canvas.copy(&texture, None, Some(rect![x, w_height-f_s as u32, texture_info.width, texture_info.height])).unwrap();

                    n = n_iter.next();

                    x += texture_info.width;
                }
            }
            //Left aligned
            {
                let lines_ui =
                    if search_handler.active {
                        format!["Search: {}", &search_handler.search_string]
                    }
                    else { 
                        format!["{}: {}", &text.get_text_type(), &text.file_path]
                    };

                let mut n_iter = lines_ui.graphemes(true);
                let mut n = n_iter.next();
                let mut x = 10;
                while n != None {
                    let f_s = text.font_size;

                    let texture = text.get_normal_char(n.unwrap(), &texture_creator);
                    let texture_info = texture.query();

                    canvas.copy(texture, None, Some(rect![x, w_height-f_s as u32, texture_info.width, texture_info.height])).unwrap();

                    n = n_iter.next();

                    x += texture_info.width;
                }
            }
        }

        canvas.present();

        text.needs_update = false;
    }
}
