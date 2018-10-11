pub struct SearchHandler {
    pub active: bool,
    pub search_string: String,

    pub cur_index: usize,
    pub found_places: Vec<(u32, u32)>
}
impl SearchHandler {
    pub fn new() -> SearchHandler {
        SearchHandler{active: false, search_string: "".to_owned(), cur_index: 0, found_places: Vec::new()}
    }

    fn recursive_find(slice: &str, searching: &str, bias: usize) -> Vec<usize> {
        let r = slice.find(searching);
        match r {
            Some(x) => {
                let mut vec = Self::recursive_find(&slice[x+1..], searching, bias+x+1);
                vec.push(x+bias);
                return vec;
            },
            None => return Vec::new()
        }
    }

    pub fn find_search_string(&mut self, text: &Vec<String>) {
        self.found_places.clear();
        self.cur_index = 0;
        if self.search_string.len() > 0 {
            for (y, line) in text.iter().enumerate() {
                let vec = Self::recursive_find(line, &self.search_string, 0);
                for x in vec {
                    self.found_places.push((x as u32, y as u32));
                }
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
