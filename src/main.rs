use std::env;
use std::io;
use termion::async_stdin; // asynchronous stdin thread for non-blocking keypresses

mod screen; use screen::Screen;
mod keypad; use keypad::Keypad;
mod cpu; use cpu::Cpu;
mod rom; use rom::ROM;


fn main() -> Result<(), io::Error> {
    let fname = env::args().nth(1).expect("expected input file!");

    let r: ROM = ROM::new_file(&fname);
    let mut c: Cpu = Cpu::new();
    c.prog_init(&r);

    let mut screen = Screen::new()?;
    let mut k = Keypad::new(async_stdin());
    loop {
        k.keytest()?;
        c.mcycle();
        screen.render(&c.vram);
    }
}
