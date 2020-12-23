extern crate sdl2;

use std::env;

mod screen;
mod cpu;
mod rom;

use screen::Screen;
use cpu::Cpu;
use rom::ROM;

const STACK_SIZE: usize = 16;
const RAM_SIZE:   usize = 4096;
const REG_COUNT:  usize = 16;
const PC_BASE:    usize = 0x200;
const V_WIDTH:    usize = 64;
const V_HEIGHT:   usize = 32;

fn main() {
    let fname = env::args().nth(1).expect("expected input file!");

    let r: ROM = ROM::new_file(&fname);
    let mut c: Cpu = Cpu::new();
    c.prog_init(&r);

    let sdl_context = sdl2::init().unwrap();
    let mut screen = Screen::new(&sdl_context);


    loop {
        screen.draw(&c.vram);
        c.mcycle();
    }
}
