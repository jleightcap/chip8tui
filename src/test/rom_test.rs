use std::io::ErrorKind;
use super::*;

#[test]
fn test_new_prog() {
    let r = ROM::new_prog(&[0xde, 0xad]).unwrap();
    assert_eq!(&r.rom[0..2], [0xde, 0xad]);

    let r = ROM::new_prog(&[0x00; RAM_SIZE]).map_err(|e| e.kind());
    assert_eq!(r, Err(ErrorKind::InvalidData));
}
