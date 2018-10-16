extern crate serde;
extern crate serde_json;
extern crate regex;

use ::utils;
use ::editor;

use sdl2::pixels::Color;

use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Serialize, Deserialize)]
struct SyntaxJSON {
    c_constant: [u8; 3],
    c_keyword: [u8; 3],
    c_secondary_word: [u8; 3],
    c_preproc: [u8; 3],
    c_data_type: [u8; 3],
    c_comment: [u8; 3],
    c_other: [u8; 3],

    s_constant: String,
    s_keyword: String,
    s_secondary_word: String,
    s_preproc: String,
    s_data_type: String,
    s_comment: String,
}

#[derive(Debug)]
pub struct SyntaxColor {
    pub constant: Color,
    pub keyword: Color,
    pub secondary_word: Color,
    pub preproc: Color,
    pub data_type: Color,
    pub comment: Color,
    pub other: Color,
}

#[derive(Debug)]
pub struct SyntaxStructs {
    pub constant: regex::Regex,
    pub keyword: regex::Regex,
    pub secondary_word: regex::Regex,
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
                        secondary_word: regex::Regex::new(&decoded.s_secondary_word).unwrap(),
                        preproc: regex::Regex::new(&decoded.s_preproc).unwrap(),
                        data_type: regex::Regex::new(&decoded.s_data_type).unwrap(),
                        comment: regex::Regex::new(&decoded.s_comment).unwrap(),
                    },
                    colors: SyntaxColor {
                        constant: color![decoded.c_constant[0], decoded.c_constant[1], decoded.c_constant[2]],
                        keyword: color![decoded.c_keyword[0], decoded.c_keyword[1], decoded.c_keyword[2]],
                        secondary_word: color![decoded.c_secondary_word[0], decoded.c_secondary_word[1], decoded.c_secondary_word[2]],
                        preproc: color![decoded.c_preproc[0], decoded.c_preproc[1], decoded.c_preproc[2]],
                        data_type: color![decoded.c_data_type[0], decoded.c_data_type[1], decoded.c_data_type[2]],
                        comment: color![decoded.c_comment[0], decoded.c_comment[1], decoded.c_comment[2]],
                        other: color![decoded.c_other[0], decoded.c_other[1], decoded.c_other[2]],
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
                                          structs.secondary_word.find_iter(line),
                                          structs.preproc.find_iter(line),
                                          structs.data_type.find_iter(line),
                                          structs.comment.find_iter(line)];

                let mut m_vec = vec![m_iter_vec[0].next(),
                                     m_iter_vec[1].next(),
                                     m_iter_vec[2].next(),
                                     m_iter_vec[3].next(),
                                     m_iter_vec[4].next(),
                                     m_iter_vec[5].next()];

                for c in c_iter {
                    let mut cur_color = 10;
                    let mut i = 0;
                    for m in m_vec.clone() {
                        match m {
                            Some(m) => {
                                if m.start() <= cur_x && cur_x < m.end() {
                                    cur_color = i;
                                }
                                if cur_x == m.end()-1 {
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
                        2 => ret.push(colors.secondary_word),
                        3 => ret.push(colors.preproc),
                        4 => ret.push(colors.data_type),
                        5 => ret.push(colors.comment),
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



