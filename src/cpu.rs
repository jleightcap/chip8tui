use crate::screen::V_WIDTH;
use crate::screen::V_HEIGHT;

use crate::rom::ROM;

const OP_LEN: usize = 2; // number of words in an opcode
enum PC {
    I,           // pc += 1*OP_LEN (increment)
    C,           // pc += 2*OP_LEN (condition, skip instrution)
    JR(usize),   // pc += arg (relative jump)
    JA(usize),   // pc  = arg (absolute jump)
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
    pub vram:   [[u8; V_WIDTH]; V_HEIGHT],  // video RAM

    /* stack */
    sp:         usize,                      // stack pointer
    s:          [usize; STACK_SIZE],        // stack memory

    /* registers */
    v:          [u8; REG_COUNT],            // general registers
    i:          usize,                      // index register
    pc:         usize,                      // program counter
}
impl Cpu {
    pub fn new() -> Self {
        let mut ram = [0; RAM_SIZE];
        for ii in 0..FONT.len() {
            ram[ii] = FONT[ii]; // memcpy?
        }
        Cpu {
            ram:    ram,
            vram:   [[0; V_WIDTH]; V_HEIGHT],
            sp:     0x0,
            s:      [0; STACK_SIZE],
            i:      0x000,
            pc:     PC_BASE,
            v:      [0; REG_COUNT],
        }
    }

    pub fn prog_init(&mut self, r: &ROM) {
        for ii in 0..r.rom.len() {
            if ii < RAM_SIZE + PC_BASE {
                self.ram[PC_BASE + ii] = r.rom[ii];
            }
            else {
                panic!("loaded program exceeds RAM!");
            }
        }
    }

    // insert an opcode at the current PC
    fn opcode_init(&mut self, op: u16) {
        self.ram[self.pc]     = ((op & 0xff00) >> 8) as u8;
        self.ram[self.pc + 1] = ((op & 0x00ff) >> 0) as u8;
    }

    fn fetch(&self) -> u16 {
        (self.ram[self.pc] as u16) << 8 | (self.ram[self.pc + 1] as u16)
    }

    // one machine cycle
    pub fn mcycle(&mut self) {
        self.icycle();
    }

    // one instruction cycle (variable machine cycle)
    // super handy specification:
    // http://johnearnest.github.io/Octo/docs/chip8ref.pdf
    fn icycle(&mut self) {
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
            (0x0, 0x0, 0xe, 0x0) =>     /* clear */
                self.op_00e0(),
            (0x0, 0x0, 0xe, 0xe) =>     /* return */
                self.op_00ee(),
            (0x1, _,   _,   _  ) =>     /* jump nnn */
                self.op_1nnn(nnn),
            (0x2, _,   _,   _  ) =>     /* call nnn */
                self.op_2nnn(nnn),
            (0x3, _,   _,   _  ) =>     /* if vx != nn then */
                self.op_3xnn(x, nn),
            (0x4, _,   _,   _  ) =>     /* if vx == nn then */
                self.op_4xnn(x, nn),
            (0x5, _,   _,   _  ) =>     /* if vx != vy then */
                self.op_5xy0(x, y),
            (0x6, _,   _,   _  ) =>     /* vx := nn */
                self.op_6xnn(x, nn),
            (0x7, _,   _,   _  ) =>     /*  vx += nn */
                self.op_7xnn(x, nn),
            (0x8, _,   _,   0x0) =>     /* vx := vy */
                self.op_8xy0(x, y),
            (0x8, _,   _,   0x1) =>     /* vx |= vy */
                self.op_8xy1(x, y),
            (0x8, _,   _,   0x2) =>     /* vx &= vy */
                self.op_8xy2(x, y),
            (0x8, _,   _,   0x3) =>     /* vx ^= vy */
                self.op_8xy3(x, y),
            (0x8, _,   _,   0x4) =>     /* vx += vy */
                self.op_8xy4(x, y),
            (0x8, _,   _,   0x5) =>     /* vx -= vy */
                self.op_8xy5(x, y),
            (0x8, _,   _,   0x6) =>     /* vx >>= vy */
                self.op_8xy6(x, y),
            (0x8, _,   _,   0x7) =>     /* vx = -vy */
                self.op_8xy7(x, y),
            (0x8, _,   _,   0xe) =>     /* vx <<= vy */
                self.op_8xye(x, y),
            (0x9, _,   _,   0x0) =>     /* if vx == vy then */
                self.op_9xy0(x, y),
            (0xa, _,   _,   _  ) =>     /* i := nnn */
                self.op_annn(nnn),
            (0xb, _,   _,   _  ) =>     /* jump nnn + v0 */
                self.op_bnnn(nnn),
            (0xc, _,   _,   _  ) =>     /* vx := random(0, 255) & nn */
                self.op_cxnn(x, nn),
            (0xd, _,   _,   _  ) =>     /* sprite vx vy n */
                self.op_dxyn(x, y, n),
            (0xe, _,   0x9, 0xe) =>     /* is a key not pressed? */
                self.op_ex9e(x),
            (0xe, _,   0xa, 0xe) =>     /* is a key pressed? */
                self.op_ex9e(x),
            (0xf, _,   0x0, 0x7) =>     /* vx := delay */
                self.op_fx07(x),
            (0xf, _,   0x0, 0xa) =>     /* vx := key */
                self.op_fx0a(x),
            (0xf, _,   0x1, 0x5) =>     /* delay := vx */
                self.op_fx15(x),
            (0xf, _,   0x1, 0x8) =>     /* buzzer := vx */
                self.op_fx18(x),
            (0xf, _,   0x1, 0xe) =>     /* buzzer := vx */
                self.op_fx1e(x),
            (0xf, _,   0x2, 0x9) =>     /* i := hex(vx) */
                self.op_fx29(x),
            (0xf, _,   0x3, 0x3) =>     /* bcd vx */
                self.op_fx33(x),
            (0xf, _,   0x5, 0x5) =>     /* save vx */
                self.op_fx55(x),
            (0xf, _,   0x6, 0x5) =>     /* load vx */
                self.op_fx55(x),
            (_,   _,   _,   _  ) => {
                panic!(format!("unexpected opcode {:#02x}!", op))
            },
        };

