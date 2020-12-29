use std::io;

use termion::raw::{IntoRawMode,RawTerminal};
use tui::Terminal;
use tui::backend::TermionBackend;
use tui::widgets::{Block, Borders};
use tui::widgets::canvas::{Canvas, Points};
use tui::style::{Color};
use tui::layout::{Layout, Constraint, Direction};

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
        // copy out config pieces of the screen struct that need coercing 
        //
        // the |f| lambda below makes borrowing weird and I'm losing arguments
        // with the compiler
        let chunks = self.chunks.clone();
        self.term.draw(|f| {
            let canvas =
                Canvas::default()
                .block(Block::default().title("CHIP8").borders(Borders::ALL))
                .x_bounds([0.0, V_WIDTH  as f64])
                .y_bounds([0.0, V_HEIGHT as f64])
                .paint(|ctx| {
                    let mut pts: Vec<(f64, f64)> = Vec::new();
                    for ii in 0..pixels.len() {
                        for jj in 0..pixels[ii].len() {
                            let px = ii;
                            let py = (V_HEIGHT as i32 - jj as i32).abs() as usize;
                            if pixels[ii][jj] == 1 { pts.push((px as f64, py as f64)) };
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
