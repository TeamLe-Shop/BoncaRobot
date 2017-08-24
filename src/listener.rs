use config::Config;
use hiirc::{Channel, ChannelUser, Irc, IrcWrite, Listener};
use plugin_api::Context;
use plugin_container::PluginContainer;
use std;
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex, MutexGuard};

pub(crate) struct BoncaListener {
    config: Arc<Mutex<Config>>,
    plugins: HashMap<String, PluginContainer>,
    irc: Option<Arc<Irc>>,
}

impl BoncaListener {
    pub fn new(config: Arc<Mutex<Config>>) -> Self {
        // Load plugins
        let mut plugins = HashMap::new();
        {
            let cfg = config.lock().unwrap();

            for k in cfg.plugins.keys() {
                plugins.insert(k.clone(), PluginContainer::load(k).unwrap());
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
    pub fn msg_all_joined_channels(&self, text: &str) {
        for channel in self.irc.as_ref().unwrap().channels() {
            self.msg(channel.name(), text);
        }
    }
    pub fn join(&self, channel: &str) {
        self.irc.as_ref().unwrap().join(channel, None).unwrap();
    }
    pub fn leave(&self, channel: &str) {
        self.irc.as_ref().unwrap().part(channel, None).unwrap();
    }
    fn channel_msg(
        &mut self,
        irc: Arc<Irc>,
        channel: Arc<Channel>,
        sender: Arc<ChannelUser>,
        message: &str,
    ) {
        let prefix = self.config.lock().unwrap().bot.cmd_prefix.clone();
        self.handle_help(&prefix, &irc, &channel, &sender, message);
        self.delegate_to_plugins(&prefix, irc, channel, sender, message);
    }
    fn handle_help(
        &mut self,
        prefix: &str,
        irc: &Irc,
        channel: &Channel,
        sender: &ChannelUser,
        message: &str,
    ) {
        use std::fmt::Write;
        let help_string = format!("{}help", prefix.clone());

        if message.starts_with(&help_string) {
            if let Some(arg) = message[help_string.len()..].split_whitespace().next() {
                for plugin in self.plugins.values() {
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
            for plugin in self.plugins.values() {
                for cmd in &plugin.meta.commands {
                    let _ = write!(&mut msg, "{}, ", cmd.name);
                }
            }
            let _ = irc.privmsg(channel.name(), &format!("{}: {}", sender.nickname(), msg));
            return;
        }
    }
    fn delegate_to_plugins(
        &mut self,
        prefix: &str,
        irc: Arc<Irc>,
        channel: Arc<Channel>,
        sender: Arc<ChannelUser>,
        message: &str,
    ) {
        for plugin in self.plugins.values_mut() {
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
    pub fn load_plugin(&mut self, name: &str) -> Result<(), Box<Error>> {
        let pc = PluginContainer::load(name)?;
        self.plugins.insert(name.to_owned(), pc);
        Ok(())
    }
    pub fn unload_plugin(&mut self, name: &str) -> bool {
        self.plugins.remove(name).is_some()
    }
    pub fn reload_plugin(&mut self, name: &str) -> Result<(), Box<Error>> {
        self.plugins.remove(name);
        let plugin = PluginContainer::load(name)?;
        self.plugins.insert(name.into(), plugin);
        Ok(())
    }
}

#[derive(Clone)]
pub struct SyncBoncaListener(Arc<Mutex<BoncaListener>>);

impl SyncBoncaListener {
    pub fn new(config: Arc<Mutex<Config>>) -> Self {
        SyncBoncaListener(Arc::new(Mutex::new(BoncaListener::new(config))))
    }
    pub(crate) fn lock(&self) -> MutexGuard<BoncaListener> {
        self.0.lock().unwrap()
    }
}

impl Listener for SyncBoncaListener {
    fn welcome(&mut self, irc: Arc<Irc>) {
        let mut lis = self.0.lock().unwrap();
        lis.irc = Some(irc.clone());
        for c in &lis.config.lock().unwrap().bot.channels {
            irc.join(c, None).unwrap();
        }
    }
    fn channel_msg(
        &mut self,
        irc: Arc<Irc>,
        channel: Arc<Channel>,
        sender: Arc<ChannelUser>,
        message: &str,
    ) {
        self.lock().channel_msg(irc, channel, sender, message);
    }
}
