use std::io::{Error, ErrorKind};
use termion::event::Key;

use crate::screen::V_WIDTH;
use crate::screen::V_HEIGHT;
use crate::rom::ROM;

const OP_LEN: usize = 2; // number of words in an opcode
enum PC {
    I,           // pc += 1*OP_LEN (increment)
    C,           // pc += 2*OP_LEN (condition, skip instrution)
    J(usize),    // pc  = arg (absolute jump)
}
impl PC {
    fn cond(q: bool) -> PC {
        if q { PC::I } // condition met, increment
        else { PC::C } // condition umnet, skip
    }
}

// built-in sprites for drawing hexadecimal digits '0' -> 'F'
// stored in the interpreter memory (0x000 -> 0x1ff)
// http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#font
/* Font {{{ */
pub const FONT: [u8; 80] = [
    0xf0, /* ####    */
    0x90, /* #  #    */
    0x90, /* #  #    */
    0x90, /* #  #    */
    0xf0, /* ####    */

    0x20, /*   #     */
    0x60, /*  ##     */
    0x20, /*   #     */
    0x20, /*   #     */
    0x70, /*  ###    */

    0xf0, /* ####    */
    0x10, /*    #    */
    0xf0, /* ####    */
    0x80, /* #       */
    0xf0, /* ####    */

    0xf0, /* ####    */
    0x10, /*    #    */
    0xf0, /* ####    */
    0x10, /*    #    */
    0xf0, /* ####    */

    0x90, /* #  #    */
    0x90, /* #  #    */
    0xf0, /* ####    */
    0x10, /*    #    */
    0x10, /*    #    */

    0xf0, /* ####    */
    0x80, /* #       */
    0xf0, /* ####    */
    0x10, /*    #    */
    0xf0, /* ####    */

    0xf0, /* ####    */
    0x80, /* #       */
    0xf0, /* ####    */
    0x90, /* #  #    */
    0xf0, /* ####    */

    0xf0, /* ####    */
    0x10, /*    #    */
    0x20, /*   #     */
    0x40, /*  #      */
    0x40, /*  #      */

    0xf0, /* ####    */
    0x90, /* #  #    */
    0xf0, /* ####    */
    0x90, /* #  #    */
    0xf0, /* ####    */

    0xf0, /* ####    */
    0x90, /* #  #    */
    0xf0, /* ####    */
    0x10, /*    #    */
    0xf0, /* ####    */

    0xf0, /* ####    */
    0x90, /* #  #    */
    0xf0, /* ####    */
    0x90, /* #  #    */
    0x90, /* #  #    */

    0xe0, /* ###     */
    0x90, /* #  #    */
    0xe0, /* ###     */
    0x90, /* #  #    */
    0xe0, /* ###     */

    0xf0, /* ####    */
    0x80, /* #       */
    0x80, /* #       */
    0x80, /* #       */
    0xf0, /* ####    */

    0xe0, /* ###     */
    0x90, /* #  #    */
    0x90, /* #  #    */
    0x90, /* #  #    */
    0xe0, /* ###     */

    0x0f, /* ####    */
    0x80, /* #       */
    0xf0, /* ####    */
    0x80, /* #       */
    0xf0, /* ####    */

    0xf0, /* ####    */
    0x80, /* #       */
    0xf0, /* ####    */
    0x80, /* #       */
    0x80, /* #       */
];

/* }}} */

pub const STACK_SIZE: usize = 16;
pub const RAM_SIZE:   usize = 4096;
pub const REG_COUNT:  usize = 16;
pub const PC_BASE:    usize = 0x200;
pub struct Cpu {
    /* memory */
    ram:        [u8; RAM_SIZE],             // RAM tape
    pub vram:   [[u8; V_HEIGHT]; V_WIDTH],  // video RAM

    /* stack */
    sp:         usize,                      // stack pointer
    s:          [usize; STACK_SIZE],        // stack memory

