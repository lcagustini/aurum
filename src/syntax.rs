extern crate serde;
extern crate serde_json;
extern crate regex;

use ::utils;
use ::editor;

use sdl2::pixels::Color;

use unicode_segmentation::UnicodeSegmentation;

macro_rules! color(($r:expr, $g:expr, $b:expr) => (Color::RGB($r as u8, $g as u8, $b as u8)));

#[derive(Debug, Serialize, Deserialize)]
struct SyntaxJSON {
    c_constant: u32,
    c_keyword: u32,
    c_preproc: u32,
    c_data_type: u32,
    c_comment: u32,
    c_other: u32,

    s_constant: String,
    s_keyword: String,
    s_preproc: String,
    s_data_type: String,
    s_comment: String,
}

#[derive(Debug)]
pub struct SyntaxColor {
    pub constant: Color,
    pub keyword: Color,
    pub preproc: Color,
    pub data_type: Color,
    pub comment: Color,
    pub other: Color,
}

#[derive(Debug)]
pub struct SyntaxStructs {
    pub constant: regex::Regex,
    pub keyword: regex::Regex,
    pub preproc: regex::Regex,
    pub data_type: regex::Regex,
    pub comment: regex::Regex,
}

#[derive(Debug)]
pub struct SyntaxHandler {
    pub structs: SyntaxStructs,
    pub colors: SyntaxColor,
}
impl SyntaxHandler {
    pub fn parse_syntax_file(path: &str) -> Option<SyntaxHandler> {
        let file = utils::read_file(path);

        let decoded: Result<SyntaxJSON, serde_json::Error> = serde_json::from_str(&file);

        match decoded {
            Ok(decoded) => {
                return Some(SyntaxHandler {
                    structs: SyntaxStructs {
                        constant: regex::Regex::new(&decoded.s_constant).unwrap(),
                        keyword: regex::Regex::new(&decoded.s_keyword).unwrap(),
                        preproc: regex::Regex::new(&decoded.s_preproc).unwrap(),
                        data_type: regex::Regex::new(&decoded.s_data_type).unwrap(),
                        comment: regex::Regex::new(&decoded.s_comment).unwrap(),
                    },
                    colors: SyntaxColor {
                        constant: color![decoded.c_constant & 0xFF, (decoded.c_constant >> 8) & 0xFF, (decoded.c_constant >> 16) & 0xFF],
                        keyword: color![decoded.c_keyword & 0xFF, (decoded.c_keyword >> 8) & 0xFF, (decoded.c_keyword >> 16) & 0xFF],
                        preproc: color![decoded.c_preproc & 0xFF, (decoded.c_preproc >> 8) & 0xFF, (decoded.c_preproc >> 16) & 0xFF],
                        data_type: color![decoded.c_data_type & 0xFF, (decoded.c_data_type >> 8) & 0xFF, (decoded.c_data_type >> 16) & 0xFF],
                        comment: color![decoded.c_comment & 0xFF, (decoded.c_comment >> 8) & 0xFF, (decoded.c_comment >> 16) & 0xFF],
                        other: color![decoded.c_other & 0xFF, (decoded.c_other >> 8) & 0xFF, (decoded.c_other >> 16) & 0xFF],
                    }
                })
            },
            Err(e) => {
                println!["{}", e];
                return None;
            }
        }
    }

    pub fn get_line_color(line: &str, editor: &editor::Editor) -> Vec<Color> {
        let mut ret: Vec<Color> = Vec::new();

        match &editor.syntax_handler {
            Some(sh) => {
                let colors = &sh.colors;
                let structs = &sh.structs;

                let mut cur_x = 0;

                let mut c_iter = line.graphemes(true);
                let mut m_iter_vec = vec![structs.constant.find_iter(line),
                                          structs.keyword.find_iter(line),
                                          structs.preproc.find_iter(line),
                                          structs.data_type.find_iter(line),
                                          structs.comment.find_iter(line)];

                let mut m_vec = vec![m_iter_vec[0].next(),
                                     m_iter_vec[1].next(),
                                     m_iter_vec[2].next(),
                                     m_iter_vec[3].next(),
                                     m_iter_vec[4].next()];

                for c in c_iter {
                    let mut cur_color = 10;
                    let mut i = 0;
                    for m in m_vec.clone() {
                        match m {
                            Some(m) => {
                                if m.start() <= cur_x && cur_x < m.end() {
                                    cur_color = i;
                                }
                                else if cur_x >= m.end() {
                                    m_vec[i] = m_iter_vec[i].next();
                                }
                            },
                            None => ()
                        }
                        i += 1;
                    }

                    match cur_color {
                        0 => ret.push(colors.constant),
                        1 => ret.push(colors.keyword),
                        2 => ret.push(colors.preproc),
                        3 => ret.push(colors.data_type),
                        4 => ret.push(colors.comment),
                        _ => ret.push(colors.other),
                    }

                    cur_x += c.len()
                }
            },
            None => {
                let mut c_iter = line.graphemes(true);
                for _ in c_iter {
                    ret.push(Color{r: 255, g: 255, b: 255, a: 255});
                }
            }
        }

        return ret;
    }
}
