use std::io::Read;
use std::fs::File;

use crate::cpu::RAM_SIZE;
use crate::cpu::PC_BASE;

// maximum available program memory
const ROM_SIZE: usize = RAM_SIZE - PC_BASE; 

pub struct ROM {
    pub rom:    [u8; ROM_SIZE],
}

impl ROM {
    pub fn new_file(path: &str) -> Self {
        let mut f = File::open(path).expect("invalid path");
        let mut rom = [0; ROM_SIZE];
        f.read(&mut rom).expect("file to buffer");

        ROM {
            rom: rom,
        }
    }

    pub fn new_prog(p: &[u8]) -> Self {
        if p.len() > ROM_SIZE {
            panic!("ROM program exceeds available memory!");
        }
        else {
            let mut buf = [0; RAM_SIZE - PC_BASE];
            for ii in 0..p.len() { buf[ii] = p[ii]; }
            ROM {
                rom: buf,
            }
        }
    }
}
