extern crate serde;
extern crate serde_json;

use ::utils;
use ::syntax;

use sdl2::pixels::Color;

#[derive(Debug, Serialize, Deserialize)]
struct ConfigJSON {
    syntax_constant_color: [u8; 3],
    syntax_keyword_color: [u8; 3],
    syntax_secondary_word_color: [u8; 3],
    syntax_preproc_color: [u8; 3],
    syntax_data_type_color: [u8; 3],
    syntax_comment_color: [u8; 3],
    syntax_other_color: [u8; 3],

    bg_color: [u8; 3],
    line_number_color: [u8; 3],

    bar_color: [u8; 3],
    bar_text_color: [u8; 3],

    select_color: [u8; 3],
    search_color: [u8; 3],

    cursor_width: u32,

    font_path: String,
    font_size: u16,
}

#[derive(Debug)]
pub struct Config {
    pub syntax_color: syntax::SyntaxColor,

    pub bg_color: Color,
    pub line_number_color: Color,

    pub bar_color: Color,
    pub bar_text_color: Color,

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
                    syntax_color: syntax::SyntaxColor {
                        constant: color![decoded.syntax_constant_color],
                        keyword: color![decoded.syntax_keyword_color],
                        secondary_word: color![decoded.syntax_secondary_word_color],
                        preproc: color![decoded.syntax_preproc_color],
                        data_type: color![decoded.syntax_data_type_color],
                        comment: color![decoded.syntax_comment_color],
                        other: color![decoded.syntax_other_color],
                    },

                    bg_color: color![decoded.bg_color],
                    line_number_color: color![decoded.line_number_color],

                    bar_color: color![decoded.bar_color],
                    bar_text_color: color![decoded.bar_text_color],

                    select_color: color![decoded.select_color],
                    search_color: color![decoded.search_color],

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
            syntax_color: syntax::SyntaxColor {
                constant: color![[0,100,255]],
                keyword: color![[255,0,0]],
                secondary_word: color![[255,100,0]],
                preproc: color![[255,255,0]],
                data_type: color![[255,0,255]],
                comment: color![[150,150,150]],
                other: color![[255,255,255]],
            },

            bg_color: color![[25u8, 25, 25]],
            line_number_color: color![[255,255,255]],

            bar_color: color![[15u8, 15, 15]],
            bar_text_color: color![[255,255,255]],

            select_color: color![[255u8, 255, 255]],
            search_color: color![[255u8, 255, 30]],

            cursor_width: 8,

            font_path: "roboto.ttf".to_owned(),
            font_size: 18,
        }
    }
}