    /* registers */
    v:          [u8; REG_COUNT],            // general registers
    i:          usize,                      // index register
    pc:         usize,                      // program counter
    prog:       Option<ROM>,                // store program (restore from reset)
}
impl Cpu {
    pub fn new(prog: Option<ROM>) -> Result<Self, Error> {
        let ram = Cpu::ram_init(&prog)?;
        Ok(Cpu {
            ram:    ram,
            vram:   [[0; V_HEIGHT]; V_WIDTH],
            sp:     0x0,
            s:      [0; STACK_SIZE],
            i:      0x000,
            pc:     PC_BASE,
            v:      [0; REG_COUNT],
            prog:   prog,
        })
    }

    fn ram_init(rom: &Option<ROM>) -> Result<[u8; RAM_SIZE], Error> {
        let mut ram = [0; RAM_SIZE];
        // always initialize font in RAM
        for ii in 0..FONT.len() {
            ram[ii] = FONT[ii]; // XXX: is this a memcpy?
        }
        match rom {
            None => { },
            Some(r) => {
                if PC_BASE + r.rom.len() > RAM_SIZE {
                    return Err(Error::new(ErrorKind::InvalidData, "Program exceeds RAM!"));
                }
                for ii in 0..r.rom.len() {
                    ram[PC_BASE + ii] = r.rom[ii];
                }
            },
        };
        Ok(ram)
    }

    // update CPU state based on key pressed key (or lack thereof)
    pub fn keyhandle(&mut self, kp: Option<Key>) {
        match kp {
            None    => { },
            Some(k) => {
                match k {
                    Key::Ctrl('r') => { self.reset().unwrap(); },
                    _ => { },
                }
            },
        }
    }

    // insert an opcode at the current PC
    #[allow(dead_code)]
    fn opcode_init(&mut self, op: u16) {
        self.ram[self.pc]     = ((op & 0xff00) >> 8) as u8;
        self.ram[self.pc + 1] = ((op & 0x00ff) >> 0) as u8;
    }

    fn reset(&mut self) -> Result<(), Error> {
        *self = Cpu::new(self.prog.clone())?;
        Ok(())
    }

    fn fetch(&self) -> u16 {
        (self.ram[self.pc] as u16) << 8 | (self.ram[self.pc + 1] as u16)
    }

    // one machine cycle
    pub fn mcycle(&mut self) -> Result<(), Error> {
        self.icycle()?;
        Ok(())
    }

