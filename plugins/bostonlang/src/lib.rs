extern crate bostondict;

use bostondict::BostonDict;

thread_local!(static DICT: BostonDict = BostonDict::new());

#[no_mangle]
pub fn respond_to_command(cmd: &str, mut buf: &mut [u8]) {
    DICT.with(|dict| {
        use std::io::Write;
        let b2ecmd = "b2e ";
        let e2bcmd = "e2b ";
        let translated = if cmd.starts_with(b2ecmd) {
            dict.boston_to_eng(&cmd[b2ecmd.len()..])
        } else if cmd.starts_with(e2bcmd) {
            dict.eng_to_boston(&cmd[e2bcmd.len()..])
        } else {
            String::new()
        };
        let _ = write!(buf, "{}", translated);
    });
}
