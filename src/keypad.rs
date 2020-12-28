use std::io::{Error, ErrorKind};

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::async_stdin;

pub struct Keypad {
    fuck: u8,
}

impl Keypad {
    pub fn new() -> Self {
        Keypad { fuck: 2 }
    }

    pub fn keytest(&self) -> Result<(), Error> {
        let stdin = async_stdin();
        for c in stdin.keys() {
            let key = c.unwrap();
            println!("{:?}", key);
            match key {
                Key::Ctrl('c') => {
                    return Err(Error::new(ErrorKind::Interrupted, "exit CHIP8"))
                },
                Key::Char('a') => {
                    return Err(Error::new(ErrorKind::Interrupted, "AAAAA"))
                }
                _ => { break; },
            }
        }
        Ok(())
    }
}

