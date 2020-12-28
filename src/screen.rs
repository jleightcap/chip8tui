use std::io;

use termion::raw::{IntoRawMode,RawTerminal};
use tui::Terminal;
use tui::backend::TermionBackend;
use tui::widgets::{Widget, Block, Borders, BorderType};
use tui::widgets::canvas::{Canvas, Points};
use tui::style::{Style, Color};
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
                    Constraint::Min(V_HEIGHT as u16 + 2)
                ].as_ref()
            );

        Ok(Screen {
            term:   term,
            chunks: chunks,
        })
    }

    pub fn render(&mut self, pixels: &[[u8; V_WIDTH]; V_HEIGHT]) {
        // copy out config pieces of the screen struct that need coercing 
        //
        // the |f| lambda below makes borrowing weird and I'm losing arguments
        // with the compiler
        let chunks = self.chunks.clone();
        self.term.draw(|f| {
            // check terminal is big enough to render at least the screen
            //assert!(f.size().width  >= V_WIDTH  as u16 + 2);
            //assert!(f.size().height >= V_HEIGHT as u16 + 2);
            let chunks = chunks.split(f.size());

            let canvas =
                Canvas::default()
                .block(Block::default().title("CHIP8").borders(Borders::ALL))
                .x_bounds([0.0, V_WIDTH  as f64])
                .y_bounds([0.0, V_HEIGHT as f64])
                .paint(|ctx| {

                    let mut pts: Vec<(f64, f64)> = Vec::new();
                    for ii in 0..pixels.len() {
                        for jj in 0..pixels.len() {
                            if pixels[ii][jj] == 1 { pts.push((ii as f64, jj as f64)) };
                        }
                    }

                    ctx.draw(&Points {
                        coords: &pts,
                        color: Color::White,
                    });
                });


            f.render_widget(canvas, chunks[0]);
        }).unwrap();
    }
}
