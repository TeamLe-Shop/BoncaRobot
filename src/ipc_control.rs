//! Implementation of IPC control.

use config::{self, Config};
use core::Core;
use scaproust::proto::pair::Pair;
use scaproust::{Ipc, SessionBuilder, Socket};
use std::sync::Mutex;
use std::{thread, time};

/// Listens for IPC messages and handle them.
pub(crate) fn listen(core: &Mutex<Core>, config: &Mutex<Config>) {
    let mut session = SessionBuilder::new().with("ipc", Ipc).build().unwrap();
    let mut socket = session.create_socket::<Pair>().unwrap();
    let tmpdir = ::std::env::temp_dir();
    socket
        .bind(&format!(
            "ipc://{}/boncarobot.sock",
            tmpdir.to_str().unwrap()
        )).unwrap();

    let mut quit_requested = false;

    while !quit_requested {
        if let Ok(buffer) = socket.recv() {
            let mut core = core.lock().unwrap();
            let mut config = config.lock().unwrap();
            handle_command(
                ::std::str::from_utf8(&buffer).unwrap(),
                &mut core,
                &mut config,
                &mut socket,
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
    sock: &mut Socket,
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
    sock.send(reply.into_bytes()).unwrap();
}
