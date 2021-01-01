extern crate rand;

use std::io::{Error, ErrorKind};

use crate::{
    screen::V_WIDTH,
    screen::V_HEIGHT,
    rom::ROM,
};

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
    kreg:       usize,                      // keypad register

    /* timers */
    delay:      u8,                         // delay timer
    sound:      u8,                         // sound timer

    /* emulator state */
    verbose:    bool,                       // print debug information
    keyb:       [bool; 16],                 // key reader
    kwait:      bool,                       // waiting for key press
    prog:       Option<ROM>,                // store program (restore from reset)
}
impl Cpu {
    pub fn new(prog: Option<ROM>, v: bool) -> Result<Self, Error> {
        let ram = Cpu::ram_init(&prog)?;
        Ok(Cpu {
            ram:        ram,
            vram:       [[0; V_HEIGHT]; V_WIDTH],
            sp:         0x0,
            s:          [0; STACK_SIZE],
            v:          [0; REG_COUNT],
            i:          0x000,
            pc:         PC_BASE,
            kreg:       0,
            delay:      0,
            sound:      0,
            verbose:    v,
            keyb:       [false; 16],
            kwait:      false,
            prog:       prog,
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

    // insert an opcode at the current PC
    #[allow(dead_code)]
    fn opcode_init(&mut self, op: u16) {
        self.ram[self.pc]     = ((op & 0xff00) >> 8) as u8;
        self.ram[self.pc + 1] = ((op & 0x00ff) >> 0) as u8;
    }

    // TODO: reset in keyhandling
    fn reset(&mut self) -> Result<(), Error> {
        let ram = Cpu::ram_init(&self.prog)?;
        self.ram    = ram;
        self.vram   = [[0; V_HEIGHT]; V_WIDTH];
        self.sp     = 0x0;
        self.s      = [0; STACK_SIZE];
        self.v      = [0; REG_COUNT];
        self.i      = 0x000;
        self.pc     = PC_BASE;
        self.kreg   = 0;
        self.delay  = 0;
        self.sound  = 0;
        self.keyb   = [false; 16];
        self.kwait  = false;
        Ok(())
    }

    fn fetch(&self) -> u16 {
        (self.ram[self.pc] as u16) << 8 | (self.ram[self.pc + 1] as u16)
    }

    // one machine cycle
    pub fn mcycle(&mut self, keypad: [bool; 16]) -> Result<(), Error> {
        self.keyb = keypad; // read the state of the keypad

        // halted state, wait for keypress
        if self.kwait {
            for ii in 0..self.keyb.len() {
                if keypad[ii] {
                    self.kwait = false;
                    self.v[self.kreg] = ii as u8;
                }
            }
        } else {
            // decrement timers
            if self.delay > 0 {
                print!("\0x07"); // BEEP
                self.delay -= 1;
            }
            if self.sound > 0 {
                self.sound -= 1;
            }
            self.icycle()?;
        }
        Ok(())
    }

    fn debug_print(&self, s: &str) {
        if self.verbose { println!("{}", s); }
    }

    // one instruction cycle (variable machine cycle)
    // super handy specification:
    // http://johnearnest.github.io/Octo/docs/chip8ref.pdf
    fn icycle(&mut self) -> Result<(), Error> {
        let op = self.fetch();
        if self.verbose { println!("fetch [{:#08x}] {:#06x}", self.pc, op); }
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
            (0x0, 0x0, 0xe, 0x0) => { self.debug_print("clear");      self.op_00e0()        },
            (0x0, 0x0, 0xe, 0xe) => { self.debug_print("return");     self.op_00ee()        },
            (0x1, _,   _,   _  ) => { self.debug_print("jmp nnn");    self.op_1nnn(nnn)     },
            (0x2, _,   _,   _  ) => { self.debug_print("call nnn");   self.op_2nnn(nnn)     },
            (0x3, _,   _,   _  ) => { self.debug_print("if vx!=nn");  self.op_3xnn(x, nn)   },
            (0x4, _,   _,   _  ) => { self.debug_print("if vx==nn");  self.op_4xnn(x, nn)   },
            (0x5, _,   _,   _  ) => { self.debug_print("if vx!=vy");  self.op_5xy0(x, y)    },
            (0x6, _,   _,   _  ) => { self.debug_print("vx = nn");    self.op_6xnn(x, nn)   },
            (0x7, _,   _,   _  ) => { self.debug_print("vx += nn");   self.op_7xnn(x, nn)   },
            (0x8, _,   _,   0x0) => { self.debug_print("vx = vy");    self.op_8xy0(x, y)    },
            (0x8, _,   _,   0x1) => { self.debug_print("vx |= vy");   self.op_8xy1(x, y)    },
            (0x8, _,   _,   0x2) => { self.debug_print("vx &= vy");   self.op_8xy2(x, y)    },
            (0x8, _,   _,   0x3) => { self.debug_print("vx ^= vy");   self.op_8xy3(x, y)    },
            (0x8, _,   _,   0x4) => { self.debug_print("vx += vy");   self.op_8xy4(x, y)    },
            (0x8, _,   _,   0x5) => { self.debug_print("vx -= vy");   self.op_8xy5(x, y)    },
            (0x8, _,   _,   0x6) => { self.debug_print("vx >>= vy");  self.op_8xy6(x, y)    },
            (0x8, _,   _,   0x7) => { self.debug_print("vx = -vy");   self.op_8xy7(x, y)    },
            (0x8, _,   _,   0xe) => { self.debug_print("vx <<= vy");  self.op_8xye(x, y)    },
            (0x9, _,   _,   0x0) => { self.debug_print("vx == vy");   self.op_9xy0(x, y)    },
            (0xa, _,   _,   _  ) => { self.debug_print("i = nnn");    self.op_annn(nnn)     },
            (0xb, _,   _,   _  ) => { self.debug_print("jmp nnn+v0"); self.op_bnnn(nnn)     },
            (0xc, _,   _,   _  ) => { self.debug_print("vx = rand");  self.op_cxnn(x, nn)   },
            (0xd, _,   _,   _  ) => { self.debug_print("sp vx vy n"); self.op_dxyn(x, y, n) },
            (0xe, _,   0x9, 0xe) => { self.debug_print("!key");       self.op_ex9e(x)       },
            (0xe, _,   0xa, 0x1) => { self.debug_print("key");        self.op_exa1(x)       },
            (0xf, _,   0x0, 0x7) => { self.debug_print("vx = delay"); self.op_fx07(x)       },
            (0xf, _,   0x0, 0xa) => { self.debug_print("vx = key");   self.op_fx0a(x)       },
            (0xf, _,   0x1, 0x5) => { self.debug_print("delay = vx"); self.op_fx15(x)       },
            (0xf, _,   0x1, 0x8) => { self.debug_print("buzz = vx");  self.op_fx18(x)       },
            (0xf, _,   0x1, 0xe) => { self.debug_print("i += vx");    self.op_fx1e(x)       },
            (0xf, _,   0x2, 0x9) => { self.debug_print("i = hx(vx)"); self.op_fx29(x)       },
            (0xf, _,   0x3, 0x3) => { self.debug_print("bcd vx");     self.op_fx33(x)       },
            (0xf, _,   0x5, 0x5) => { self.debug_print("save vx");    self.op_fx55(x)       },
            (0xf, _,   0x6, 0x5) => { self.debug_print("load vx");    self.op_fx65(x)       },
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
        for jj in 0..V_HEIGHT {
            for ii in 0..V_WIDTH {
                self.vram[ii][jj] = 0;
            }
        }
        PC::I
    }

    // return
    fn op_00ee(&mut self) -> PC {
        let rl = self.s[self.sp];
        self.pc -= 1;
        if self.verbose { println!("stack: {:#x?}", self.s); }
        PC::J(rl)
    }

    // pc = nnn
    fn op_1nnn(&mut self, nnn: usize) -> PC {
        PC::J(nnn)
    }

    // call nnn
    fn op_2nnn(&mut self, nnn: usize) -> PC {
        self.sp += 1;
        self.s[self.sp] = self.pc + OP_LEN;
        if self.verbose { println!("stack: {:#x?}", self.s); }
        PC::J(nnn)
    }

    // if v[x] != nn then
    fn op_3xnn(&mut self, x: usize, nn: u8) -> PC {
        PC::cond(self.v[x] != nn)
    }

    // if v[x] == nn then
    fn op_4xnn(&mut self, x: usize, nn: u8) -> PC {
        PC::cond(self.v[x] == nn)
    }

    // if v[x] != v[y] then
    fn op_5xy0(&mut self, x: usize, y: usize) -> PC {
        PC::cond(self.v[x] != self.v[y])
    }

    // v[x] = nn
    fn op_6xnn(&mut self, x: usize, nn: u8) -> PC {
        self.v[x] = nn;
        PC::I
    }

    // v[x] += nn
    fn op_7xnn(&mut self, x: usize, nn: u8) -> PC {
        let res = self.v[x].wrapping_add(nn);
        self.v[x] = res as u8;
        PC::I
    }

    // v[x] = v[y]
    fn op_8xy0(&mut self, x: usize, y: usize) -> PC {
        self.v[x] = self.v[y];
        PC::I
    }

    // v[x] |= v[y]
    fn op_8xy1(&mut self, x: usize, y: usize) -> PC {
        self.v[x] |= self.v[y];
        PC::I
    }

    // v[x] &= v[y]
    fn op_8xy2(&mut self, x: usize, y: usize) -> PC {
        self.v[x] &= self.v[y];
        PC::I
    }

    // v[x] ^= v[y]
    fn op_8xy3(&mut self, x: usize, y: usize) -> PC {
        self.v[x] ^= self.v[y];
        PC::I
    }

    // v[x] += v[y]
    fn op_8xy4(&mut self, x: usize, y: usize) -> PC {
        let res = self.v[x].wrapping_add(self.v[y]);
        // flag set on addition overflow
        self.v[0xf] = if self.v[x] ^ res == 0b1000_0000 { 1 } else { 0 };
        self.v[x] = res as u8;
        PC::I
    }

    // v[x] -= v[y]
    fn op_8xy5(&mut self, x: usize, y: usize) -> PC {
        let res = self.v[x].wrapping_sub(self.v[y]);
        // flag set on subtraction underflow
        self.v[0xf] = if self.v[x] ^ res == 0b1000_0000 { 1 } else { 0 };
        self.v[x] = res as u8;
        PC::I
    }

    // v[x] >>= v[y]
    fn op_8xy6(&mut self, x: usize, _y: usize) -> PC {
        let res = (self.v[x] as u16) / 2;
        self.v[x] = res as u8; // does this underflow?
        // flag set old LSB
        self.v[0xf] = if self.v[x] & 0b0000_0001 == 0b0000_0001 { 1 } else { 0 };
        PC::I
    }

    // v[x] = (v[y] - v[x])
    fn op_8xy7(&mut self, x: usize, y: usize) -> PC {
        let res = self.v[y].wrapping_sub(self.v[x]);
        // flag set on subtraction underflow
        self.v[0xf] = if self.v[x] ^ res == 0b1000_0000 { 1 } else { 0 };
        self.v[x] = res as u8;
        return PC::I
    }

    // v[x] <<= v[y]
    fn op_8xye(&mut self, x: usize, _y: usize) -> PC {
        let res = (self.v[x] as u16) * 2;
        // flag set old MSB
        self.v[0xf] = if self.v[x] & 0b1000_0000 == 0b1000_0000 { 1 } else { 0 };
        self.v[x] = res as u8; // does this overflow?
        PC::I
    }

    // if v[x] == v[y] then
    fn op_9xy0(&mut self, x: usize, y: usize) -> PC {
        PC::cond(self.v[x] == self.v[y])
    }

    // i = nnn
    fn op_annn(&mut self, nnn: usize) -> PC {
        self.i = nnn;
        PC::I
    }

    // pc = nnn + v[0]
    fn op_bnnn(&mut self, nnn: usize) -> PC {
        PC::J(nnn + self.v[0] as usize)
    }

    // v[x] = rand(255) & nn
    fn op_cxnn(&mut self, x: usize, nn: u8) -> PC {
        // two approaches:
        // 0: r = rand() % nn
        // 1: r = (rand() % 255) & nn
        // both produce r in [0, nn], but distrubtion of (1) follows instruction set
        self.v[x] = (rand::random::<u8>() % 255) & nn;
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

    // if v[x] == key then
    fn op_ex9e(&mut self, x: usize) -> PC {
        PC::cond(self.keyb[self.v[x] as usize])
    }

    // if v[x] != key then
    fn op_exa1(&mut self, x: usize) -> PC {
        PC::cond(!self.keyb[self.v[x] as usize])
    }

    // v[x] = delay
    fn op_fx07(&mut self, x: usize) -> PC {
        self.v[x] = self.delay;
        PC::I
    }

    // v[x] = key
    fn op_fx0a(&mut self, x: usize) -> PC {
        self.kreg = x;
        self.kwait = true;
        PC::I
    }

    // delay = v[x]
    fn op_fx15(&mut self, x: usize) -> PC {
        self.delay = self.v[x];
        PC::I
    }

    // sound timer = v[x]
    fn op_fx18(&mut self, x: usize) -> PC {
        self.sound = self.v[x];
        PC::I
    }

    // i += v[x]
    fn op_fx1e(&mut self, x: usize) -> PC {
        let res = self.v[x].wrapping_add(self.i as u8);
        self.i = res as usize;
        PC::I
    }

    // i = sprite(v[x])
    fn op_fx29(&mut self, x: usize) -> PC {
        // location of char x in RAM
        // font starts at ram[0], and each is 5 bytes
        self.i = (self.v[x] as usize) * 5;
        PC::I
    }

    // bcd(v[x])
    fn op_fx33(&mut self, x: usize) -> PC {
        self.ram[self.i    ] = self.v[x] / 100;             // hundreds place
        self.ram[self.i + 1] = (self.v[x] % 100) / 10;      // tens place
        self.ram[self.i + 2] = self.v[x] % 10;              // ones place
        PC::I
    }

    // store v
    fn op_fx55(&mut self, x: usize) -> PC {
        for ii in 0..x+1 {
            self.ram[self.i + ii] = self.v[ii];
        }
        PC::I
    }

    // load v
    fn op_fx65(&mut self, x: usize) -> PC {
        for ii in 0..x+1 {
            self.v[ii] = self.ram[self.i + ii];
        }
        PC::I
    }
}

#[cfg(test)]
#[path = "test/cpu_test.rs"]
mod cpu_test;

/* vim: set fdm=marker : */
