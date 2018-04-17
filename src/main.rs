extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;
const BG_COLOR: Color = Color{r: 255, g: 25, b: 25, a: 255};

macro_rules! rect(
    ($x:expr, $y:expr, $w:expr, $h:expr) => (
        sdl2::rect::Rect::new($x as i32, $y as i32, $w as u32, $h as u32)
    )
);

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let ttf_context = sdl2::ttf::init().unwrap();

    let window = video_subsystem.window("Aurum", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .allow_highdpi()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();

    let mut font = ttf_context.load_font("roboto.ttf", 128).unwrap();
    font.set_style(sdl2::ttf::STYLE_NORMAL);

    canvas.set_draw_color(BG_COLOR);
    canvas.clear();
    canvas.present();

    let mut x = 0;
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }

        let surface = font.render("Hello Rust!").blended(Color::RGBA(255, 255, 255, 255)).unwrap();
        let texture = texture_creator.create_texture_from_surface(&surface).unwrap();
        let sdl2::render::TextureQuery { width, height, .. } = texture.query();

        if x == 300 {
            x = 0;
        }
        else {
            x += 1;
        }

        canvas.clear();
        canvas.copy(&texture, None, Some(rect![x, 0, width, height])).unwrap();
        canvas.present();
    }
}
