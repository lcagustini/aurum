extern crate sdl2;
extern crate unicode_segmentation;

use sdl2::pixels::Color;
use sdl2::render::{Texture, TextureCreator};
use sdl2::video::WindowContext;

use ::config;

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
    pub fn new(font: sdl2::ttf::Font<'ttf, 'a>, raw: Vec<String>, config: &config::Config) -> Text<'ttf, 'a> {
        Text { font: font, font_size: config.font_size, raw: raw, file_path: "".to_owned(), normal_character_cache: HashMap::new(), bold_character_cache: HashMap::new(), needs_update: true }
    }

    pub fn get_bold_char(&mut self, character: &str, texture_creator: &'a TextureCreator<WindowContext>, color: &sdl2::pixels::Color) -> &Texture {
        if !self.bold_character_cache.contains_key(character) {
            self.font.set_style(sdl2::ttf::STYLE_BOLD);

            let surface = self.font.render(character).blended(Color::RGBA(255, 255, 255, 255)).unwrap();
            let texture = texture_creator.create_texture_from_surface(&surface).unwrap();

            self.bold_character_cache.insert(character.to_owned(), texture);
        }

        {
            let t = self.bold_character_cache.get_mut(character).unwrap();
            let (r, g, b) = color.rgb();
            t.set_color_mod(r, g, b);
        }

        self.bold_character_cache.get(character).unwrap()
    }

    pub fn get_normal_char(&mut self, character: &str, texture_creator: &'a TextureCreator<WindowContext>, color: &sdl2::pixels::Color) -> &Texture {
        if !self.normal_character_cache.contains_key(character) {
            self.font.set_style(sdl2::ttf::STYLE_NORMAL);

            let surface = self.font.render(character).blended(Color::RGBA(255, 255, 255, 255)).unwrap();
            let texture = texture_creator.create_texture_from_surface(&surface).unwrap();

            self.normal_character_cache.insert(character.to_owned(), texture);
        }

        {
            let t = self.normal_character_cache.get_mut(character).unwrap();
            let (r, g, b) = color.rgb();
            t.set_color_mod(r, g, b);
        }

        self.normal_character_cache.get(character).unwrap()
    }

    pub fn get_text_type(&self) -> String {
        let n_iter = self.file_path.graphemes(true).rev();
        let mut ext = "".to_owned();
        for n in n_iter {
            if n == "." {
                return ext;
            }
            else if n == "/" {
                break;
            }
            else {
                ext.insert_str(0, n);
            }
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
