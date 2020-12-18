use super::*;

#[test]
fn test_init() {
    let c: Cpu = Cpu::new();
    assert_eq!(c.pc, 0x200);
}

#[test]
fn test_icycle() {
    let mut c = Cpu::new();
    // 0x6xnn => vx := nn
    // v[0] = 0x42
    c.ram[0x200] = 0x60; c.ram[0x201] = 0x42;
    c.icycle();
    assert_eq!(c.pc, 0x202);
    assert_eq!(c.v[0], 0x42);
}
