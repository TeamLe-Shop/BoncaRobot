extern crate hiirc;
extern crate toml;
extern crate libloading;
extern crate zmq;
extern crate plugin_api;

use hiirc::IrcWrite;
use libloading::{Library, Symbol};
use plugin_api::Plugin;
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::thread;

struct PluginContainer {
    plugin: Box<plugin_api::Plugin>,
    /// The `Library` must kept alive as long as code from the plugin can run.
    ///
    /// WARNING: We are relying here on the current unspecified FIFO drop order of Rust.
    ///
    /// `_lib` must be dropped last, because the plugin can run drop code that would be
    /// destroyed if the library was destroyed first.
    _lib: Library,
}

mod config;

fn reload_plugin(name: &str,
                 plugins: &mut HashMap<String, PluginContainer>)
                 -> Result<(), Box<Error>> {
    plugins.remove(name);
    // Reload the configuration
    let cfg = match config::load_config_for_plugin(name) {
        Ok(config) => config,
        Err(e) => {
            println!("Warning: Failed to load config for plugin: {}", e);
            config::Plugin {
                name: name.into(),
                options: HashMap::new(),
            }
        }
    };
    let plugin = load_plugin(&cfg)?;
    plugins.insert(name.into(), plugin);
    Ok(())
}

fn load_plugin(plugin: &config::Plugin) -> Result<PluginContainer, Box<Error>> {
    #[cfg(debug_assertions)]
    let root = "target/debug";
    #[cfg(not(debug_assertions))]
    let root = "target/release";
    let path = format!("{}/lib{}.so", root, plugin.name);
    let lib = Library::new(path)?;
    let plugin = {
        let init: Symbol<fn() -> Box<Plugin>> = unsafe { lib.get(b"init")? };
        init()
    };
    Ok(PluginContainer {
        plugin: plugin,
        _lib: lib,
    })
}

struct BoncaListener {
    config: config::Config,
    plugins: HashMap<String, PluginContainer>,
    irc: Option<Arc<hiirc::Irc>>,
}

impl BoncaListener {
    pub fn new(config: config::Config) -> Self {
        // Load plugins
        let mut plugins = HashMap::new();

        for plugin in &config.plugins {
            plugins.insert(plugin.name.clone(), load_plugin(plugin).unwrap());
        }

        BoncaListener {
            config: config,
            plugins: plugins,
            irc: None,
        }
    }
    pub fn request_quit(&self, msg: Option<&str>) {
        self.irc.as_ref().unwrap().quit(msg).unwrap();
    }
    pub fn msg(&self, target: &str, text: &str) {
        self.irc.as_ref().unwrap().privmsg(target, text).unwrap();
    }
    pub fn join(&self, channel: &str) {
        self.irc.as_ref().unwrap().join(channel, None).unwrap();
    }
    pub fn leave(&self, channel: &str) {
        self.irc.as_ref().unwrap().part(channel, None).unwrap();
    }
}

#[derive(Clone)]
struct SyncBoncaListener(Arc<Mutex<BoncaListener>>);

impl SyncBoncaListener {
    pub fn new(config: config::Config) -> Self {
        SyncBoncaListener(Arc::new(Mutex::new(BoncaListener::new(config))))
    }
}

