use std::io::ErrorKind;
use super::*;

#[test]
fn test_poll_key() {
    // valid keys in the keypad
    let k = poll(Some(Key::Char('1'))).unwrap();
    let mut r = [false; 16]; r[0x0] = true;
    assert_eq!(&k[..], r);

    let k = poll(Some(Key::Char('v'))).unwrap();
    let mut r = [false; 16]; r[0xf] = true;
    assert_eq!(&k[..], r);

    // valid key not in the keypad
    let k = poll(Some(Key::Char('h'))).unwrap();
    let r = [false; 16];
    assert_eq!(&k[..], r);
    
    // no key
    let k = poll(None).unwrap();
    let r = [false; 16];
    assert_eq!(&k[..], r);
}

#[test]
fn test_poll_error() {
    let e = poll(Some(Key::Ctrl('c'))).map_err(|e| e.kind());
    assert_eq!(e, Err(ErrorKind::Interrupted));
}
