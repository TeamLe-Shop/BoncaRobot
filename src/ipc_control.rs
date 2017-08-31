//! Implementation of IPC control.

extern crate zmq;

use self::zmq::Socket;
use config::{self, Config};
use core::{Core, SharedCore};
use std::{thread, time};
use std::sync::Mutex;

/// Listens for IPC messages and handle them.
pub fn listen(core: SharedCore, config: &Mutex<Config>) {
    let zmq_ctx = zmq::Context::new();
    let sock = zmq_ctx.socket(zmq::SocketType::REP).unwrap();
    sock.bind("ipc:///tmp/boncarobot.sock").unwrap();
    let mut quit_requested = false;

    while !quit_requested {
        if let Ok(Ok(command_str)) = sock.recv_string(zmq::DONTWAIT) {
            let mut core = core.lock();
            let mut config = config.lock().unwrap();
            handle_command(
                &command_str,
                &mut core,
                &mut config,
                &sock,
                &mut quit_requested,
            );
        }
        // Don't overwork ourselves
        thread::sleep(time::Duration::from_millis(250));
    }
}

fn handle_command(
    command_str: &str,
    core: &mut Core,
    config: &mut Config,
    sock: &Socket,
    quit_requested: &mut bool,
) {
    use std::fmt::Write;

    let mut words = command_str.split(' ');
    let mut reply = String::new();
    match words.next().unwrap() {
        "quit" => {
            core.irc_bridge.request_quit(words.next());
            *quit_requested = true;
        }
        "say" => match words.next() {
            Some(channel) => {
                let msg = words.collect::<Vec<_>>().join(" ");
                core.irc_bridge.msg(channel, &msg);
            }
            None => writeln!(&mut reply, "Need channel, buddy.").unwrap(),
        },
        "load" => match words.next() {
            Some(name) => match core.load_plugin(name) {
                Ok(()) => {
                    writeln!(&mut reply, "Loaded \"{}\" plugin.", name).unwrap();
                    core.irc_bridge
                        .msg_all_joined_channels(&format!("[Plugin '{}' was loaded]", name));
                }
                Err(e) => {
                    writeln!(&mut reply, "Failed to load \"{}\": {}", name, e).unwrap();
                }
            },
            None => writeln!(&mut reply, "Name, please!").unwrap(),
        },
        "unload" => match words.next() {
            Some(name) => if core.unload_plugin(name) {
                writeln!(&mut reply, "Removed \"{}\" plugin.", name).unwrap();
                core.irc_bridge
                    .msg_all_joined_channels(&format!("[Plugin '{}' was unloaded]", name));
            },
            None => writeln!(&mut reply, "Don't forget the name!").unwrap(),
        },
        "reload" => match words.next() {
            Some(name) => match core.reload_plugin(name) {
                Ok(()) => {
                    writeln!(&mut reply, "Reloaded plugin {}", name).unwrap();
                    core.irc_bridge
                        .msg_all_joined_channels(&format!("[Plugin '{}' was reloaded]", name));
                }
                Err(e) => writeln!(&mut reply, "Failed to reload plugin {}: {}", name, e).unwrap(),
            },
            None => writeln!(&mut reply, "Need a name, faggot").unwrap(),
        },
        "reload-cfg" => match config::load() {
            Ok(cfg) => *config = cfg,
            Err(e) => writeln!(&mut reply, "{}", e).unwrap(),
        },
        "join" => match words.next() {
            Some(name) => core.irc_bridge.join(name),
            None => writeln!(&mut reply, "Need a channel name to join").unwrap(),
        },
        "leave" => match words.next() {
            Some(name) => core.irc_bridge.leave(name),
            None => writeln!(&mut reply, "Need a channel name to leave").unwrap(),
        },
        _ => writeln!(&mut reply, "Unknown command, bro.").unwrap(),
    }
    sock.send(reply.as_bytes(), 0).unwrap();
}
