const OP_LEN: usize = 2; // number of words in an opcode
enum PC {
    I,          // pc += 1*OP_LEN
    C,          // pc += 2*OP_LEN (condition, skip jump instrution)
    J(usize),   // pc += arg
}
impl PC {
    fn cond(q: bool) -> PC {
        if q { PC::C }
        else { PC::I }
    }
}

const RAM_SIZE:  usize = 4096;
const REG_COUNT: usize = 16;
pub struct Cpu {
    i:      usize,              // index register
    pc:     usize,              // program counter
    sp:     usize,              // stack pointer
    ram:    [u8; RAM_SIZE],     // RAM tape
    v:      [u8; REG_COUNT],    // registers
}
impl Cpu {
    pub fn new() -> Self {
        //let mut ram = [0; RAM_SIZE];
        Cpu {
            i:      0,
            pc:     0x200,
            sp:     0,
            ram:    [0; RAM_SIZE],
            v:      [0; REG_COUNT],
        }
    }

    // insert an opcode at the current PC
    fn opcode_init(&mut self, op: u16) {
        self.ram[self.pc]     = ((op & 0xff00) >> 8) as u8;
        self.ram[self.pc + 1] = ((op & 0x00ff) >> 0) as u8;
    }

    fn next_opcode(&self) -> u16 {
        (self.ram[self.pc] as u16) << 8 | (self.ram[self.pc + 1] as u16)
    }

    // one instruction cycle (variable machine cycle)
    // super handy specification:
    // http://johnearnest.github.io/Octo/docs/chip8ref.pdf
    fn icycle(&mut self) {
        let op = self.next_opcode();
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
            (0x0, 0x0, 0xe, 0x0) => /* clear */
                self.op_00e0(),
            (0x0, 0x0, 0xe, 0xe) => /* return */
                self.op_00ee(),
            (0x1, _,   _,   _  ) => /* jump nnn */
                self.op_1nnn(nnn),
            (0x2, _,   _,   _  ) => /* call nnn */
                self.op_2nnn(nnn),
            (0x3, _,   _,   _  ) => /* if vx != nn then */
                self.op_3xnn(x, nn),
            (0x4, _,   _,   _  ) => /* if vx == nn then */
                self.op_4xnn(x, nn),
            (0x5, _,   _,   _  ) => /* if vx != vy then */
                self.op_5xy0(x, y),
            (0x6, _,   _,   _  ) => /* vx := nn */
                self.op_6xnn(x, nn),
            (0x7, _,   _,   _  ) => /*  vx += nn */
                self.op_7xnn(x, nn),
            (0x8, _,   _,   0x0) => /* vx := vy */
                self.op_8xy0(x, y),
            (0x8, _,   _,   0x1) => /* vx |= vy */
                self.op_8xy1(x, y),
            (0x8, _,   _,   0x2) => /* vx &= vy */
                self.op_8xy2(x, y),
            (0x8, _,   _,   0x3) => /* vx ^= vy */
                self.op_8xy3(x, y),
            (0x8, _,   _,   0x4) => /* vx += vy */
                self.op_8xy4(x, y),
            (0x8, _,   _,   0x5) => /* vx -= vy */
                self.op_8xy5(x, y),
            (0x8, _,   _,   0x6) => /* vx >>= vy */
                self.op_8xy6(x, y),
            (0x8, _,   _,   0x7) => /* vx = -vy */
                self.op_8xy7(x, y),
            (0x8, _,   _,   0xe) => /* vx <<= vy */
                self.op_8xye(x, y),
            (0x9, _,   _,   0x0) => /* if vx == vy then */
                self.op_9xy0(x, y),
            (0xa, _,   _,   _  ) => /* i := nnn */
                self.op_annn(nnn),
            (0xb, _,   _,   _  ) => /* jump nnn + v0 */
                self.op_bnnn(nnn),
            (0xc, _,   _,   _  ) => /* vx := random(0, 255) & nn */
                self.op_cxnn(x, nn),
            (0xd, _,   _,   _  ) => /* sprite vx vy n */
                self.op_dxyn(x, y, n),
            (0xe, _,   0x9, 0xe) => /* is a key not pressed? */
                self.op_ex9e(x),
            (0xe, _,   0xa, 0xe) => /* is a key pressed? */
                self.op_ex9e(x),
            (0xf, _,   0x0, 0x7) => /* vx := delay */
                self.op_fx07(x),
            (0xf, _,   0x0, 0xa) => /* vx := key */
                self.op_fx0a(x),
            (0xf, _,   0x1, 0x5) => /* delay := vx */
                self.op_fx15(x),
            (0xf, _,   0x1, 0x8) => /* buzzer := vx */
                self.op_fx18(x),
            (0xf, _,   0x2, 0x9) => /* i := hex(vx) */
                self.op_fx29(x),
            (0xf, _,   0x3, 0x3) => /* bcd vx */
                self.op_fx33(x),
            (0xf, _,   0x5, 0x5) => /* save vx */
                self.op_fx55(x),
            (0xf, _,   0x6, 0x5) => /* load vx */
                self.op_fx55(x),
            (_,   _,   _,   _  ) => {
                panic!("unexpected opcode!");
            },
        };

