extern crate irc;
extern crate librc;
extern crate toml;
extern crate dylib;
extern crate pluginapi;

use irc::client::prelude::*;
use dylib::DynamicLibrary;
use std::path::Path;
use std::collections::HashMap;

mod config;

struct PluginDylibPair {
    plugin: Box<pluginapi::Plugin>,
    _dylib: DynamicLibrary,
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
    let mut plugin_dylib_pairs = Vec::new();

    for plugin in config.plugins {
        let path = format!("plugins/{0}/target/release/lib{0}.so", plugin.name);
        let dl = DynamicLibrary::open(Some(&Path::new(&path))).unwrap();
        let init: fn(&HashMap<String, String>) -> Box<pluginapi::Plugin> = unsafe {
            std::mem::transmute(dl.symbol::<()>(// )>(
                                                "init")
                                  .unwrap())
        };
        let plugin = init(&plugin.options);
        plugin_dylib_pairs.push(PluginDylibPair {
            plugin: plugin,
            _dylib: dl,
        });
    }

    let my_nick = config.irc.nickname().to_owned();
    let serv = IrcServer::from_config(config.irc).unwrap();
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
                println!("Doesn't start with cmd_prefix \"{}\"", &config.cmd_prefix);
                continue;
            }
            println!("Okay, starts with cmd_prefix \"{}\"", &config.cmd_prefix);
            let cmd = &suffix[config.cmd_prefix.len()..];
            println!("Command is \"{}\"", cmd);
            for &mut PluginDylibPair{ref mut plugin, ..} in &mut plugin_dylib_pairs {
                plugin.handle_command(target, cmd, &serv);
            }
            if cmd.starts_with("rc ") {
                let wot = &cmd[3..];
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
