extern crate librc;
extern crate pluginapi;
extern crate irc;
use irc::client::server::utils::ServerExt;

use pluginapi::{IrcServer, Plugin};
use std::collections::HashMap;

#[no_mangle]
pub fn init(_options: HashMap<String, String>) -> Box<Plugin> {
    Box::new(RcPlugin::new())
}

struct RcPlugin {
    calc: librc::calc::Calc,
}

impl RcPlugin {
    fn new() -> Self {
        RcPlugin {
            calc: librc::calc::Calc::new(),
        }
    }
}

impl Plugin for RcPlugin {
    fn handle_command(&mut self, target: &str, cmd: &str, serv: &IrcServer) {
        if cmd.starts_with("rc ") {
            let wot = &cmd[3..];
            let mut response = String::new();
            for expr in wot.split(';') {
                match self.calc.eval(expr) {
                    Ok(num) => response.push_str(&num.to_string()),
                    Err(e) => response.push_str(&e.to_string()),
                }
                response.push_str(", ");
            }
            serv.send_privmsg(target, &response).unwrap();
        }
    }
}
