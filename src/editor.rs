extern crate sdl2;

use std::env;

use ::utils;
use ::text;
use ::cursor;
use ::select;
use ::undo;
use ::search;

use ::FONT_SIZE;

pub struct Editor<'ttf, 'r> {
    pub text: text::Text<'ttf, 'r>,
    pub cursor: cursor::Cursor<'r>,
    pub selected: select::SelectHandler,
    pub undo_handler: undo::UndoHandler,
    pub search_handler: search::SearchHandler,

    pub canvas: sdl2::render::Canvas<sdl2::video::Window>,
}
impl<'ttf, 'r> Editor<'ttf, 'r> {
    pub fn create(canvas: sdl2::render::Canvas<sdl2::video::Window>, ttf_context: &'ttf sdl2::ttf::Sdl2TtfContext) -> Editor<'ttf, 'r> {
        let args: Vec<String> = env::args().collect();

        let mut font = ttf_context.load_font("roboto.ttf", FONT_SIZE).unwrap();
        font.set_style(sdl2::ttf::STYLE_NORMAL);

        let lines: Vec<String> =
            if args.len() > 1 {
                utils::read_file(&args[1]).split("\n").map(|x| x.to_owned()).collect()
            }
            else {
                vec!["".to_owned()]
            };

        Editor{
            text: text::Text::new(font, lines, args),
            cursor: cursor::Cursor::new(0, 0),
            selected: select::SelectHandler{x1: 0, y1: 0, x2: 0, y2: 0},
            undo_handler: undo::UndoHandler::new(),
            search_handler: search::SearchHandler::new(),
            canvas: canvas,
        }
    }
}
