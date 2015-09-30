extern crate irc;
extern crate librc;
extern crate toml;

use irc::client::prelude::*;

mod config;
mod shift;

fn main() {
    let config = config::load().unwrap();
    let my_nick = config.nickname().to_owned();
    let serv = IrcServer::from_config(config).unwrap();
    serv.identify().unwrap();
    let mut calc = librc::calc::Calc::new();

    for msg in serv.iter().map(|m| m.unwrap()) {
        println!("{:#?}", msg);
        if let Message{suffix: Some(ref suffix), ref args, ref command, ..} = msg {
            let target = {
                let arg0 = match args.get(0) {
                    Some(arg) => arg,
                    // No args, probably ping
                    None => {
                        if command == "PING" {
                            serv.send("PONG").unwrap();
                        }
                        continue;
                    }
                };
                if arg0 == &my_nick {
                    match msg.get_source_nickname() {
                        Some(nick) => nick,
                        // We don't know who to reply to, so we bail out
                        None => continue,
                    }
                } else {
                    &arg0[..]
                }
            };
            if suffix.starts_with("shl ") {
                let wot = &suffix[4..];
                serv.send_privmsg(target, &shift::shl(wot)).unwrap();
            }
            if suffix.starts_with("shr ") {
                let wot = &suffix[4..];
                serv.send_privmsg(target, &shift::shr(wot)).unwrap();
            }
            if suffix.starts_with("rc ") {
                let wot = &suffix[3..];
                let mut response = String::new();
                for expr in wot.split(';') {
                    match calc.eval(expr) {
                        Ok(num) => response.push_str(&num.to_string()),
                        Err(e) => response.push_str(&e.to_string()),
                    }
                    response.push_str(", ");
                }
                serv.send_privmsg(target, &response).unwrap();
            }
        }
    }
}
