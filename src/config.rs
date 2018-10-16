extern crate serde;
extern crate serde_json;

use ::utils;

use sdl2::pixels::Color;

#[derive(Debug, Serialize, Deserialize)]
struct ConfigJSON {
    bg_color: [u8; 3],
    bar_color: [u8; 3],
    select_color: [u8; 3],
    search_color: [u8; 3],

    cursor_width: u32,

    font_path: String,
    font_size: u16,
}

#[derive(Debug)]
pub struct Config {
    pub bg_color: Color,
    pub bar_color: Color,
    pub select_color: Color,
    pub search_color: Color,

    pub cursor_width: u32,

    pub font_path: String,
    pub font_size: u16,
}
impl Config {
    pub fn load_config(path: &str) -> Config {
        let file = utils::read_file(path);

        let decoded: Result<ConfigJSON, serde_json::Error> = serde_json::from_str(&file);

        match decoded {
            Ok(decoded) => {
                return Config {
                    bg_color: color![decoded.bg_color[0], decoded.bg_color[1], decoded.bg_color[2]],
                    bar_color: color![decoded.bar_color[0], decoded.bar_color[1], decoded.bar_color[2]],
                    select_color: color![decoded.select_color[0], decoded.select_color[1], decoded.select_color[2]],
                    search_color: color![decoded.search_color[0], decoded.search_color[1], decoded.search_color[2]],

                    cursor_width: decoded.cursor_width as u32,

                    font_path: decoded.font_path,
                    font_size: decoded.font_size as u16,
                };
            },
            Err(e) => {
                println!["{}", e];
                return Default::default();
            }
        }
    }
}
impl Default for Config {
    fn default() -> Config {
        Config {
            bg_color: color![25, 25, 25],
            bar_color: color![15, 15, 15],
            select_color: color![255, 255, 255],
            search_color: color![255, 255, 30],

            cursor_width: 8,

            font_path: "roboto.ttf".to_owned(),
            font_size: 18,
        }
    }
}
