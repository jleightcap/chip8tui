use std::io::{Error, ErrorKind, Read};

use termion::{
    event::Key,
    input::TermRead,
    AsyncReader,
};

pub struct Keypad {
    astdin: AsyncReader, // asynchronous thread to handle input
}

impl Keypad {
    pub fn new(stdin: AsyncReader) -> Self {
        Keypad { astdin: stdin }
    }

    fn getkey(&mut self) -> Option<Result<Key, Error>> {
        self.astdin.by_ref().keys().nth(0)
    }

    pub fn poll_reader(&mut self) -> Result<[bool; 16], Error> {
        let k = self.getkey();
        match k {
            None        => Ok([false; 16]),
            Some(event) => match event {
                Err(_)  => Err(Error::new(ErrorKind::InvalidData, "bad key!")),
                Ok(key) => poll(Some(key)),
            },
        }
    }
}

// poll the keypad state
pub fn poll(event: Option<Key>) -> Result<[bool; 16], Error> {
    let mut keystate = [false; 16];
    match event {
        None    => Ok(keystate), // no keys pressed
        Some(k) => match k {
                Key::Ctrl('c') =>
                    Err(Error::new(ErrorKind::Interrupted, "exit CHIP8")),
                Key::Char('1') => { keystate[0x0] = true; Ok(keystate) },
                Key::Char('2') => { keystate[0x1] = true; Ok(keystate) },
                Key::Char('3') => { keystate[0x2] = true; Ok(keystate) },
                Key::Char('4') => { keystate[0x3] = true; Ok(keystate) },
                Key::Char('q') => { keystate[0x4] = true; Ok(keystate) },
                Key::Char('w') => { keystate[0x5] = true; Ok(keystate) },
                Key::Char('e') => { keystate[0x6] = true; Ok(keystate) },
                Key::Char('r') => { keystate[0x7] = true; Ok(keystate) },
                Key::Char('a') => { keystate[0x8] = true; Ok(keystate) },
                Key::Char('s') => { keystate[0x9] = true; Ok(keystate) },
                Key::Char('d') => { keystate[0xa] = true; Ok(keystate) },
                Key::Char('f') => { keystate[0xb] = true; Ok(keystate) },
                Key::Char('z') => { keystate[0xc] = true; Ok(keystate) },
                Key::Char('x') => { keystate[0xd] = true; Ok(keystate) },
                Key::Char('c') => { keystate[0xe] = true; Ok(keystate) },
                Key::Char('v') => { keystate[0xf] = true; Ok(keystate) },
                _ => Ok(keystate),
            },
    }
}

#[cfg(test)]
#[path = "test/keypad_test.rs"]
mod keypad_test;
