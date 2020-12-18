use std::env;

mod cpu;
mod rom;

use cpu::Cpu;
use rom::ROM;

fn main() {
    let fname = env::args().nth(1).expect("expected input file!");

    let r: ROM = ROM::new(&fname);
    let c: Cpu = Cpu::new();

    println!("{:?}", r.rom[0]);
}
