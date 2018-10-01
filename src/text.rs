extern crate sdl2;
extern crate unicode_segmentation;

use sdl2::pixels::Color;
use sdl2::render::{Texture, TextureCreator};
use sdl2::video::WindowContext;

use unicode_segmentation::UnicodeSegmentation;

use std::collections::HashMap;

pub struct Text<'ttf, 'a> {
    pub font: sdl2::ttf::Font<'ttf, 'a>,
    pub font_size: u16,

    pub raw: Vec<String>,
    pub file_path: String,

    pub normal_character_cache: HashMap<String, Texture<'a>>,
    pub bold_character_cache: HashMap<String, Texture<'a>>,

    pub needs_update: bool,
}
impl<'ttf, 'a> Text<'ttf, 'a> {
    pub fn new(font: sdl2::ttf::Font<'ttf, 'a>, raw: Vec<String>, args: Vec<String>) -> Text<'ttf, 'a> {
        let file =
            if args.len() > 1 {
                args[1].clone()
            }
            else {
                "".to_owned()
            };

        Text { font: font, font_size: ::FONT_SIZE, raw: raw, file_path: file, normal_character_cache: HashMap::new(), bold_character_cache: HashMap::new(), needs_update: true }
    }

    pub fn get_bold_char(&mut self, character: &str, texture_creator: &'a TextureCreator<WindowContext>) -> &Texture {
        if !self.bold_character_cache.contains_key(character) {
            self.font.set_style(sdl2::ttf::STYLE_BOLD);

            let surface = self.font.render(character).blended(Color::RGBA(255, 255, 255, 255)).unwrap();
            let texture = texture_creator.create_texture_from_surface(&surface).unwrap();

            self.bold_character_cache.insert(character.to_owned(), texture);
        }

        self.bold_character_cache.get(character).unwrap()
    }

    pub fn get_normal_char(&mut self, character: &str, texture_creator: &'a TextureCreator<WindowContext>) -> &Texture {
        if !self.normal_character_cache.contains_key(character) {
            self.font.set_style(sdl2::ttf::STYLE_NORMAL);

            let surface = self.font.render(character).blended(Color::RGBA(255, 255, 255, 255)).unwrap();
            let texture = texture_creator.create_texture_from_surface(&surface).unwrap();

            self.normal_character_cache.insert(character.to_owned(), texture);
        }

        self.normal_character_cache.get(character).unwrap()
    }

    pub fn get_text_type(&self) -> String {
        let mut n_iter = self.file_path.graphemes(true).rev();
        let mut n = n_iter.next();
        let mut ext = "".to_owned();
        while n != None {
            if n.unwrap() == "." {
                return ext;
            }
            else if n.unwrap() == "/" {
                break;
            }
            else {
                ext.insert_str(0, n.unwrap());
            }
            n = n_iter.next();
        }
        return "?".to_owned();
    }

    pub fn get_text_dir(&self) -> String {
        let mut n_iter = self.file_path.graphemes(true).rev();
        let mut n = n_iter.next();
        while n != None {
            if n.unwrap() == "/" {
                return n_iter.rev().collect();
            }
            n = n_iter.next();
        }
        return "~".to_owned();
    }
}
