#[no_mangle]
pub fn respond_to_command(cmd: &str, mut buf: &mut [u8]) {
    use std::io::Write;
    let shl_command = "shl ";
    let shr_command = "shr ";
    if cmd.starts_with(shl_command) {
        let wot = &cmd[shl_command.len()..];
        let _ = write!(buf, "{}", shl(wot));
    } else if cmd.starts_with(shr_command) {
        let wot = &cmd[shr_command.len()..];
        let _ = write!(buf, "{}", shr(wot));
    }
}

fn find_shl(seq: &[u8], c: char) -> Option<char> {
    if let Some(pos) = seq.iter().position(|b| *b == c as u8) {
        if pos > 0 {
            Some(seq[pos - 1] as char)
        } else {
            Some(*seq.last().unwrap() as char)
        }
    } else {
        None
    }
}

fn find_shr(seq: &[u8], c: char) -> Option<char> {
    if let Some(pos) = seq.iter().position(|b| *b == c as u8) {
        if pos < seq.len() - 1 {
            Some(seq[pos + 1] as char)
        } else {
            Some(*seq.first().unwrap() as char)
        }
    } else {
        None
    }
}

fn driver<T: Fn(&[u8], char) -> Option<char>>(txt: &str, f: T) -> String {
    txt.chars()
       .map(|c| {
           f(b"qwertyuiop", c)
               .or_else(|| f(b"QWERTYUIOP", c))
               .or_else(|| f(b"asdfghjkl", c))
               .or_else(|| f(b"ASDFGHJKL", c))
               .or_else(|| f(b"zxcvbnm", c))
               .or_else(|| f(b"ZXCVBNM", c))
               .or_else(|| f(b"1234567890", c))
               .unwrap_or(c)
       })
       .collect()
}

fn shl(txt: &str) -> String {
    driver(txt, find_shl)
}

fn shr(txt: &str) -> String {
    driver(txt, find_shr)
}

#[test]
fn test() {
    assert_eq!(shl("_X_C_V_B"), "_Z_X_C_V");
    assert_eq!(shl("QWERTY"), "PQWERT");
    assert_eq!(shl("1936"), "0825");
    assert_eq!(shl("z"), "m");
    assert_eq!(shr("_X_C_V_B"), "_C_V_B_N");
    assert_eq!(shr("QWERTY"), "WERTYU");
    assert_eq!(shr("1936"), "2047");
    assert_eq!(shr("z"), "x");
    assert_eq!(shr("m"), "z");
}
