use std::io::prelude::*;
use std::fs::File;
use std::env;

use ::text;
use ::editor;

pub fn read_file(path: &str) -> String {
    let file = File::open(path);
    match file {
        Ok(mut file) => {
            let mut s = String::new();
            let _ = file.read_to_string(&mut s);

            return s;
        },
        Err(_) => {
            return "?".to_owned();
        },
    }
}

pub fn save_file(path: &str, buffer: &Vec<String>) {
    let mut file = File::create(path).unwrap();
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

pub fn number_of_digits(n: usize) -> usize {
    let mut i = 0;
    let mut n = n;
    while n != 0 {
        n /= 10;
        i += 1;
    }
    i
}

pub fn get_lang_name(text: &text::Text) -> String {
    let text_type = text.get_text_type();
    if text_type == "?" {
        return text_type;
    }
    else {
        let path = format!["{}/langs/{}/name", env::current_dir().unwrap().display(), text_type];
        return read_file(&path).trim().to_owned();
    }
}

pub fn update_timer(editor: &mut editor::Editor) {
    editor.undo_handler.create_state(&editor.cursor, &editor.text);

    for line in &editor.text.raw {
        editor.completion_engine.update_cache(line);
    }

    editor.char_timer = 0;
}
