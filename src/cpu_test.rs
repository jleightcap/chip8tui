use super::*;

#[test]
fn test_init() {
    let c: Cpu = Cpu::new();
    assert_eq!(c.pc, 0x200);
}


// ========================================================================= //
//
// OPCODE TEST
//
// ========================================================================= //
#[test]
fn test_0x6xnn() {
    // 0x6xnn => vx := nn
    // v[0] = 0x42
    let mut c = Cpu::new();
    c.opcode_init(0x6042);
    c.icycle();
    assert_eq!(c.pc, 0x202);
    assert_eq!(c.v[0], 0x42);
}

#[test]
fn test_0x7xnn() {
    // 0x7xnn => vx += nn
    // v[0] += 1
    let mut c = Cpu::new();
    c.opcode_init(0x7001);
    c.icycle();
    assert_eq!(c.pc, 0x202);
    assert_eq!(c.v[0], 1);
}

#[test]
fn test_0x8xn0() {
    let mut c = Cpu::new();
    c.opcode_init(0x6042);
    c.icycle();
    c.opcode_init(0x8100);
    c.icycle();
    assert_eq!(c.pc, 0x204);
    assert_eq!(c.v[0], 0x42);
    assert_eq!(c.v[1], 0x42);
}

#[test]
fn test_0x8xy1() {
    let mut c = Cpu::new();
    c.opcode_init(0x6042); // v[0] = 0x42
    c.icycle();
    c.opcode_init(0x61ff); // v[1] = 0xff
    c.icycle();
    c.opcode_init(0x8011); // v[0] |= v[1]
    c.icycle();
    assert_eq!(c.pc, 0x206);
    assert_eq!(c.v[1], 0xff);
    assert_eq!(c.v[0], 0xff); // 0x42 | 0xff
}

#[test]
fn test_0x8xy2() {
    let mut c = Cpu::new();
    c.opcode_init(0x6042); // v[0] = 0x42
    c.icycle();
    c.opcode_init(0x61ff); // v[1] = 0xff
    c.icycle();
    c.opcode_init(0x8012); // v[0] &= v[1]
    c.icycle();
    assert_eq!(c.pc, 0x206);
    assert_eq!(c.v[1], 0xff);
    assert_eq!(c.v[0], 0x42); // 0x42 & 0xff
}

#[test]
fn test_0x8xy3() {
    let mut c = Cpu::new();
    c.opcode_init(0x6042); // v[0] = 0x42
    c.icycle();
    c.opcode_init(0x61ff); // v[1] = 0xff
    c.icycle();
    c.opcode_init(0x8013); // v[0] ^= v[1]
    c.icycle();
    assert_eq!(c.pc, 0x206);
    assert_eq!(c.v[1], 0xff);
    assert_eq!(c.v[0], 0xbd); // 0x42 ^ 0xff
}

#[test]
fn test_0x8xy4() {
    let mut c = Cpu::new();
    c.opcode_init(0x6021); // v[0] = 0x21
    c.icycle();
    c.opcode_init(0x6121); // v[1] = 0x21
    c.icycle();
    c.opcode_init(0x8014); // v[0] += v[1]
    c.icycle();
    assert_eq!(c.pc, 0x206);
    assert_eq!(c.v[1], 0x21);
    assert_eq!(c.v[0], 0x42); // 0x21 + 0x21
}

#[test]
fn test_0x8xy5() {
    let mut c = Cpu::new();
    c.opcode_init(0x6063); // v[0] = 0x63
    c.icycle();
    c.opcode_init(0x6121); // v[1] = 0x21
    c.icycle();
    c.opcode_init(0x8015); // v[0] -= v[1]
    c.icycle();
    assert_eq!(c.pc, 0x206);
    assert_eq!(c.v[1], 0x21);
    assert_eq!(c.v[0], 0x42); // 0x63 - 0x21
}

#[test]
fn test_0x8xy6() {
    let mut c = Cpu::new();
    c.opcode_init(0x6084); // v[0] = 0x84
    c.icycle();
    c.opcode_init(0x6101); // v[1] = 0x01
    c.icycle();
    c.opcode_init(0x8016); // v[0] >>= v[1]
    c.icycle();
    assert_eq!(c.pc, 0x206);
    assert_eq!(c.v[1], 0x01);
    assert_eq!(c.v[0], 0x42); // 0x84 >> 0x01
}

#[test]
fn test_0x8xy6() {
    let mut c = Cpu::new();
    c.opcode_init(0x6084); // v[0] = 0x84
    c.icycle();
    c.opcode_init(0x6101); // v[1] = 0x01
    c.icycle();
    c.opcode_init(0x8016); // v[0] >>= v[1]
    c.icycle();
    assert_eq!(c.pc, 0x206);
    assert_eq!(c.v[1], 0x01);
    assert_eq!(c.v[0], 0x42); // 0x84 >> 0x01
}
