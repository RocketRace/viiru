pub fn assume_string(bytes: Vec<u8>) -> String {
    String::from_utf8(bytes).unwrap()
}

pub fn parse_rgb(s: &str) -> (u8, u8, u8) {
    let bits = u32::from_str_radix(s, 16).unwrap();
    ((bits >> 16) as u8, (bits >> 8) as u8, bits as u8)
}
