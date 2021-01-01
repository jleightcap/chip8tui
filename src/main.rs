extern crate clap;

use clap::{Arg, App};
use std::io;
use termion::async_stdin; // asynchronous stdin thread for non-blocking keypresses

mod screen; use screen::Screen;
mod keypad; use keypad::Keypad;
mod cpu;    use cpu::Cpu;
mod rom;    use rom::ROM;


fn main() -> Result<(), io::Error> {
    let matches = 
        App::new("CHIP8 TUI Emulator")
            .version("1.0")
            .author("Jack Leightcap <jleightcap@protonmail.com>")
            .about("Emulate CHIP8 architecture entirely in the terminal")
            .arg(Arg::with_name("INPUT")
                 .help("Sets input ROM")
                 .required(true)
                 .index(1))
            .arg(Arg::with_name("verbose")
                 .help("Print emulator debug state information")
                 .short("v")
                 .long("verbose"))
            .arg(Arg::with_name("nographic")
                 .help("Run emulator without TUI interface")
                 .short("n")
                 .long("nographic"))
            .get_matches();
    let fname = matches.value_of("INPUT").unwrap();
    let verbo = matches.is_present("verbose");
    let blank = matches.is_present("nographic");

    // components
    let mut screen = Screen::new(!blank)?;
    let mut k = Keypad::new(async_stdin());
    let r = ROM::new_file(&fname);

    let mut c: Cpu = Cpu::new(Some(r), verbo)?;


    loop {
        let key = k.poll_reader()?;
        c.mcycle(key)?;
        match &mut screen {
            Some(s) => s.render(&c.vram),
            None    => (),
        }
    }
}
