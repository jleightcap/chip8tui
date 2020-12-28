use std::env;
use std::io;

mod screen;
mod cpu;
mod rom;

use screen::Screen;
use cpu::Cpu;
use rom::ROM;

fn main() -> Result<(), io::Error> {
    let fname = env::args().nth(1).expect("expected input file!");

    let r: ROM = ROM::new_file(&fname);
    let mut c: Cpu = Cpu::new();
    c.prog_init(&r);

    let mut screen = Screen::new()?;
    for _ in 0..100 {
        screen.render(&c.vram);
    }
    Ok(())
}
