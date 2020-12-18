use std::io::Read;
use std::fs::File;

const ROM_SIZE: usize = 3584;

pub struct ROM {
    pub rom: [u8; ROM_SIZE],
}

impl ROM {
    pub fn new(path: &str) -> Self {
        let mut f = File::open(path).expect("invalid path");
        let mut rom = [0; ROM_SIZE];
        f.read(&mut rom).expect("file to buffer");

        ROM {
            rom: rom,
        }
    }
}
