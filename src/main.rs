extern crate hiirc;
extern crate toml;
extern crate dylib;
extern crate zmq;

use dylib::DynamicLibrary;
use hiirc::IrcWrite;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;

mod config;

type RespondToCommand = fn(cmd: &str, sender: &str) -> String;

struct PluginContainer {
    respond_to_command: Option<RespondToCommand>,
    dylib: Option<DynamicLibrary>,
}

// TODO: This is hugely unsafe, don't touch dylibs from two different threads
unsafe impl Send for PluginContainer {}

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

fn reload_plugin(name: &str,
                 containers: &mut HashMap<String, PluginContainer>)
                 -> Result<(), Box<Error>> {
    let mut cont = try!(containers.get_mut(name)
        .ok_or(NoSuchPluginError { name: name.into() }));
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
    drop(cont.respond_to_command.take());
    drop(cont.dylib.take());
    *cont = try!(load_dl_init(&cfg));
    Ok(())
}

fn load_dl_init(plugin: &config::Plugin) -> Result<PluginContainer, Box<Error>> {
    let path = format!("plugins/{0}/target/release/lib{0}.so", plugin.name);
    let dl = try!(DynamicLibrary::open(Some(&Path::new(&path))).map_err(|e| DylibError { err: e }));
    let respond_to_command: RespondToCommand =
        unsafe {
            std::mem::transmute(try!(dl.symbol::<()>("respond_to_command")
                .map_err(|e| DylibError { err: e })))
        };
    Ok(PluginContainer {
        respond_to_command: Some(respond_to_command),
        dylib: Some(dl),
    })
}

struct BoncaListener {
    config: config::Config,
    containers: HashMap<String, PluginContainer>,
    irc: Option<Arc<hiirc::Irc>>,
}

impl BoncaListener {
    pub fn new(config: config::Config) -> Self {
        // Load plugins
        let mut containers = HashMap::new();

        for plugin in &config.plugins {
            containers.insert(plugin.name.clone(), load_dl_init(&plugin).unwrap());
        }

        BoncaListener {
            config: config,
            containers: containers,
            irc: None,
        }
    }
    pub fn request_quit(&self) {
        self.irc.as_ref().unwrap().quit(None).unwrap();
    }
    pub fn msg(&self, target: &str, text: &str) {
        self.irc.as_ref().unwrap().privmsg(target, text).unwrap();
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
        let recipient = channel.name();
        if !message.starts_with(&lis.config.cmd_prefix) {
            return;
        }
        let cmd = &message[lis.config.cmd_prefix.len()..];
        for (name, &mut PluginContainer { respond_to_command, .. }) in &mut lis.containers {
            let fresh = cmd.to_owned();
            let nick = sender.nickname().clone();
            match std::panic::catch_unwind(move || respond_to_command.unwrap()(&fresh, &nick)) {
                Ok(msg) => {
                    if !msg.is_empty() {
                        println!("!!! Sending {:?} !!!", msg);
                        irc.privmsg(recipient, &msg).unwrap();
                    }
                }
                Err(_) => {
                    let errmsg = format!("Plugin \"{}\" panicked.", name);
                    let _ = irc.privmsg(recipient, &errmsg);
                }
            }
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
    let sock = zmq_ctx.socket(zmq::SocketType::PULL).unwrap();
    sock.bind("ipc:///tmp/boncarobot.sock").unwrap();

    loop {
        if let Ok(Ok(command_str)) = sock.recv_string(zmq::DONTWAIT) {
            let mut words = command_str.split(' ');
            match words.next().unwrap() {
                "quit" => listener.0.lock().unwrap().request_quit(),
                "say" => {
                    match words.next() {
                        Some(channel) => {
                            let msg = words.collect::<Vec<_>>().join(" ");
                            listener.0.lock().unwrap().msg(channel, &msg);
                        }
                        None => println!("Need channel, buddy."),
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
                            match load_dl_init(&plugin) {
                                Ok(pc) => {
                                    let mut lis = listener.0.lock().unwrap();
                                    lis.containers.insert(name.to_owned(), pc);
                                    println!("Loaded \"{}\" plugin.", name);
                                }
                                Err(e) => {
                                    println!("Failed to load \"{}\": {}", name, e);
                                }
                            }
                        }
                        None => println!("Name, please!"),
                    }
                }
                "unload" => {
                    match words.next() {
                        Some(name) => {
                            let mut lis = listener.0.lock().unwrap();
                            if lis.containers.remove(name).is_some() {
                                println!("Removed \"{}\" plugin.", name);
                            }
                        }
                        None => println!("Don't forget the name!"),
                    }
                }
                "reload" => {
                    match words.next() {
                        Some(name) => {
                            let mut lis = listener.0.lock().unwrap();
                            match reload_plugin(name, &mut lis.containers) {
                                Ok(()) => println!("Reloaded plugin {}", name),
                                Err(e) => println!("Failed to reload plugin {}: {}", name, e),
                            }
                        }
                        None => println!("Need a name, faggot"),
                    }
                }
                _ => {}
            }
        }
        // Don't overwork ourselves
        thread::sleep(std::time::Duration::from_millis(250));
    }
}