    // one instruction cycle (variable machine cycle)
    // super handy specification:
    // http://johnearnest.github.io/Octo/docs/chip8ref.pdf
    fn icycle(&mut self) -> Result<(), Error> {
        let op = self.fetch();
        //println!("fetch [{:#08x}] {:#06x}", self.pc, op);
        // split 2-byte opcode into 4 nibbles
        let nibs = (
            (op & 0xf000) >> 12 as u8,
            (op & 0x0f00) >> 8  as u8,
            (op & 0x00f0) >> 4  as u8,
            (op & 0x000f) >> 0  as u8,
        );
        let nnn = (op & 0x0fff) as usize; // address
        let nn  = (op & 0x00ff) as u8;    // 8-bit constant
        let x   = nibs.1 as usize;        // 4-bit constant
        let y   = nibs.2 as usize;        // 4-bit constant
        let n   = nibs.3 as usize;        // opcode argument

        let cycle_count: PC = match nibs {
            (0x0, 0x0, 0xe, 0x0) => self.op_00e0(),         /* clear */
            (0x0, 0x0, 0xe, 0xe) => self.op_00ee(),         /* return */
            (0x1, _,   _,   _  ) => self.op_1nnn(nnn),      /* jump nnn */
            (0x2, _,   _,   _  ) => self.op_2nnn(nnn),      /* call nnn */
            (0x3, _,   _,   _  ) => self.op_3xnn(x, nn),    /* if vx != nn then */
            (0x4, _,   _,   _  ) => self.op_4xnn(x, nn),    /* if vx == nn then */
            (0x5, _,   _,   _  ) => self.op_5xy0(x, y),     /* if vx != vy then */
            (0x6, _,   _,   _  ) => self.op_6xnn(x, nn),    /* vx := nn */
            (0x7, _,   _,   _  ) => self.op_7xnn(x, nn),    /* vx += nn */
            (0x8, _,   _,   0x0) => self.op_8xy0(x, y),     /* vx := vy */
            (0x8, _,   _,   0x1) => self.op_8xy1(x, y),     /* vx |= vy */
            (0x8, _,   _,   0x2) => self.op_8xy2(x, y),     /* vx &= vy */
            (0x8, _,   _,   0x3) => self.op_8xy3(x, y),     /* vx ^= vy */
            (0x8, _,   _,   0x4) => self.op_8xy4(x, y),     /* vx += vy */
            (0x8, _,   _,   0x5) => self.op_8xy5(x, y),     /* vx -= vy */
            (0x8, _,   _,   0x6) => self.op_8xy6(x, y),     /* vx >>= vy */
            (0x8, _,   _,   0x7) => self.op_8xy7(x, y),     /* vx = -vy */
            (0x8, _,   _,   0xe) => self.op_8xye(x, y),     /* vx <<= vy */
            (0x9, _,   _,   0x0) => self.op_9xy0(x, y),     /* if vx == vy then */
            (0xa, _,   _,   _  ) => self.op_annn(nnn),      /* i := nnn */
            (0xb, _,   _,   _  ) => self.op_bnnn(nnn),      /* jump nnn + v0 */
            (0xc, _,   _,   _  ) => self.op_cxnn(x, nn),    /* vx := random(0, 255) & nn */
            (0xd, _,   _,   _  ) => self.op_dxyn(x, y, n),  /* sprite vx vy n */
            (0xe, _,   0x9, 0xe) => self.op_ex9e(x),        /* is a key not pressed? */
            (0xe, _,   0xa, 0x1) => self.op_exa1(x),        /* is a key pressed? */
            (0xf, _,   0x0, 0x7) => self.op_fx07(x),        /* vx := delay */
            (0xf, _,   0x0, 0xa) => self.op_fx0a(x),        /* vx := key */
            (0xf, _,   0x1, 0x5) => self.op_fx15(x),        /* delay := vx */
            (0xf, _,   0x1, 0x8) => self.op_fx18(x),        /* buzzer := vx */
            (0xf, _,   0x1, 0xe) => self.op_fx1e(x),        /* buzzer := vx */
            (0xf, _,   0x2, 0x9) => self.op_fx29(x),        /* i := hex(vx) */
            (0xf, _,   0x3, 0x3) => self.op_fx33(x),        /* bcd vx */
            (0xf, _,   0x5, 0x5) => self.op_fx55(x),        /* save vx */
            (0xf, _,   0x6, 0x5) => self.op_fx65(x),        /* load vx */
            (_,   _,   _,   _  ) => {
                return Err(Error::new(
                    ErrorKind::InvalidData, format!("unexpected opcode {:#02x}!", op)
                ));
            },
        };

        match cycle_count {
            PC::I    => Ok(self.pc = self.pc.wrapping_add(1*OP_LEN)),
            PC::C    => Ok(self.pc = self.pc.wrapping_add(2*OP_LEN)),
            PC::J(a) => Ok(self.pc = a),
        }
    }

    // clear
    fn op_00e0(&mut self) -> PC {
        //println!("clear");
        for jj in 0..V_HEIGHT {
            for ii in 0..V_WIDTH {
                self.vram[ii][jj] = 0;
            }
        }
        PC::I
    }

    // return
    fn op_00ee(&mut self) -> PC {
        //println!("return");
        self.pc -= 1;
        PC::J(self.s[self.sp])
    }

    // pc = nnn
    fn op_1nnn(&mut self, nnn: usize) -> PC {
        //println!("jmp {:#05x}", nnn);
        PC::J(nnn)
    }

