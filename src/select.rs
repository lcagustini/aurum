use ::text;

pub struct Select {
    pub x1: usize,
    pub y1: usize,
    pub x2: usize,
    pub y2: usize,
}
impl Select {
    // TODO: fix multiline selection
    pub fn get_selected_text(&self, text: &text::Text) -> String {
        text.raw[self.y1][self.x1..self.x2].to_owned()
    }
}
