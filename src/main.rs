extern crate hiirc;
extern crate libloading;
extern crate plugin_api;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate toml;
extern crate zmq;

mod config;
mod boncactl_server;

use config::Config;
use hiirc::IrcWrite;
use libloading::Library;
use plugin_api::{Context, Plugin, PluginMeta};
use std::collections::HashMap;
use std::error::Error;
use std::mem::ManuallyDrop;
use std::sync::{Arc, Mutex};
use std::thread;

struct PluginContainer {
    plugin: ManuallyDrop<Arc<Mutex<Plugin>>>,
    meta: ManuallyDrop<PluginMeta>,
    lib: ManuallyDrop<Library>,
}

impl Drop for PluginContainer {
    fn drop(&mut self) {
        unsafe {
            // First drop the plugin, as it depends on both meta and lib
            ManuallyDrop::drop(&mut self.plugin);
            // Drop meta, it depends on lib
            ManuallyDrop::drop(&mut self.meta);
            // Finally drop the lib
            ManuallyDrop::drop(&mut self.lib);
        }
    }
}

fn reload_plugin(
    name: &str,
    plugins: &mut HashMap<String, PluginContainer>,
) -> Result<(), Box<Error>> {
    plugins.remove(name);
    let plugin = load_plugin(name)?;
    plugins.insert(name.into(), plugin);
    Ok(())
}

fn load_plugin(name: &str) -> Result<PluginContainer, Box<Error>> {
    use std::env::consts::{DLL_PREFIX, DLL_SUFFIX};
    #[cfg(debug_assertions)]
    let root = "target/debug";
    #[cfg(not(debug_assertions))]
    let root = "target/release";
    let path = format!(
        "{dir}/{prefix}{name}{suffix}",
        dir = root,
        prefix = DLL_PREFIX,
        name = name,
        suffix = DLL_SUFFIX
    );
    let lib = Library::new(path)?;
    let plugin = {
        let init = unsafe { lib.get::<fn() -> Arc<Mutex<Plugin>>>(b"init")? };
        init()
    };
    let mut meta = PluginMeta::default();
    plugin.lock().unwrap().register(&mut meta);
    Ok(PluginContainer {
        plugin: ManuallyDrop::new(plugin),
        meta: ManuallyDrop::new(meta),
        lib: ManuallyDrop::new(lib),
    })
}

struct BoncaListener {
    config: Arc<Mutex<Config>>,
    plugins: HashMap<String, PluginContainer>,
    irc: Option<Arc<hiirc::Irc>>,
}

impl BoncaListener {
    pub fn new(config: Arc<Mutex<Config>>) -> Self {
        // Load plugins
        let mut plugins = HashMap::new();
        {
            let cfg = config.lock().unwrap();

            for k in cfg.plugins.keys() {
                plugins.insert(k.clone(), load_plugin(k).unwrap());
            }
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
    pub fn new(config: Arc<Mutex<Config>>) -> Self {
        SyncBoncaListener(Arc::new(Mutex::new(BoncaListener::new(config))))
    }
}

impl hiirc::Listener for SyncBoncaListener {
    fn welcome(&mut self, irc: Arc<hiirc::Irc>) {
        let mut lis = self.0.lock().unwrap();
        lis.irc = Some(irc.clone());
        for c in &lis.config.lock().unwrap().bot.channels {
            irc.join(c, None).unwrap();
        }
    }
    fn channel_msg(
        &mut self,
        irc: Arc<hiirc::Irc>,
        channel: Arc<hiirc::Channel>,
        sender: Arc<hiirc::ChannelUser>,
        message: &str,
    ) {
        use std::fmt::Write;
        let mut lis = self.0.lock().unwrap();
        let prefix = lis.config.lock().unwrap().bot.cmd_prefix.clone();
        let help_string = format!("{}help", prefix.clone());

        if message.starts_with(&help_string) {
            if let Some(arg) = message[help_string.len()..].split_whitespace().next() {
                for plugin in lis.plugins.values() {
                    for cmd in &plugin.meta.commands {
                        if cmd.name == arg {
                            let _ = irc.privmsg(
                                channel.name(),
                                &format!("{}: {}", sender.nickname(), cmd.help),
                            );
                            return;
                        }
                    }
                }
            }
            let mut msg = String::new();
            let _ = write!(
                &mut msg,
                "The following commands are available ({} <command>): ",
                &help_string
            );
            for plugin in lis.plugins.values() {
                for cmd in &plugin.meta.commands {
                    let _ = write!(&mut msg, "{}, ", cmd.name);
                }
            }
            let _ = irc.privmsg(channel.name(), &format!("{}: {}", sender.nickname(), msg));
            return;
        }

        for plugin in lis.plugins.values_mut() {
            std::thread::spawn({
                let plugin = plugin.plugin.clone();
                let message = message.to_owned();
                let irc = irc.clone();
                let channel = channel.clone();
                let sender = sender.clone();
                move || {
                    plugin
                        .lock()
                        .unwrap()
                        .channel_msg(&message, Context::new(&irc, &channel, &sender));
                }
            });
            for cmd in &plugin.meta.commands {
                let cmd_string = format!("{}{}", prefix, cmd.name);
                if message.starts_with(&cmd_string) {
                    std::thread::spawn({
                        let plugin = plugin.plugin.clone();
                        let irc = irc.clone();
                        let channel = channel.clone();
                        let sender = sender.clone();
                        let arg = message[cmd_string.len()..].trim_left().to_owned();
                        let fun = cmd.fun;
                        move || {
                            fun(
                                &mut *plugin.lock().unwrap(),
                                &arg,
                                Context::new(&irc, &channel, &sender),
                            );
                        }
                    });
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
            panic!(
                "Could not copy {} to {}. Try copying it manually. (error: {})",
                TEMPLATE_PATH,
                config::PATH,
                e
            );
        });
        println!(
            "Created configuration file \"{}\". Please review it.",
            config::PATH
        );
        return;
    }

    let config = config::load().unwrap_or_else(|e| panic!("Error loading config: {}", e));
    let mut server = config.server.url.clone();
    let nick = config.bot.nick.clone();
    server.push_str(":6667");
    let config = Arc::new(Mutex::new(config));

    let listener = SyncBoncaListener::new(config.clone());
    let listener_clone = listener.clone();
    thread::spawn(move || {
        let settings = hiirc::Settings::new(&server, &nick);
        settings
            .dispatch(listener_clone)
            .unwrap_or_else(|e| panic!("Failed to dispatch: {:?}", e));
    });

    let zmq_ctx = zmq::Context::new();
    let sock = zmq_ctx.socket(zmq::SocketType::REP).unwrap();
    sock.bind("ipc:///tmp/boncarobot.sock").unwrap();
    let mut quit_requested = false;

    while !quit_requested {
        if let Ok(Ok(command_str)) = sock.recv_string(zmq::DONTWAIT) {
            let mut lis = listener.0.lock().unwrap();
            let mut config = config.lock().unwrap();
            boncactl_server::handle_command(
                &command_str,
                &mut lis,
                &mut config,
                &sock,
                &mut quit_requested,
            );
        }
        // Don't overwork ourselves
        thread::sleep(std::time::Duration::from_millis(250));
    }
}
