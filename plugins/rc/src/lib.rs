extern crate librc;

use std::cell::RefCell;
use librc::calc::Calc;

thread_local!(static CALC: RefCell<Calc> = RefCell::new(Calc::new()));


#[no_mangle]
pub fn respond_to_command(cmd: &str, mut buf: &mut [u8]) {
    CALC.with(|calc| {
        if cmd.starts_with("rc ") {
            use std::io::Write;

            let wot = &cmd[3..];
            let mut response = String::new();
            for expr in wot.split(';') {
                match calc.borrow_mut().eval(expr) {
                    Ok(num) => response.push_str(&num.to_string()),
                    Err(e) => response.push_str(&e.to_string()),
                }
                response.push_str(", ");
            }

            let _ = write!(buf, "{}", response);
        }
    });
}