        match cycle_count {
            PC::I     => self.pc = self.pc.wrapping_add(1*OP_LEN),
            PC::C     => self.pc = self.pc.wrapping_add(2*OP_LEN),
            PC::JR(a) => self.pc = self.pc.wrapping_add(a),
            PC::JA(a) => self.pc = a,
        }
    }

    // clear
    fn op_00e0(&mut self) -> PC {
        println!("clear");
        for jj in 0..V_HEIGHT {
            for ii in 0..V_WIDTH {
                self.vram[jj][ii] = 0;
            }
        }
        PC::I
    }

    // return
    fn op_00ee(&mut self) -> PC {
        println!("return");
        self.pc -= 1;
        PC::JA(self.s[self.sp])
    }

    // pc = nnn
    fn op_1nnn(&mut self, nnn: usize) -> PC {
        println!("jmp {:#05x}", nnn);
        PC::JA(nnn)
    }

    // call nnn
    fn op_2nnn(&mut self, nnn: usize) -> PC {
        println!("call {:#05x}", nnn);
        self.s[self.sp] = self.pc + OP_LEN;
        self.sp += 1;
        PC::JA(nnn)
    }

    // if v[x] != nn then
    fn op_3xnn(&mut self, x: usize, nn: u8) -> PC {
        println!("if v[{}] != {} then", x, nn);
        PC::cond(self.v[x] != nn)
    }

    // if v[x] == nn then
    fn op_4xnn(&mut self, x: usize, nn: u8) -> PC {
        println!("if v[{}] == {} then", x, nn);
        PC::cond(self.v[x] == nn)
    }

    // if v[x] != v[y] then
    fn op_5xy0(&mut self, x: usize, y: usize) -> PC {
        println!("if v[{}] != v[{}] then", x, y);
        PC::cond(self.v[x] != self.v[y])
    }

    // v[x] = nn
    fn op_6xnn(&mut self, x: usize, nn: u8) -> PC {
        println!("v[{}] = {}", x, nn);
        self.v[x] = nn;
        PC::I
    }

    // v[x] += nn
    fn op_7xnn(&mut self, x: usize, nn: u8) -> PC {
        println!("v[{}] += {}", x, nn);
        let res = self.v[x].wrapping_add(nn);
        self.v[x] = res as u8;
        PC::I
    }

    // v[x] = v[y]
    fn op_8xy0(&mut self, x: usize, y: usize) -> PC {
        println!("v[{}] = v[{}]", x, y);
        self.v[x] = self.v[y];
        PC::I
    }

    // v[x] |= v[y]
    fn op_8xy1(&mut self, x: usize, y: usize) -> PC {
        println!("v[{}] |= v[{}]", x, y);
        self.v[x] |= self.v[y];
        PC::I
    }

    // v[x] &= v[y]
    fn op_8xy2(&mut self, x: usize, y: usize) -> PC {
        println!("v[{}] &= v[{}]", x, y);
        self.v[x] &= self.v[y];
        PC::I
    }

    // v[x] ^= v[y]
    fn op_8xy3(&mut self, x: usize, y: usize) -> PC {
        println!("v[{}] ^= v[{}]", x, y);
        self.v[x] ^= self.v[y];
        PC::I
    }

    // v[x] += v[y]
    fn op_8xy4(&mut self, x: usize, y: usize) -> PC {
        println!("v[{}] ++ v[{}]", x, y);
        let res = self.v[x].wrapping_add(self.v[y]);
        self.v[x] = res as u8;
        PC::I
    }

    // v[x] -= v[y]
    fn op_8xy5(&mut self, x: usize, y: usize) -> PC {
        println!("v[{}] -= v[{}]", x, y);
        let res = self.v[x].wrapping_sub(self.v[y]);
        self.v[x] = res as u8;
        return PC::I
    }

    // v[x] >>= v[y]
    fn op_8xy6(&mut self, x: usize, y: usize) -> PC {
        println!("v[{}] >>= v[{}]", x, y);
        let res = (self.v[x] as u16) >> (self.v[y] as u16);
        self.v[x] = res as u8; // does this underflow?
        return PC::I
    }

    // v[x] = (v[y] - v[x])
    fn op_8xy7(&mut self, x: usize, y: usize) -> PC {
        println!("v[{}] =- v[{}]", x, y);
        let res = self.v[y].wrapping_sub(self.v[x]);
        self.v[x] = res as u8;
        return PC::I
    }

    // v[x] <<= v[y]
    fn op_8xye(&mut self, x: usize, y: usize) -> PC {
        println!("v[{}] <<= v[{}]", x, y);
        let res = (self.v[x] as u16) << (self.v[y] as u16);
        self.v[x] = res as u8; // does this overflow?
        PC::I
    }

    // if v[x] == v[y] then
    fn op_9xy0(&mut self, x: usize, y: usize) -> PC {
        println!("if v[{}] == v[{}] then", x, y);
        PC::cond(self.v[x] == self.v[y])
    }

    // i = nnn
    fn op_annn(&mut self, nnn: usize) -> PC {
        println!("i = {:#05x}", nnn);
        self.i = nnn;
        PC::I
    }

    // pc = nnn + v[0]
    fn op_bnnn(&mut self, nnn: usize) -> PC {
        println!("pc = {:#05x} + v[0]", nnn);
        PC::JA(nnn + self.v[0] as usize)
    }

    // v[x] = rand(255) & nn
    fn op_cxnn(&mut self, x: usize, nn: u8) -> PC {
        panic!("TODO: random");
        PC::I
    }

    // sprite v[x] v[y] n
    fn op_dxyn(&mut self, x: usize, y: usize, n: usize) -> PC {
        println!("sprite v[{}] v[{}] {}", x, y, n);
        self.v[0xf] = 0;
        for byte in 0..n {
            let ii = (self.v[y] as usize + byte) % V_HEIGHT;
            for bit in 0..8 {
                let jj = (self.v[x] as usize + byte) % V_WIDTH;
                let c = (self.ram[self.i + byte] >> (7 - bit)) & 0x1;
                self.v[0xf] |= c & self.vram[ii][jj]; // flag set if bit cleared
                self.vram[ii][jj] ^= c;
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
        println!("i += v[{}]", x);
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
        println!("store v");
        for ii in 0..REG_COUNT {
            self.ram[self.i + ii] = self.v[ii];
        }
        PC::I
    }

    // load v
    fn op_fx65(&mut self, x: usize) -> PC {
        println!("load v");
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
