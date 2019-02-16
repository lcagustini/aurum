extern crate sdl2;

use ::text;
use ::cursor;
use ::select;
use ::undo;
use ::search;
use ::syntax;
use ::autocomplete;
use ::config;

pub struct Editor<'ttf, 'r> {
    pub text: text::Text<'ttf, 'r>,
    pub cursor: cursor::Cursor<'r>,
    pub selected: select::SelectHandler,
    pub undo_handler: undo::UndoHandler,
    pub search_handler: search::SearchHandler,
    pub syntax_handler: Option<syntax::SyntaxHandler>,
    pub completion_engine: autocomplete::CompletionEngine,

    pub canvas: sdl2::render::Canvas<sdl2::video::Window>,
    pub char_timer: usize,
}
impl<'ttf, 'r> Editor<'ttf, 'r> {
    pub fn create(canvas: sdl2::render::Canvas<sdl2::video::Window>,
                  ttf_context: &'ttf sdl2::ttf::Sdl2TtfContext,
                  config: &config::Config) -> Editor<'ttf, 'r> {
        let mut font = ttf_context.load_font(&config.font_path, config.font_size).unwrap();
        font.set_style(sdl2::ttf::FontStyle::NORMAL);

        let lines: Vec<String> = vec!["".to_owned()];

        Editor{
            text: text::Text::new(font, lines, config),
            cursor: cursor::Cursor::new(0, 0, config),
            selected: select::SelectHandler{old_x: 0, old_y: 0, x1: 0, y1: 0, x2: 0, y2: 0},
            undo_handler: undo::UndoHandler::new(),
            search_handler: search::SearchHandler::new(),
            syntax_handler: None,
            completion_engine: autocomplete::CompletionEngine::new(),
            canvas: canvas,
            char_timer: 0,
        }
    }
}


