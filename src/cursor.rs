extern crate sdl2;
extern crate unicode_segmentation;

use sdl2::render::{TextureCreator, Canvas};
use sdl2::video::{Window, WindowContext};

use unicode_segmentation::UnicodeSegmentation;

use ::text;
use ::config;

pub struct Cursor<'r> {
    pub x: u32,
    pub y: u32,

    pub wanted_x: u32,
    pub number_w: u32,

    pub screen_x: u32,
    pub screen_y: u32,

    pub surface: sdl2::surface::Surface<'r>,
}
impl<'r> Cursor<'r> {
    pub fn new(x: u32, y: u32, config: &config::Config) -> Cursor<'r> {
        let mut cursor_surface = sdl2::surface::Surface::new((6*config.font_size/10) as u32, config.font_size as u32, sdl2::pixels::PixelFormatEnum::RGBA8888).unwrap();
        cursor_surface.fill_rect(rect![0, 0, config.cursor_width, config.font_size], sdl2::pixels::Color::RGBA(255, 255, 255, 128)).unwrap();

        Cursor{ x: x, y: y, wanted_x: x, number_w: 0, screen_x: 0, screen_y: 0, surface: cursor_surface }
    }

    pub fn up(&mut self, text: &Vec<String>, canvas: &Canvas<Window>, config: &config::Config) {
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
            self.scroll_screen(canvas, text, 1, config);
            self.up(text, canvas, config);
        }
    }

    pub fn down(&mut self, text: &Vec<String>, canvas: &Canvas<Window>, config: &config::Config) {
        if self.get_absolute_y() < text.len()-1 && self.y < (canvas.window().size().1/config.font_size as u32)-2 {
            self.y += 1;
            self.x = if self.wanted_x > text[self.get_absolute_y()].len() as u32 {
                text[self.get_absolute_y()].len() as u32
            }
            else {
                self.wanted_x
            };
        }
        else if self.get_absolute_y() < text.len()-1 {
            self.scroll_screen(canvas, text, -1, &config);
            self.down(text, canvas, config);
        }
    }

    pub fn left(&mut self, text: &Vec<String>) {
        if self.x > 0 {
            self.x -= 1;
            while !text[self.get_absolute_y()].is_char_boundary(self.x as usize) {
                self.x -= 1;
            }
            self.wanted_x = self.x;
        }
    }

    pub fn right(&mut self, text: &Vec<String>) {
        if self.x < text[self.get_absolute_y()].len() as u32 {
            self.x += 1;
            while !text[self.get_absolute_y()].is_char_boundary(self.x as usize) {
                self.x += 1;
            }
            self.wanted_x = self.x;
        }
    }

    pub fn get_absolute_y(&self) -> usize {
        (self.y + self.screen_y) as usize
    }

    pub fn scroll_screen(&mut self, canvas: &Canvas<Window>, text: &Vec<String>, dir: i32, config: &config::Config) {
        if (self.screen_y > 0 && dir > 0) || (self.screen_y < (text.len()-1) as u32 && dir < 0) {
            self.screen_y = if dir > 0 { self.screen_y - 1 } else { self.screen_y + 1 };
            if (self.y > 0 && dir < 0) || (self.y < (canvas.window().size().1/config.font_size as u32)-2 && dir > 0) {
                if dir > 0 {
                    self.down(text, canvas, config);
                }
                else {
                    self.up(text, canvas, config);
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

    pub fn move_to(&mut self, x: i32, y: i32, texture_creator: &'r TextureCreator<WindowContext>, text: &mut text::Text<'_, 'r>, config: &config::Config) {
        if (y/config.font_size as i32) as u32 + self.screen_y < text.raw.len() as u32 {
            self.y = (y/config.font_size as i32) as u32;
        }
        else {
            self.y = text.raw.len() as u32 - self.screen_y - 1;
        }

        let mut width = 0;
        let mut len = 0;
        let line = text.raw[self.get_absolute_y()].clone();
        let mut c_iter = line.graphemes(true);
        let mut c = c_iter.next();
        while c != None {
            if ((x-self.number_w as i32)-width as i32) < config.font_size as i32/2 {
                break;
            }

            let cur = c.unwrap();
            let texture = text.get_normal_char(&cur, &texture_creator, &::WHITE);
            let texture_info = texture.query();

            width += texture_info.width;
            len += cur.len() as u32;
            c = c_iter.next();
        }
        self.x = len;
        self.wanted_x = len;

        text.needs_update = true;
    }
}
