use super::*;

// execute a vector of machine code
fn exec_test_prog(p: &Vec<u16>, c: &mut Cpu) {
    for i in p {
        c.opcode_init(*i);
        c.icycle().unwrap();
    }
}

#[test]
fn test_new() {
    let c: Cpu = Cpu::new(None).unwrap();
    // register states
    assert_eq!(c.sp, 0x0);
    assert_eq!(c.pc, 0x200);

    let c: Cpu = Cpu::new(
        Some(ROM::new_prog(&[0xde, 0xad, 0xbe, 0xef]))
    ).unwrap();
    assert_eq!(c.sp, 0x0);
    assert_eq!(c.ram[0x200], 0xde);
    assert_eq!(c.ram[0x201], 0xad);
    assert_eq!(c.ram[0x202], 0xbe);
    assert_eq!(c.ram[0x203], 0xef);
}

#[test]
fn test_cpu_reset() {
    // reset on no program
    let mut c = Cpu::new(None).unwrap();
    c.ram[0x200] = 0x42;
    c.pc = 0xdead;
    c.i = 0xbeef;
    c.reset().unwrap();
    assert_eq!(c.ram[0x200], 0x00);
    assert_eq!(c.pc, 0x200);
    assert_eq!(c.i, 0x00);

    // reset with some program
    let mut c: Cpu = Cpu::new(
        Some(ROM::new_prog(&[0xde, 0xad, 0xbe, 0xef]))
    ).unwrap();
    c.pc = 0x204;
    c.reset().unwrap();
    assert_eq!(c.ram[0x200], 0xde); // the program is maintained
    assert_eq!(c.ram[0x201], 0xad);
    assert_eq!(c.ram[0x202], 0xbe);
    assert_eq!(c.ram[0x203], 0xef);
    assert_eq!(c.pc, 0x200); // but execution state is not
    assert_eq!(c.i, 0x00);
}

/* Opcodes {{{ */
#[test]
fn test_0x00e0() {
    let mut c = Cpu::new(None).unwrap();
    c.vram[3][4] = 1;
    c.vram[31][63] = 2;
    exec_test_prog(&vec![0x00e0], &mut c);
    for jj in 0..V_HEIGHT {
        for ii in 0..V_WIDTH {
            assert_eq!(c.vram[jj][ii], 0);
        }
    }
}

#[test]
fn test_0x00ee() {
    let mut c = Cpu::new(None).unwrap();
    c.s[0] = 0x200;
    exec_test_prog(&vec![0x00ee], &mut c);
    assert_eq!(c.sp, 0x0); // return address poped from stack
    assert_eq!(c.pc, 0x200); // pc at value poped
}

#[test]
fn test_0x2nnn() {
    let mut c = Cpu::new(None).unwrap();
    exec_test_prog(&vec![0x2242], &mut c);
    assert_eq!(c.sp, 0x1); // return address on stack
    assert_eq!(c.s[c.sp - 1], 0x202); // return address after call
    assert_eq!(c.pc, 0x242); // pc at called address
}

#[test]
fn test_0x1nnn() {
    let mut c = Cpu::new(None).unwrap();
    exec_test_prog(&vec![0x1242], &mut c);
    assert_eq!(c.pc, 0x242); // pc = nnn
}

#[test]
fn test_0x3xnn() {
    let mut c = Cpu::new(None).unwrap();
    exec_test_prog(&vec![0x6022, 0x3021], &mut c);
    assert_eq!(c.pc, 0x204); // v[x] != nn, pc + 1*OP_LEN
    let mut c = Cpu::new(None).unwrap();
    exec_test_prog(&vec![0x6021, 0x3021], &mut c);
    assert_eq!(c.pc, 0x206); // v[x] == nn, pc + 2*OP_LEN
}

#[test]
fn test_0x4xnn() {
    let mut c = Cpu::new(None).unwrap();
    exec_test_prog(&vec![0x6022, 0x4022], &mut c);
    assert_eq!(c.pc, 0x204); // v[x] == nn, pc + 1*OP_LEN
    let mut c = Cpu::new(None).unwrap();
    exec_test_prog(&vec![0x6022, 0x4021], &mut c);
    assert_eq!(c.pc, 0x206); // v[x] != nn, pc + 2*OP_LEN
}

#[test]
fn test_0x5xy0() {
    let mut c = Cpu::new(None).unwrap();
    exec_test_prog(&vec![0x6022, 0x6123, 0x5010], &mut c);
    assert_eq!(c.pc, 0x206); // v[x] != v[y], pc + 1*OP_LEN
    let mut c = Cpu::new(None).unwrap();
    exec_test_prog(&vec![0x6022, 0x6122, 0x5010], &mut c);
    assert_eq!(c.pc, 0x208); // v[x] == v[y], pc + 2*OP_LEN
}

#[test]
fn test_0x6xnn() {
    // 0x6xnn => vx := nn
    // v[0] = 0x42
    let mut c = Cpu::new(None).unwrap();
    exec_test_prog(&vec![0x6042], &mut c);
    assert_eq!(c.pc, 0x202);
    assert_eq!(c.v[0], 0x42);
}

