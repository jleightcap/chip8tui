use std::{
    io::{Error, ErrorKind, Read},
    fs::File,
};

use crate::cpu::{PC_BASE, RAM_SIZE};

// maximum available program memory
const ROM_SIZE: usize = RAM_SIZE - PC_BASE; 

#[derive(Clone, Debug, PartialEq)]
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

    #[allow(dead_code)] // initialize from hard-codeded vector, used in testing
    pub fn new_prog(p: &[u8]) -> Result<Self, Error> {
        if p.len() <= ROM_SIZE {
            let mut buf = [0; RAM_SIZE - PC_BASE];
            for ii in 0..p.len() { buf[ii] = p[ii]; }
            Ok(ROM {
                rom: buf,
            })
        }
        else {
            Err(Error::new(ErrorKind::InvalidData, "ROM program exceeds available memory!"))
        }
    }
}

#[cfg(test)]
#[path = "test/rom_test.rs"]
mod rom_test;
