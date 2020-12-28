use std::io;

use termion::raw::{IntoRawMode,RawTerminal};
use tui::Terminal;
use tui::backend::TermionBackend;
use tui::widgets::{Widget, Block, Borders};
use tui::layout::{Layout, Constraint, Direction};

pub const V_WIDTH:    usize = 64;
pub const V_HEIGHT:   usize = 32;

pub struct Screen {
    term: Terminal<TermionBackend<RawTerminal<io::Stdout>>>,
    chunks: tui::layout::Layout,
}

impl Screen {
    pub fn new() -> Result<Self, io::Error> {
        let stdout = io::stdout().into_raw_mode()?;
        let backend = TermionBackend::new(stdout);
        let mut term = Terminal::new(backend)?;
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Min(V_HEIGHT as u16 + 2)
                ].as_ref()
            );
        term.clear()?;
        Ok(Screen {
            term: term,
            chunks: chunks,
        })
    }

    pub fn render(&mut self, pixels: &[[u8; V_WIDTH]; V_HEIGHT]) {
        let mut chunks = self.chunks.clone();
        self.term.draw(|f| {
            // check terminal is big enough to render at least the screen
            assert!(f.size().width  >= V_WIDTH  as u16 + 2);
            assert!(f.size().height >= V_HEIGHT as u16 + 2);
            let chunks = chunks.split(f.size());

            let block = Block::default()
                .title("Screen")
                .borders(Borders::ALL);

            f.render_widget(block, chunks[0]);
        }).unwrap();
    }
}
