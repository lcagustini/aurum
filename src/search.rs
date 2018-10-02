pub struct SearchHandler {
    pub active: bool,
    pub search_string: String,

    cur_index: usize,
    pub found_places: Vec<(u32, u32)>
}
impl SearchHandler {
    pub fn new() -> SearchHandler {
        SearchHandler{active: false, search_string: "".to_owned(), cur_index: 0, found_places: Vec::new()}
    }

    pub fn find_search_string(&mut self, text: &Vec<String>) {
        self.found_places.clear();
        self.cur_index = 0;
        for (y, line) in text.iter().enumerate() {
            let r = line.find(&self.search_string);
            match r {
                Some(x) => self.found_places.push((x as u32, y as u32)),
                _ => ()
            }
        }
    }

    pub fn next_string_pos(&mut self) -> Option<(u32, u32)> {
        if self.found_places.len() > 0 {
            let ret = self.found_places[self.cur_index];
            self.cur_index += 1;
            if self.cur_index == self.found_places.len() {
                self.cur_index = 0;
            }
            return Some(ret);
        }
        return None
    }
}
