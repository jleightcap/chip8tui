use sdl2;
use sdl2::pixels;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::V_WIDTH;
use crate::V_HEIGHT;

const SCALE: u32 = 20;
const SCREEN_WIDTH:  u32 = (V_WIDTH  as u32) * SCALE;
const SCREEN_HEIGHT: u32 = (V_HEIGHT as u32) * SCALE;

pub struct Screen {
    canvas:     Canvas<Window>,
}
impl Screen {
    pub fn new(sdl_context: &sdl2::Sdl) -> Self {
        let video = sdl_context.video().unwrap();
        let window = video
            .window(
                "rust-sdl2_gfx: draw line & FPSManager",
                SCREEN_WIDTH,
                SCREEN_HEIGHT,
            )
            .position_centered()
            .opengl()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();

        canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        Screen { canvas: canvas }
    }

    pub fn draw(&mut self, pixels: &[[u8; V_WIDTH]; V_HEIGHT]) {
        for (y, &row) in pixels.iter().enumerate() {
            for (x, &col) in row.iter().enumerate() {
                let x = (x as u32) * SCALE;
                let y = (y as u32) * SCALE;

                self.canvas.set_draw_color(color(col));
                self.canvas.fill_rect(Rect::new(x as i32, y as i32, SCALE, SCALE)).unwrap();
            }
        }
        self.canvas.present();
    }
}

fn color(v: u8) -> pixels::Color {
    if v == 0 { pixels::Color::RGB(0, 0,   0) }
    else      { pixels::Color::RGB(0, 250, 0) }
}
