use std::io::{Error, ErrorKind, Read};

use termion::event::Key;
use termion::input::TermRead;
use termion::AsyncReader;

pub struct Keypad {
    astdin: AsyncReader, // asynchronous thread to handle input
}

impl Keypad {
    pub fn new(stdin: AsyncReader) -> Self {
        Keypad { astdin: stdin }
    }

    // TODO: spin out into testable keyhandler
    pub fn getkey(&mut self) -> Result<Option<Key>, Error> {
        let event = self.astdin.by_ref().keys().nth(0);
        match event {
            // no key pressed
            None => Ok(None),
            // handle keys that affect global state (exit program, for example)
            // otherwise return key to be handled in updating CPU state
            Some(keypress) => 
                match keypress {
                    Ok(key) => match key {
                        // special key handlers
                        Key::Ctrl('c') =>
                            Err(Error::new(ErrorKind::Interrupted, "exit CHIP8")),
                        _ => Ok(Some(key)),
                    },
                    Err(_) => Err(Error::new(ErrorKind::NotFound, "bad keypress!")),
                },
        }
    }
}

