extern crate regex;

use std::collections::HashSet;

use ::editor;

pub struct CompletionEngine {
    pub word_cache: HashSet<String>,

    spliting_regex: regex::Regex,
    word_regex: regex::Regex,
}
impl CompletionEngine {
    pub fn new() -> CompletionEngine {
        let reg1 = regex::Regex::new(r"\W+").unwrap();
        let reg2 = regex::Regex::new(r"\w+").unwrap();

        CompletionEngine{word_cache: HashSet::new(), spliting_regex: reg1, word_regex: reg2}
    }

    pub fn update_cache(&mut self, line: &str) {
        for word in self.spliting_regex.split(line) {
            if word.len() > 1 {
                self.word_cache.insert(word.to_owned());
            }
        }
    }

    pub fn complete(&self, editor: &editor::Editor) -> Vec<String> {
        let mut ret = Vec::new();
        let iter = self.word_regex.find_iter(&editor.text.raw[editor.cursor.get_absolute_y()]);
        for word in iter {
            if (word.start() as u32) <= editor.cursor.x && editor.cursor.x <= (word.end() as u32) {
                let cache_iter = editor.completion_engine.word_cache.iter();
                for cached_word in cache_iter {
                    if cached_word.starts_with(word.as_str()) {
                        ret.push(cached_word.to_owned());
                    }
                }

                break;
            }
        }

        return ret;
    }
}
