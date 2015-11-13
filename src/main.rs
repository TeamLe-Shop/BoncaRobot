extern crate irc;
extern crate toml;
extern crate dylib;

use irc::client::prelude::*;
use dylib::DynamicLibrary;
use std::path::Path;

mod config;

type RespondToCommand = fn(cmd: &str) -> String;

struct PluginContainer {
    name: String,
    respond_to_command: RespondToCommand,
    _dylib: DynamicLibrary,
}

fn reload_plugin(name: &str, containers: &mut [PluginContainer]) {
    let mut cont = containers.iter_mut().find(|cont| cont.name == name).unwrap();
    // Reload the configuration
    let cfg = config::load_config_for_plugin(name).unwrap();
    *cont = load_dl_init(&cfg);
}

fn load_dl_init(plugin: &config::Plugin) -> PluginContainer {
    let path = format!("plugins/{0}/target/debug/lib{0}.so", plugin.name);
    let dl = DynamicLibrary::open(Some(&Path::new(&path))).unwrap();
    let respond_to_command: RespondToCommand = unsafe {
        std::mem::transmute(dl.symbol::<()>("respond_to_command").unwrap())
    };
    PluginContainer {
        name: plugin.name.clone(),
        respond_to_command: respond_to_command,
        _dylib: dl,
    }
}

fn main() {
    // If the configuration file does not exist, try copying over the template.
    if !std::path::Path::new(config::PATH).exists() {
        const TEMPLATE_PATH: &'static str = "boncarobot.template.toml";
        std::fs::copy(TEMPLATE_PATH, config::PATH).unwrap_or_else(|e| {
            panic!("Could not copy {} to {}. Try copying it manually. (error: {})",
                   TEMPLATE_PATH,
                   config::PATH,
                   e);
        });
        println!("Created configuration file \"{}\". Please review it.",
                 config::PATH);
        return;
    }
    let config = config::load().unwrap();

    // Load plugins
    let mut containers = Vec::new();

    for plugin in config.plugins {
        containers.push(load_dl_init(&plugin));
    }

    let my_nick = config.irc.nickname().to_owned();
    let serv = IrcServer::from_config(config.irc).unwrap();
    serv.identify().unwrap();

    for msg in serv.iter().map(|m| m.unwrap()) {
        println!("{:#?}", msg);
        if let Message{suffix: Some(ref suffix), ref args, ref command, ..} = msg {
            let target = {
                let arg0 = match args.get(0) {
                    Some(arg) => arg,
                    // No args, probably ping
                    None => {
                        if command == "PING" {
                            serv.send(Command::PONG(suffix.clone(), None)).unwrap();
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
            if !suffix.starts_with(&config.cmd_prefix) {
                continue;
            }
            let cmd = &suffix[config.cmd_prefix.len()..];
            let reload_cmd = "reload-plugin ";
            if cmd.starts_with(reload_cmd) {
                let name = &cmd[reload_cmd.len()..];
                reload_plugin(name, &mut containers);
                serv.send_privmsg(target, &format!("Reloaded plugin {}", name)).unwrap();
            }
            for &mut PluginContainer{respond_to_command, ..} in &mut containers {
                let msg = respond_to_command(cmd);
                if !msg.is_empty() {
                    println!("!!! Sending {:?} !!!", msg);
                    serv.send_privmsg(target, &msg).unwrap();
                }
            }
        }
    }
}