        match cycle_count {
            PC::I    => self.pc = self.pc.wrapping_add(1*OP_LEN),
            PC::C    => self.pc = self.pc.wrapping_add(2*OP_LEN),
            PC::J(a) => self.pc =  a,
        }
    }

    fn op_00e0(&mut self) -> PC {
        // TODO
        PC::I
    }

    fn op_00ee(&mut self) -> PC {
        PC::I
    }

    fn op_1nnn(&mut self, nnn: usize) -> PC {
        PC::I
    }

    fn op_2nnn(&mut self, nnn: usize) -> PC {
        PC::I
    }

    fn op_3xnn(&mut self, x: usize, nn: u8) -> PC {
        PC::I
    }

    fn op_4xnn(&mut self, x: usize, nn: u8) -> PC {
        PC::I
    }

    fn op_5xy0(&mut self, x: usize, y: usize) -> PC {
        PC::I
    }

    // v[x] = nn
    fn op_6xnn(&mut self, x: usize, nn: u8) -> PC {
        self.v[x] = nn; PC::I
    }

    // v[x] += nn
    fn op_7xnn(&mut self, x: usize, nn: u8) -> PC {
        let res = self.v[x].wrapping_add(nn);
        self.v[x] = res as u8;
        PC::I
    }

    // v[x] = v[y]
    fn op_8xy0(&mut self, x: usize, y: usize) -> PC {
        self.v[x] = self.v[y]; PC::I
    }

    // v[x] |= v[y]
    fn op_8xy1(&mut self, x: usize, y: usize) -> PC {
        self.v[x] |= self.v[y]; PC::I
    }

    // v[x] &= v[y]
    fn op_8xy2(&mut self, x: usize, y: usize) -> PC {
        self.v[x] &= self.v[y]; PC::I
    }

    // v[x] ^= v[y]
    fn op_8xy3(&mut self, x: usize, y: usize) -> PC {
        self.v[x] ^= self.v[y]; PC::I
    }

    // v[x] += v[y]
    fn op_8xy4(&mut self, x: usize, y: usize) -> PC {
        let res = self.v[x].wrapping_add(self.v[y]);
        self.v[x] = res as u8;
        PC::I
    }

    // v[x] -= v[y]
    fn op_8xy5(&mut self, x: usize, y: usize) -> PC {
        let res = self.v[x].wrapping_sub(self.v[y]);
        self.v[x] = res as u8;
        return PC::I
    }

    // v[x] >>= v[y]
    fn op_8xy6(&mut self, x: usize, y: usize) -> PC {
        let res = (self.v[x] as u16) >> (self.v[y] as u16);
        self.v[x] = res as u8;
        return PC::I
    }

    // v[x] = (v[y] - v[x])
    fn op_8xy7(&mut self, x: usize, y: usize) -> PC {
        let res = self.v[y].wrapping_sub(self.v[x]);
        self.v[x] = res as u8;
        return PC::I
    }

    // v[x] <<= v[y]
    fn op_8xye(&mut self, x: usize, y: usize) -> PC {
        PC::I
    }

    fn op_9xy0(&mut self, x: usize, y: usize) -> PC {
        PC::I
    }

    fn op_annn(&mut self, nnn: usize) -> PC {
        PC::I
    }

    fn op_bnnn(&mut self, nnn: usize) -> PC {
        PC::I
    }

    fn op_cxnn(&mut self, x: usize, nn: u8) -> PC {
        PC::I
    }

    fn op_dxyn(&mut self, x: usize, y: usize, n: usize) -> PC {
        PC::I
    }

    fn op_ex9e(&mut self, x: usize) -> PC {
        PC::I
    }

    fn op_exa1(&mut self, x: usize) -> PC {
        PC::I
    }

    fn op_fx07(&mut self, x: usize) -> PC {
        PC::I
    }

    fn op_fx0a(&mut self, x: usize) -> PC {
        PC::I
    }

    fn op_fx15(&mut self, x: usize) -> PC {
        PC::I
    }

    fn op_fx18(&mut self, x: usize) -> PC {
        PC::I
    }

    fn op_fx1e(&mut self, x: usize) -> PC {
        PC::I
    }

    fn op_fx29(&mut self, x: usize) -> PC {
        PC::I
    }

    fn op_fx33(&mut self, x: usize) -> PC {
        PC::I
    }

    fn op_fx55(&mut self, x: usize) -> PC {
        PC::I
    }

    fn op_fx65(&mut self, x: usize) -> PC {
        PC::I
    }
}

#[cfg(test)]
#[path = "cpu_test.rs"]
mod cpu_test;
