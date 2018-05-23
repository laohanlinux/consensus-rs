
// TODO: opz
pub fn equal_delegate(a: &str, b: &Vec<u8>) -> bool {
    String::from_utf8_lossy(b) == String::from_str(a)
}