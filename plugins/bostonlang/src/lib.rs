extern crate pluginapi;
extern crate irc;
extern crate bostondict;

use std::collections::HashMap;
use pluginapi::{IrcServer, Plugin};
use bostondict::BostonDict;
use irc::client::server::utils::ServerExt;

#[no_mangle]
pub fn init(_cfg: HashMap<String, String>) -> Box<Plugin> {
    Box::new(BostonLangPlugin {
        dict: BostonDict::new(),
    })
}

struct BostonLangPlugin {
    dict: BostonDict,
}

impl Plugin for BostonLangPlugin {
    fn handle_command(&mut self, target: &str, command: &str, irc: &IrcServer) {
        let b2ecmd = "b2e ";
        let e2bcmd = "e2b ";
        if command.starts_with(b2ecmd) {
            let translated = self.dict.boston_to_eng(&command[b2ecmd.len()..]);
            irc.send_privmsg(target, &translated).unwrap();
        } else if command.starts_with(e2bcmd) {
            let translated = self.dict.eng_to_boston(&command[e2bcmd.len()..]);
            irc.send_privmsg(target, &translated).unwrap();
        }
    }
}
