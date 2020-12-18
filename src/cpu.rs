const RAM_SIZE: usize = 2;

pub struct Cpu {
    i: usize,
    pc: usize,
    sp: usize,
    ram: [u8; RAM_SIZE],
}

impl Cpu {
    pub fn new() -> Self {
        let mut ram = [0; RAM_SIZE];
        Cpu {
            i: 0,
            pc: 0x200,
            sp: 0,
            ram: ram,
        }
    }

    fn next_opcode(&self) -> u16 {
        (self.ram[self.pc] as u16) << 8 | (self.ram[self.pc + 1] as u16)
    }

    // one instruction cycle (variable machine cycle)
    fn icycle(&mut self, op: u16) {
        // super handy specification:
        // http://johnearnest.github.io/Octo/docs/chip8ref.pdf
        let nibs = (
            (op & 0xf000) >> 12 as u8,
            (op & 0x0f00) >> 8  as u8,
            (op & 0x00f0) >> 4  as u8,
            (op & 0x000f) >> 0  as u8,
        );
        let nnn = (op & 0x0fff) as usize; // address
        let kk  = (op & 0x00ff) as u8;    // 8-bit constant
        let x   = nibs.1 as usize;        // 4-bit constant
        let y   = nibs.2 as usize;        // 4-bit constant
        let n   = nibs.3 as usize;        // opcode argument

        let cycle_count = match nibbles {
            (0x0, 0x0, 0xe, 0x0) => /* clear */
                (),
            (0x0, 0x0, 0xe, 0xe) => /* return */
                (),
            (0x1, _,   _,   _  ) => /* jump nnn */
                (),
            (0x2, _,   _,   _  ) => /* call nnn */
                (),
            (0x3, _,   _,   _) => /* if vx != nn then */
                (),
            (0x4, _,   _,   _) => /* if vx == nn then */
                (),
            (0x5, _,   _,   _) => /* if vx != vy then */
                (),
            (0x6, _,   _,   _) => /* vx := nn */
                (),
            (0x7, _,   _,   _) => /*  vx += nn */
                (),
            (0x8, _,   _, 0x0) => /* vx := vy */
                (),
            (0x8, _,   _, 0x1) => /* vx |= vy */
                (),
            (0x8, _,   _, 0x2) => /* vx &= vy */
                (),
            (0x8, _,   _, 0x3) => /* vx ^= vy */
                (),
            (0x8, _,   _, 0x4) => /* vx += vy */
                (),
            (0x8, _,   _, 0x5) => /* vx -= vy */
                (),
            (0x8, _,   _, 0x6) => /* vx >>= vy */
                (),
            (0x8, _,   _, 0x7) => /* vx = -vy */
                (),
            (0x8, _,   _, 0xe) => /* vx <<= vy */
                (),
            (0x9, _,   _, 0x0) => /* if vx == vy then */
                (),
            (0xa, _,   _,   _) => /* i := nnn */
                (),
            (0xb, _,   _,   _) => /* jump nnn + v0 */
                (),
            (0xc, _,   _,   _) => /* vx := random(0, 255) & nn */
                (),
            (0xd, _,   _,   _) => /* sprite vx vy n */
                (),
            (0xe, _, 0x9, 0xe) => /* is a key not pressed? */
                (),
            (0xe,   _, 0xa, 0xe) => /* is a key pressed? */
                (),
            //(0xf,   _, 0x0, 0x7)
        };
    }
}

#[cfg(test)]
#[path = "cpu_test.rs"]
mod cpu_test;
