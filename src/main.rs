use std::env;
use std::io;
use termion::async_stdin; // asynchronous stdin thread for non-blocking keypresses

mod screen; use screen::Screen;
mod keypad; use keypad::Keypad;
mod cpu; use cpu::Cpu;
mod rom; use rom::ROM;


fn main() -> Result<(), io::Error> {
    let fname = env::args().nth(1).expect("expected input file!");

    // components
    let mut screen = Screen::new()?;
    let k = Keypad::new(async_stdin());
    let r = ROM::new_file(&fname);

    let mut c: Cpu = Cpu::new(Some(r), Some(k))?;


    loop {
        c.mcycle()?;
        screen.render(&c.vram);
    }
}