impl hiirc::Listener for SyncBoncaListener {
    fn welcome(&mut self, irc: Arc<hiirc::Irc>) {
        let mut lis = self.0.lock().unwrap();
        lis.irc = Some(irc.clone());
        for c in &lis.config.channels {
            irc.join(c, None).unwrap();
        }
    }
    fn channel_msg(&mut self,
                   irc: Arc<hiirc::Irc>,
                   channel: Arc<hiirc::Channel>,
                   sender: Arc<hiirc::ChannelUser>,
                   message: &str) {
        let mut lis = self.0.lock().unwrap();
        if !message.starts_with(&lis.config.cmd_prefix) {
            return;
        }
        for plugin in lis.plugins.values_mut() {
            plugin.plugin.channel_msg(message,
                                      plugin_api::Context {
                                          irc: &irc,
                                          channel: &channel,
                                          sender: &sender,
                                      })
        }
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

    let mut server = config.server.clone();
    server.push_str(":6667");
    let nick = config.nick.clone();

    let listener = SyncBoncaListener::new(config);
    let listener_clone = listener.clone();
    thread::spawn(move || {
        let settings = hiirc::Settings::new(&server, &nick);
        settings.dispatch(listener_clone)
            .unwrap_or_else(|e| panic!("Failed to dispatch: {:?}", e));
    });

    let zmq_ctx = zmq::Context::new();
    let sock = zmq_ctx.socket(zmq::SocketType::REP).unwrap();
    sock.bind("ipc:///tmp/boncarobot.sock").unwrap();
    let mut quit_requested = false;

    while !quit_requested {
        if let Ok(Ok(command_str)) = sock.recv_string(zmq::DONTWAIT) {
            use std::fmt::Write;

            let mut words = command_str.split(' ');
            let mut reply = String::new();
            let mut lis = listener.0.lock().unwrap();
            match words.next().unwrap() {
                "quit" => {
                    lis.request_quit(words.next());
                    quit_requested = true;
                }
                "say" => {
                    match words.next() {
                        Some(channel) => {
                            let msg = words.collect::<Vec<_>>().join(" ");
                            lis.msg(channel, &msg);
                        }
                        None => writeln!(&mut reply, "Need channel, buddy.").unwrap(),
                    }
                }
                "load" => {
                    use std::collections::HashMap;
                    match words.next() {
                        Some(name) => {
                            let plugin = config::Plugin {
                                name: name.to_owned(),
                                options: HashMap::new(),
                            };
                            match load_plugin(&plugin) {
                                Ok(pc) => {
                                    lis.plugins.insert(name.to_owned(), pc);
                                    writeln!(&mut reply, "Loaded \"{}\" plugin.", name).unwrap();
                                    for channel in lis.irc.as_ref().unwrap().channels() {
                                        lis.msg(channel.name(),
                                                &format!("[Plugin '{}' was loaded]", name));
                                    }
                                }
                                Err(e) => {
                                    writeln!(&mut reply, "Failed to load \"{}\": {}", name, e)
                                        .unwrap();
                                }
                            }
                        }
                        None => writeln!(&mut reply, "Name, please!").unwrap(),
                    }
                }
                "unload" => {
                    match words.next() {
                        Some(name) => {
                            if lis.plugins.remove(name).is_some() {
                                writeln!(&mut reply, "Removed \"{}\" plugin.", name).unwrap();
                                for channel in lis.irc.as_ref().unwrap().channels() {
                                    lis.msg(channel.name(),
                                            &format!("[Plugin '{}' was unloaded]", name));
                                }
                            }
                        }
                        None => writeln!(&mut reply, "Don't forget the name!").unwrap(),
                    }
                }
                "reload" => {
                    match words.next() {
                        Some(name) => {
                            match reload_plugin(name, &mut lis.plugins) {
                                Ok(()) => {
                                    writeln!(&mut reply, "Reloaded plugin {}", name).unwrap();
                                    for channel in lis.irc.as_ref().unwrap().channels() {
                                    lis.msg(channel.name(),
                                            &format!("[Plugin '{}' was reloaded]", name));
                                    }
                                }
                                Err(e) => {
                                    writeln!(&mut reply, "Failed to reload plugin {}: {}", name, e)
                                        .unwrap()
                                }
                            }
                        }
                        None => writeln!(&mut reply, "Need a name, faggot").unwrap(),
                    }
                }
                "join" => {
                    match words.next() {
                        Some(name) => {
                            lis.join(name);
                        }
                        None => writeln!(&mut reply, "Need a channel name to join").unwrap(),
                    }
                }
                "leave" => {
                    match words.next() {
                        Some(name) => {
                            lis.leave(name);
                        }
                        None => writeln!(&mut reply, "Need a channel name to leave").unwrap(),
                    }
                }
                _ => writeln!(&mut reply, "Unknown command, bro.").unwrap(),
            }
            sock.send(&reply, 0).unwrap();
        }
        // Don't overwork ourselves
        thread::sleep(std::time::Duration::from_millis(250));
    }
}
