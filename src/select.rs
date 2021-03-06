use ::text;

pub struct SelectHandler {
    pub old_x: usize,
    pub old_y: usize,

    pub x1: usize,
    pub y1: usize,
    pub x2: usize,
    pub y2: usize,
}
impl SelectHandler {
    pub fn get_selected_text(&self, text: &text::Text) -> String {
        if self.y1 == self.y2 {
            return text.raw[self.y1][self.x1..self.x2].to_owned();
        }
        else {
            let mut selected_text: String = "".to_owned();
            for i in self.y1..=self.y2 {
                if i == self.y1 {
                    selected_text.push_str(&text.raw[i][self.x1..]);
                }
                else if i == self.y2 {
                    selected_text.push_str(&text.raw[i][..self.x2]);
                }
                else {
                    selected_text.push_str(&text.raw[i][..]);
                }
                selected_text.push_str("\n");
            }
            return selected_text;
        }
    }

    pub fn reset_selection(&mut self) {
        self.x1 = 0;
        self.y1 = 0;
        self.x2 = 0;
        self.y2 = 0;
    }
}
