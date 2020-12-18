use super::*;

#[test]
fn test_init() {
    let c: Cpu = Cpu::new();
    assert_eq!(c.pc, 0x200);
}