    // call nnn
    fn op_2nnn(&mut self, nnn: usize) -> PC {
        //println!("call {:#05x}", nnn);
        self.s[self.sp] = self.pc + OP_LEN;
        self.sp += 1;
        PC::J(nnn)
    }

    // if v[x] != nn then
    fn op_3xnn(&mut self, x: usize, nn: u8) -> PC {
        //println!("if v[{}] != {} then", x, nn);
        PC::cond(self.v[x] != nn)
    }

    // if v[x] == nn then
    fn op_4xnn(&mut self, x: usize, nn: u8) -> PC {
        //println!("if v[{}] == {} then", x, nn);
        PC::cond(self.v[x] == nn)
    }

    // if v[x] != v[y] then
    fn op_5xy0(&mut self, x: usize, y: usize) -> PC {
        //println!("if v[{}] != v[{}] then", x, y);
        PC::cond(self.v[x] != self.v[y])
    }

    // v[x] = nn
    fn op_6xnn(&mut self, x: usize, nn: u8) -> PC {
        //println!("v[{}] = {}", x, nn);
        self.v[x] = nn;
        PC::I
    }

    // v[x] += nn
    fn op_7xnn(&mut self, x: usize, nn: u8) -> PC {
        //println!("v[{}] += {}", x, nn);
        let res = self.v[x].wrapping_add(nn);
        self.v[x] = res as u8;
        PC::I
    }

    // v[x] = v[y]
    fn op_8xy0(&mut self, x: usize, y: usize) -> PC {
        //println!("v[{}] = v[{}]", x, y);
        self.v[x] = self.v[y];
        PC::I
    }

    // v[x] |= v[y]
    fn op_8xy1(&mut self, x: usize, y: usize) -> PC {
        //println!("v[{}] |= v[{}]", x, y);
        self.v[x] |= self.v[y];
        PC::I
    }

    // v[x] &= v[y]
    fn op_8xy2(&mut self, x: usize, y: usize) -> PC {
        //println!("v[{}] &= v[{}]", x, y);
        self.v[x] &= self.v[y];
        PC::I
    }

    // v[x] ^= v[y]
    fn op_8xy3(&mut self, x: usize, y: usize) -> PC {
        //println!("v[{}] ^= v[{}]", x, y);
        self.v[x] ^= self.v[y];
        PC::I
    }

    // v[x] += v[y]
    fn op_8xy4(&mut self, x: usize, y: usize) -> PC {
        //println!("v[{}] ++ v[{}]", x, y);
        let res = self.v[x].wrapping_add(self.v[y]);
        // flag set on addition overflow
        self.v[0xf] = if self.v[x] ^ res == 0b1000_0000 { 1 } else { 0 };
        self.v[x] = res as u8;
        PC::I
    }

    // v[x] -= v[y]
    fn op_8xy5(&mut self, x: usize, y: usize) -> PC {
        //println!("v[{}] -= v[{}]", x, y);
        let res = self.v[x].wrapping_sub(self.v[y]);
        // flag set on subtraction underflow
        self.v[0xf] = if self.v[x] ^ res == 0b1000_0000 { 1 } else { 0 };
        self.v[x] = res as u8;
        PC::I
    }

    // v[x] >>= v[y]
    fn op_8xy6(&mut self, x: usize, y: usize) -> PC {
        //println!("v[{}] >>= v[{}]", x, y);
        let res = (self.v[x] as u16) >> (self.v[y] as u16);
        self.v[x] = res as u8; // does this underflow?
        // flag set old LSB
        self.v[0xf] = if self.v[x] & 0b0000_0001 == 0b0000_0001 { 1 } else { 0 };
        PC::I
    }

    // v[x] = (v[y] - v[x])
    fn op_8xy7(&mut self, x: usize, y: usize) -> PC {
        //println!("v[{}] =- v[{}]", x, y);
        let res = self.v[y].wrapping_sub(self.v[x]);
        // flag set on subtraction underflow
        self.v[0xf] = if self.v[x] ^ res == 0b1000_0000 { 1 } else { 0 };
        self.v[x] = res as u8;
        return PC::I
    }

