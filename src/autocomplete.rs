extern crate regex;

use std::collections::HashSet;

use ::cursor;

pub struct CompletionEngine {
    pub list_mode: bool,
    pub selected_word: usize,

    pub cur_word: String,
    pub completion_list: Vec<String>,

    word_cache: HashSet<String>,

    spliting_regex: regex::Regex,
    word_regex: regex::Regex,
}
impl CompletionEngine {
    pub fn new() -> CompletionEngine {
        let reg1 = regex::Regex::new(r"\W+").unwrap();
        let reg2 = regex::Regex::new(r"\w+").unwrap();

        CompletionEngine{cur_word: "".to_owned(), selected_word: 0, completion_list: Vec::new(), word_cache: HashSet::new(), list_mode: false, spliting_regex: reg1, word_regex: reg2}
    }

    pub fn update_cache(&mut self, line: &str) {
        for word in self.spliting_regex.split(line) {
            if word.len() > 1 {
                self.word_cache.insert(word.to_owned());
            }
        }
    }

    pub fn complete(&mut self, text: &Vec<String>, cursor: &cursor::Cursor) {
        let mut ret = Vec::new();
        let mut cur = "".to_owned();

        let iter = self.word_regex.find_iter(&text[cursor.get_absolute_y()]);
        for word in iter {
            if (word.start() as u32) <= cursor.x && cursor.x <= (word.end() as u32) {
                cur = word.as_str().to_owned();

                let cache_iter = self.word_cache.iter();
                for cached_word in cache_iter {
                    if cached_word.starts_with(&cur) {
                        ret.push(cached_word.to_owned());
                    }
                }

                break;
            }
        }

        self.cur_word = cur;
        self.completion_list = ret;
    }
}
