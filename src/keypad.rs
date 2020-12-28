use std::io::{Error, ErrorKind, Read};

use termion::event::Key;
use termion::input::TermRead;
use termion::AsyncReader;

pub struct Keypad {
    astdin: AsyncReader,
}

impl Keypad {
    pub fn new(stdin: AsyncReader) -> Self {
        Keypad { astdin: stdin }
    }

    pub fn keytest(&mut self) -> Result<(), Error> {
        for c in self.astdin.by_ref().keys() {
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