    // v[x] <<= v[y]
    fn op_8xye(&mut self, x: usize, y: usize) -> PC {
        //println!("v[{}] <<= v[{}]", x, y);
        let res = (self.v[x] as u16) << (self.v[y] as u16);
        // flag set old MSB
        self.v[0xf] = if self.v[x] & 0b1000_0000 == 0b1000_0000 { 1 } else { 0 };
        self.v[x] = res as u8; // does this overflow?
        PC::I
    }

    // if v[x] == v[y] then
    fn op_9xy0(&mut self, x: usize, y: usize) -> PC {
        //println!("if v[{}] == v[{}] then", x, y);
        PC::cond(self.v[x] == self.v[y])
    }

    // i = nnn
    fn op_annn(&mut self, nnn: usize) -> PC {
        //println!("i = {:#05x}", nnn);
        self.i = nnn;
        PC::I
    }

    // pc = nnn + v[0]
    fn op_bnnn(&mut self, nnn: usize) -> PC {
        //println!("pc = {:#05x} + v[0]", nnn);
        PC::J(nnn + self.v[0] as usize)
    }

    // v[x] = rand(255) & nn
    fn op_cxnn(&mut self, x: usize, nn: u8) -> PC {
        panic!("TODO: random");
        PC::I
    }

    // sprite v[x] v[y] n
    fn op_dxyn(&mut self, x: usize, y: usize, n: usize) -> PC {
        self.v[0xf] = 0; // change flag
        for byte_number in 0..n {
            let sprite_y = (self.v[y] as usize + byte_number) % V_HEIGHT;
            for bit in 0..8 {
                let sprite_x = (self.v[x] as usize + bit) % V_WIDTH;
                let c = (self.ram[self.i + byte_number] >> (7 - bit)) & 0x1;
                self.v[0xf] |= c & self.vram[sprite_x][sprite_y]; // flag set if bit cleared
                self.vram[sprite_x][sprite_y] ^= c;
            }
        }
        PC::I
    }

    // if v[x] != key then
    fn op_ex9e(&mut self, x: usize) -> PC {
        panic!("TODO: key");
        PC::I
    }

    // if v[s] == key then
    fn op_exa1(&mut self, x: usize) -> PC {
        panic!("TODO: key");
        PC::I
    }

    // v[x] = delay
    fn op_fx07(&mut self, x: usize) -> PC {
        panic!("TODO: delay");
        PC::I
    }

    // v[x] = key
    fn op_fx0a(&mut self, x: usize) -> PC {
        panic!("TODO: keypresses");
        PC::I
    }

    // delay = v[x]
    fn op_fx15(&mut self, x: usize) -> PC {
        panic!("TODO: delay");
        PC::I
    }

    // sound timer = v[x]
    fn op_fx18(&mut self, x: usize) -> PC {
        panic!("TODO: sound timer");
        PC::I
    }

    // i += v[x]
    fn op_fx1e(&mut self, x: usize) -> PC {
        //println!("i += v[{}]", x);
        let res = self.v[x].wrapping_add(self.i as u8);
        self.i = res as usize;
        PC::I
    }

    // i = sprite(v[x])
    fn op_fx29(&mut self, x: usize) -> PC {
        panic!("TODO: sprites");
        PC::I
    }

    // bcd(v[x])
    fn op_fx33(&mut self, x: usize) -> PC {
        panic!("TODO: BCD");
        PC::I
    }

    // store v
    fn op_fx55(&mut self, x: usize) -> PC {
        //println!("store v");
        for ii in 0..REG_COUNT {
            self.ram[self.i + ii] = self.v[ii];
        }
        PC::I
    }

    // load v
    fn op_fx65(&mut self, x: usize) -> PC {
        //println!("load v");
        for ii in 0..REG_COUNT {
            self.v[ii] = self.ram[self.i + ii];
        }
        PC::I
    }
}

#[cfg(test)]
#[path = "cpu_test.rs"]
mod cpu_test;

/* vim: set fdm=marker : */
