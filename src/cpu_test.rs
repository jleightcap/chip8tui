use super::*;

// execute a vector of machine code
fn exec_test_prog(p: &Vec<u16>, c: &mut Cpu) {
    for i in p {
        c.opcode_init(*i);
        c.icycle();
    }
}

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
    exec_test_prog(&vec![0x6042], &mut c);
    assert_eq!(c.pc, 0x202);
    assert_eq!(c.v[0], 0x42);
}

#[test]
fn test_0x7xnn() {
    // 0x7xnn => vx += nn
    // v[0] += 1
    let mut c = Cpu::new();
    exec_test_prog(&vec![0x7001], &mut c);
    assert_eq!(c.pc, 0x202);
    assert_eq!(c.v[0], 1);
}

#[test]
fn test_0x8xn0() {
    let mut c = Cpu::new();
    exec_test_prog(&vec![0x6042, 0x8100], &mut c);
    assert_eq!(c.pc, 0x204);
    assert_eq!(c.v[0], 0x42);
    assert_eq!(c.v[1], 0x42);
}

#[test]
fn test_0x8xy1() {
    let mut c = Cpu::new();
    exec_test_prog(&vec![0x6042, 0x61ff, 0x8011], &mut c);
    assert_eq!(c.pc, 0x206);
    assert_eq!(c.v[1], 0xff);
    assert_eq!(c.v[0], 0xff); // 0x42 | 0xff
}

#[test]
fn test_0x8xy2() {
    let mut c = Cpu::new();
    exec_test_prog(&vec![0x6042, 0x61ff, 0x8012], &mut c);
    assert_eq!(c.pc, 0x206);
    assert_eq!(c.v[1], 0xff);
    assert_eq!(c.v[0], 0x42); // 0x42 & 0xff
}

#[test]
fn test_0x8xy3() {
    let mut c = Cpu::new();
    exec_test_prog(&vec![0x6042, 0x61ff, 0x8013], &mut c);
    assert_eq!(c.pc, 0x206);
    assert_eq!(c.v[1], 0xff);
    assert_eq!(c.v[0], 0xbd); // 0x42 ^ 0xff
}

#[test]
fn test_0x8xy4() {
    let mut c = Cpu::new();
    exec_test_prog(&vec![0x6021, 0x6121, 0x8014], &mut c);
    assert_eq!(c.pc, 0x206);
    assert_eq!(c.v[1], 0x21);
    assert_eq!(c.v[0], 0x42); // 0x21 + 0x21
}

#[test]
fn test_0x8xy5() {
    let mut c = Cpu::new();
    exec_test_prog(&vec![0x6063, 0x6121, 0x8015], &mut c);
    assert_eq!(c.pc, 0x206);
    assert_eq!(c.v[1], 0x21);
    assert_eq!(c.v[0], 0x42); // 0x63 - 0x21
}

#[test]
fn test_0x8xy6() {
    let mut c = Cpu::new();
    exec_test_prog(&vec![0x6084, 0x6101, 0x8016], &mut c);
    assert_eq!(c.pc, 0x206);
    assert_eq!(c.v[1], 0x01);
    assert_eq!(c.v[0], 0x42); // 0x84 >> 0x01
}

#[test]
fn test_0x8xy7() {
    let mut c = Cpu::new();
    exec_test_prog(&vec![0x6020, 0x6162, 0x8017], &mut c);
    assert_eq!(c.pc, 0x206);
    assert_eq!(c.v[1], 0x62);
    assert_eq!(c.v[0], 0x42); // 0x62 - 0x20
}
