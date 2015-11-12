extern crate pluginapi;
extern crate irc;
use irc::client::server::utils::ServerExt;

use pluginapi::{IrcServer, Plugin};

struct Shifter {
    shl_command: String,
    shr_command: String,
}

impl Plugin for Shifter {
    fn handle_command(&mut self, target: &str, cmd: &str, serv: &IrcServer) {
        if cmd.starts_with(&self.shl_command) {
            let wot = &cmd[self.shl_command.len()..];
            serv.send_privmsg(target, &shl(wot)).unwrap();
        }
        if cmd.starts_with(&self.shr_command) {
            let wot = &cmd[self.shr_command.len()..];
            serv.send_privmsg(target, &shr(wot)).unwrap();
        }
    }
}

#[no_mangle]
pub fn init(opts: &std::collections::HashMap<String, String>) -> Box<Plugin> {
    Box::new(Shifter {
        shl_command: opts.get("shl-command".into()).unwrap_or(&"shl ".into()).clone(),
        shr_command: opts.get("shr-command".into()).unwrap_or(&"shr ".into()).clone(),
    })
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
