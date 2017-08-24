
use config::{self, Config};
use listener::BoncaListener;
use plugin_hosting::{load_plugin, reload_plugin};
use zmq::Socket;

pub(crate) fn handle_command(
    command_str: &str,
    lis: &mut BoncaListener,
    config: &mut Config,
    sock: &Socket,
    quit_requested: &mut bool,
) {
    use std::fmt::Write;

    let mut words = command_str.split(' ');
    let mut reply = String::new();
    match words.next().unwrap() {
        "quit" => {
            lis.request_quit(words.next());
            *quit_requested = true;
        }
        "say" => match words.next() {
            Some(channel) => {
                let msg = words.collect::<Vec<_>>().join(" ");
                lis.msg(channel, &msg);
            }
            None => writeln!(&mut reply, "Need channel, buddy.").unwrap(),
        },
        "load" => match words.next() {
            Some(name) => match load_plugin(name) {
                Ok(pc) => {
                    lis.plugins.insert(name.to_owned(), pc);
                    writeln!(&mut reply, "Loaded \"{}\" plugin.", name).unwrap();
                    for channel in lis.irc.as_ref().unwrap().channels() {
                        lis.msg(channel.name(), &format!("[Plugin '{}' was loaded]", name));
                    }
                }
                Err(e) => {
                    writeln!(&mut reply, "Failed to load \"{}\": {}", name, e).unwrap();
                }
            },
            None => writeln!(&mut reply, "Name, please!").unwrap(),
        },
        "unload" => match words.next() {
            Some(name) => if lis.plugins.remove(name).is_some() {
                writeln!(&mut reply, "Removed \"{}\" plugin.", name).unwrap();
                for channel in lis.irc.as_ref().unwrap().channels() {
                    lis.msg(channel.name(), &format!("[Plugin '{}' was unloaded]", name));
                }
            },
            None => writeln!(&mut reply, "Don't forget the name!").unwrap(),
        },
        "reload" => match words.next() {
            Some(name) => match reload_plugin(name, &mut lis.plugins) {
                Ok(()) => {
                    writeln!(&mut reply, "Reloaded plugin {}", name).unwrap();
                    for channel in lis.irc.as_ref().unwrap().channels() {
                        lis.msg(channel.name(), &format!("[Plugin '{}' was reloaded]", name));
                    }
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
            Some(name) => lis.join(name),
            None => writeln!(&mut reply, "Need a channel name to join").unwrap(),
        },
        "leave" => match words.next() {
            Some(name) => lis.leave(name),
            None => writeln!(&mut reply, "Need a channel name to leave").unwrap(),
        },
        _ => writeln!(&mut reply, "Unknown command, bro.").unwrap(),
    }
    sock.send(reply.as_bytes(), 0).unwrap();
}
