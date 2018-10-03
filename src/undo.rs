use ::cursor;
use ::text;

struct UndoState {
    cursor_x: u32,
    cursor_y: u32,

    cursor_wanted_x: u32,
    cursor_screen_y: u32,
    cursor_number_w: u32,

    text: Vec<String>,
}

pub struct UndoHandler {
    states: Vec<UndoState>,
    cur_state: usize,
}
impl UndoHandler {
    pub fn new() -> UndoHandler {
        UndoHandler{states: Vec::new(), cur_state: 0}
    }

    pub fn create_state(&mut self, cursor: &cursor::Cursor, text: &text::Text) {
        let new_state = UndoState{cursor_x: cursor.x,
                                  cursor_y: cursor.y,
                                  cursor_wanted_x: cursor.wanted_x,
                                  cursor_screen_y: cursor.screen_y,
                                  cursor_number_w: cursor.number_w,
                                  text: text.raw.clone()};

        if self.cur_state != self.states.len() {
            self.states.truncate(self.cur_state+1);
        }
        self.states.push(new_state);
        self.cur_state += 1;
    }

    pub fn clear_states(&mut self) {
        self.cur_state = 0;
        self.states.clear();
    }

    pub fn restore_previous_state(&mut self, cursor: &mut cursor::Cursor, text: &mut text::Text) {
        if self.cur_state > 0 {
            self.cur_state -= 1;
            let state = &self.states[self.cur_state];

            cursor.x = state.cursor_x;
            cursor.y = state.cursor_y;
            cursor.wanted_x = state.cursor_wanted_x;
            cursor.screen_y = state.cursor_screen_y;
            cursor.number_w = state.cursor_number_w;
            text.raw = state.text.clone();
        }
    }

    pub fn restore_next_state(&mut self, cursor: &mut cursor::Cursor, text: &mut text::Text) {
        if self.cur_state < self.states.len()-1 {
            self.cur_state += 1;
            let state = &self.states[self.cur_state];

            cursor.x = state.cursor_x;
            cursor.y = state.cursor_y;
            cursor.wanted_x = state.cursor_wanted_x;
            cursor.screen_y = state.cursor_screen_y;
            cursor.number_w = state.cursor_number_w;
            text.raw = state.text.clone();
        }
    }
}