#[test]
fn test_0x7xnn() {
    // 0x7xnn => vx += nn
    // v[0] += 1
    let mut c = Cpu::new(None).unwrap();
    exec_test_prog(&vec![0x7001], &mut c);
    assert_eq!(c.pc, 0x202);
    assert_eq!(c.v[0], 1);
}

#[test]
fn test_0x8xn0() {
    let mut c = Cpu::new(None).unwrap();
    exec_test_prog(&vec![0x6042, 0x8100], &mut c);
    assert_eq!(c.pc, 0x204);
    assert_eq!(c.v[0], 0x42);
    assert_eq!(c.v[1], 0x42);
}

#[test]
fn test_0x8xy1() {
    let mut c = Cpu::new(None).unwrap();
    exec_test_prog(&vec![0x6042, 0x61ff, 0x8011], &mut c);
    assert_eq!(c.pc, 0x206);
    assert_eq!(c.v[1], 0xff);
    assert_eq!(c.v[0], 0xff); // 0x42 | 0xff
}

#[test]
fn test_0x8xy2() {
    let mut c = Cpu::new(None).unwrap();
    exec_test_prog(&vec![0x6042, 0x61ff, 0x8012], &mut c);
    assert_eq!(c.pc, 0x206);
    assert_eq!(c.v[1], 0xff);
    assert_eq!(c.v[0], 0x42); // 0x42 & 0xff
}

#[test]
fn test_0x8xy3() {
    let mut c = Cpu::new(None).unwrap();
    exec_test_prog(&vec![0x6042, 0x61ff, 0x8013], &mut c);
    assert_eq!(c.pc, 0x206);
    assert_eq!(c.v[1], 0xff);
    assert_eq!(c.v[0], 0xbd); // 0x42 ^ 0xff
}

#[test]
fn test_0x8xy4() {
    let mut c = Cpu::new(None).unwrap();
    exec_test_prog(&vec![0x6021, 0x6121, 0x8014], &mut c);
    assert_eq!(c.pc, 0x206);
    assert_eq!(c.v[1], 0x21);
    assert_eq!(c.v[0], 0x42); // 0x21 + 0x21
}

#[test]
fn test_0x8xy5() {
    let mut c = Cpu::new(None).unwrap();
    exec_test_prog(&vec![0x6063, 0x6121, 0x8015], &mut c);
    assert_eq!(c.pc, 0x206);
    assert_eq!(c.v[1], 0x21);
    assert_eq!(c.v[0], 0x42); // 0x63 - 0x21
}

#[test]
fn test_0x8xy6() {
    let mut c = Cpu::new(None).unwrap();
    exec_test_prog(&vec![0x6084, 0x6101, 0x8016], &mut c);
    assert_eq!(c.pc, 0x206);
    assert_eq!(c.v[1], 0x01);
    assert_eq!(c.v[0], 0x42); // 0x84 >> 0x01
}

#[test]
fn test_0x8xy7() {
    let mut c = Cpu::new(None).unwrap();
    exec_test_prog(&vec![0x6020, 0x6162, 0x8017], &mut c);
    assert_eq!(c.pc, 0x206);
    assert_eq!(c.v[1], 0x62);
    assert_eq!(c.v[0], 0x42); // 0x62 - 0x20
}

#[test]
fn test_0x8xye() {
    let mut c = Cpu::new(None).unwrap();
    exec_test_prog(&vec![0x6021, 0x6101, 0x801e], &mut c);
    assert_eq!(c.pc, 0x206);
    assert_eq!(c.v[1], 0x01);
    assert_eq!(c.v[0], 0x42); // 0x21 << 0x01
}

#[test]
fn test_0x9xy0() {
    let mut c = Cpu::new(None).unwrap();
    exec_test_prog(&vec![0x6021, 0x6121, 0x9010], &mut c);
    assert_eq!(c.pc, 0x206); // v[x] == v[y], pc + 1*OP_LEN
    let mut c = Cpu::new(None).unwrap();
    exec_test_prog(&vec![0x6021, 0x6122, 0x9010], &mut c);
    assert_eq!(c.pc, 0x208); // v[x] != v[y], pc + 2*OP_LEN
}

#[test]
fn test_0xannn() {
    let mut c = Cpu::new(None).unwrap();
    exec_test_prog(&vec![0xa042], &mut c);
    assert_eq!(c.pc, 0x202);
    assert_eq!(c.i, 0x042); // i = nnn
}

#[test]
fn test_0xbnnn() {
    let mut c = Cpu::new(None).unwrap();
    exec_test_prog(&vec![0x6021, 0xb221], &mut c);
    assert_eq!(c.pc, 0x242); // pc = v[0] + nnn
}

#[test]
fn test_0xfx1e() {
    let mut c = Cpu::new(None).unwrap();
    exec_test_prog(&vec![0x6042, 0xf01e], &mut c);
    assert_eq!(c.pc, 0x204);
    assert_eq!(c.i, 0x0042); // i += v[x]
}
/* }}} */

/* vim: set fdm=marker : */
