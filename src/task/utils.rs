#[allow(unused)]
pub fn uncolorize<S: ToString>(s: S) -> String {
    String::from_utf8(strip_ansi_escapes::strip(s.to_string()).unwrap()).unwrap()
}
