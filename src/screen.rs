use std::io;

use termion::raw::{IntoRawMode,RawTerminal};
use tui::{
    Terminal,
    backend::TermionBackend,
    widgets::{
        Block,
        Borders,
        canvas::{
            Canvas,
            Points
        },
    },
    style::Color,
    symbols::Marker,
    layout::{
        Layout,
        Constraint,
        Direction
    },
};

pub const V_WIDTH:    usize = 64;
pub const V_HEIGHT:   usize = 32;

pub struct Screen {
    term:           Terminal<TermionBackend<RawTerminal<io::Stdout>>>,
    chunks:         tui::layout::Layout,
}

impl Screen {
    pub fn new() -> Result<Self, io::Error> {
        let stdout = io::stdout().into_raw_mode()?;
        let backend = TermionBackend::new(stdout);
        let mut term = Terminal::new(backend)?;
        term.clear()?;

        // layout constraints and configuration
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Length(V_HEIGHT as u16 + 2)
                ].as_ref()
            );

        Ok(Screen {
            term:   term,
            chunks: chunks,
        })
    }

    // TODO: pass vram as compile-time constant &[&[u8]]
    pub fn render(&mut self, pixels: &[[u8; V_HEIGHT]; V_WIDTH]) {
        // I hope I can come back to this and think:
        // "man, I *was* bad at Rust"
        //
        // - self.chunks inside the |f| closure
        // - move canvas into Screen struct
        let chunks = self.chunks.clone();
        self.term.draw(|f| {
            let canvas =
                Canvas::default()
                .block(Block::default().title("CHIP8").borders(Borders::ALL))
                .marker(Marker::Block)
                .x_bounds([0.0, V_WIDTH  as f64])
                .y_bounds([0.0, V_HEIGHT as f64])
                .paint(|ctx| {
                    let mut pts: Vec<(f64, f64)> = Vec::new();
                    for ii in 0..pixels.len() {
                        for jj in 0..pixels[ii].len() {
                            let px = ii as f64;
                            // convert Canvas y coordinate to graphics coordinates
                            let py = (V_HEIGHT as f64 - jj as f64).abs();
                            if pixels[ii][jj] == 1 { pts.push((px, py)) };
                        }
                    }

                    ctx.draw(&Points {
                        coords: &pts,
                        color: Color::White,
                    });
                });

            let chunks = chunks.split(f.size());
            f.render_widget(canvas, chunks[0]);
        }).unwrap();
    }
}
