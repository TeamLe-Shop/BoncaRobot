#![feature(recover)]

extern crate irc;
extern crate toml;
extern crate dylib;

use irc::client::prelude::*;
use dylib::DynamicLibrary;
use std::path::Path;
use std::error::Error;
use std::fmt;

mod config;

type RespondToCommand = fn(cmd: &str) -> String;

struct PluginContainer {
    name: String,
    respond_to_command: Option<RespondToCommand>,
    dylib: Option<DynamicLibrary>,
}

impl Drop for PluginContainer {
    fn drop(&mut self) {
        drop(self.respond_to_command.take());
        drop(self.dylib.take());
    }
}

#[derive(Debug)]
struct NoSuchPluginError {
    name: String,
}

impl fmt::Display for NoSuchPluginError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "No such plugin: {:?}", self.name)
    }
}

impl Error for NoSuchPluginError {
    fn description(&self) -> &str {
        "No such plugin"
    }
}

#[derive(Debug)]
struct DylibError {
    err: String,
}

impl fmt::Display for DylibError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Dylib error: {}", self.err)
    }
}

impl Error for DylibError {
    fn description(&self) -> &str {
        "Dynamic library error"
    }
}

fn reload_plugin(name: &str, containers: &mut [PluginContainer]) -> Result<(), Box<Error>> {
    let mut cont = try!(containers.iter_mut()
                                  .find(|cont| cont.name == name)
                                  .ok_or(NoSuchPluginError { name: name.into() }));
    // Reload the configuration
    let cfg = try!(config::load_config_for_plugin(name));
    drop(cont.respond_to_command.take());
    drop(cont.dylib.take());
    *cont = try!(load_dl_init(&cfg));
    Ok(())
}

fn load_dl_init(plugin: &config::Plugin) -> Result<PluginContainer, Box<Error>> {
    let path = format!("plugins/{0}/target/release/lib{0}.so", plugin.name);
    let dl = try!(DynamicLibrary::open(Some(&Path::new(&path))).map_err(|e| DylibError { err: e }));
    let respond_to_command: RespondToCommand = unsafe {
        std::mem::transmute(try!(dl.symbol::<()>("respond_to_command")
                                   .map_err(|e| DylibError { err: e })))
    };
    Ok(PluginContainer {
        name: plugin.name.clone(),
        respond_to_command: Some(respond_to_command),
        dylib: Some(dl),
    })
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
    let mut containers: Vec<PluginContainer> = Vec::new();

    for plugin in config.plugins {
        containers.push(load_dl_init(&plugin).unwrap());
    }

    let my_nick = config.irc.nickname().to_owned();
    let serv = IrcServer::from_config(config.irc).unwrap();
    serv.identify().unwrap();

    for msg in serv.iter().map(|m| m.unwrap()) {
        println!("{:#?}", msg);
        match msg.command {
            Command::PING(srv1, srv2) => serv.send(Command::PONG(srv1, srv2)).unwrap(),
            Command::PRIVMSG(ref target, ref message) => {
                let recipient = if *target == my_nick {
                    match msg.source_nickname() {
                        Some(nick) => nick,
                        // We don't know who to reply to, so we bail out
                        None => continue,
                    }
                } else {
                    &target[..]
                };
                if !message.starts_with(&config.cmd_prefix) {
                    continue;
                }
                let cmd = &message[config.cmd_prefix.len()..];
                let reload_cmd = "reload-plugin ";
                if cmd.starts_with(reload_cmd) {
                    let name = &cmd[reload_cmd.len()..];
                    match reload_plugin(name, &mut containers) {
                        Ok(()) => {
                            serv.send_privmsg(recipient, &format!("Reloaded plugin {}", name)).unwrap();
                        }
                        Err(e) => {
                            serv.send_privmsg(recipient,
                                              &format!("Failed to reload plugin {}: {}", name, e))
                                .unwrap();
                        }
                    }

                }
                for &mut PluginContainer{respond_to_command, ref name, ..} in &mut containers {
                    let fresh = cmd.to_owned();
                    match std::panic::recover(move || respond_to_command.unwrap()(&fresh)) {
                        Ok(msg) => {
                            if !msg.is_empty() {
                                println!("!!! Sending {:?} !!!", msg);
                                serv.send_privmsg(recipient, &msg).unwrap();
                            }
                        }
                        Err(_) => {
                            let errmsg = format!("Plugin \"{}\" panicked.", name);
                            let _ = serv.send_privmsg(recipient, &errmsg);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
