use crate::config::Config;
use crate::plugin_container::PluginContainer;
use distance::damerau_levenshtein;
use hiirc::{Channel, ChannelUser, Irc, IrcWrite, Listener};
use plugin_api::Context;
use split_whitespace_rest::SplitWhitespace;
use std;
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex, MutexGuard};

/// The core of the bot.
///
/// All user-facing functionality is implemented through plugins.
/// The core is responsible for handling the IRC events, and notifying the plugins about it.
///
/// It can also be manipulated through IPC.
pub(crate) struct Core {
    config: Arc<Mutex<Config>>,
    plugins: HashMap<String, PluginContainer>,
    pub irc_bridge: IrcBridge,
}

/// Allows IRC access (send messages/join/leave/quit/etc.) for IPC clients.
pub(crate) struct IrcBridge {
    /// IRC handle. It has delayed initialization, but can be assumed to be always `Some` after
    /// the initialization.
    handle: Option<Arc<Irc>>,
}

impl IrcBridge {
    fn new() -> Self {
        Self { handle: None }
    }
    fn init(&mut self, irc: Arc<Irc>) {
        self.handle = Some(irc);
    }
    pub fn request_quit(&self, msg: Option<&str>) {
        self.handle.as_ref().unwrap().quit(msg).unwrap();
    }
    pub fn msg(&self, target: &str, text: &str) {
        self.handle.as_ref().unwrap().privmsg(target, text).unwrap();
    }
    pub fn msg_all_joined_channels(&self, text: &str) {
        for channel in self.handle.as_ref().unwrap().channels() {
            self.msg(channel.name(), text);
        }
    }
    pub fn join(&self, channel: &str) {
        self.handle.as_ref().unwrap().join(channel, None).unwrap();
    }
    pub fn leave(&self, channel: &str) {
        self.handle.as_ref().unwrap().part(channel, None).unwrap();
    }
}

impl Core {
    pub fn new(config: Arc<Mutex<Config>>) -> Self {
        // Load plugins
        let mut plugins = HashMap::new();
        {
            let cfg = config.lock().unwrap();

            for k in cfg.plugins.keys() {
                plugins.insert(k.clone(), PluginContainer::load(k).unwrap());
            }
        }

        Self {
            config,
            plugins,
            irc_bridge: IrcBridge::new(),
        }
    }
    fn channel_msg(
        &mut self,
        irc: &Arc<Irc>,
        channel: &Arc<Channel>,
        sender: &Arc<ChannelUser>,
        message: &str,
    ) {
        let prefix = self.config.lock().unwrap().bot.cmd_prefix.clone();
        if !self.handle_help(&prefix, irc, channel, sender, message) {
            self.delegate_to_plugins(&prefix, irc, channel, sender, message);
        }
    }
    /// Recognize and handle the help command. Returns whether the command we looked at was
    /// the help command.
    fn handle_help(
        &mut self,
        prefix: &str,
        irc: &Irc,
        channel: &Channel,
        sender: &ChannelUser,
        message: &str,
    ) -> bool {
        use std::fmt::Write;
        let help_string = format!("{}help", prefix);

        if message.starts_with(&help_string) {
            if let Some(arg) = message[help_string.len()..].split_whitespace().next() {
                for plugin in self.plugins.values() {
                    for cmd in &plugin.meta.commands {
                        if cmd.name == arg {
                            let _ = irc.privmsg(
                                channel.name(),
                                &format!("{}: {}", sender.nickname(), cmd.help),
                            );
                            return true;
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
            return true;
        }
        false
    }
    fn delegate_to_plugins(
        &mut self,
        command_prefix: &str,
        irc: &Arc<Irc>,
        channel: &Arc<Channel>,
        sender: &Arc<ChannelUser>,
        message: &str,
    ) {
        if is_valid_command(message, command_prefix) {
            self.handle_command(irc, channel, sender, &message[command_prefix.len()..]);
        }
        self.delegate_non_command(irc, channel, sender, message);
    }
    fn handle_command(
        &mut self,
        irc: &Arc<Irc>,
        channel: &Arc<Channel>,
        sender: &Arc<ChannelUser>,
        command: &str,
    ) {
        let mut sw = SplitWhitespace::new(command);
        let command = match sw.next() {
            Some(command) => command,
            None => return,
        };
        let command = &command.to_lowercase();
        let arg = sw.rest_as_slice();
        let mut match_found = false;
        let mut closest_match = ("", usize::max_value());
        for plugin in self.plugins.values_mut() {
            for cmd in &plugin.meta.commands {
                if command == cmd.name {
                    match_found = true;
                    std::thread::spawn({
                        let plugin = plugin.plugin.clone();
                        let irc = Arc::clone(irc);
                        let channel = Arc::clone(channel);
                        let sender = Arc::clone(sender);
                        let arg = arg.trim_start().to_owned();
                        let fun = cmd.fun;
                        move || {
                            fun(
                                &mut *plugin.lock().unwrap(),
                                &arg,
                                Context::new(&irc, &channel, &sender),
                            );
                        }
                    });
                } else {
                    let distance = damerau_levenshtein(command, cmd.name);
                    if distance < closest_match.1 {
                        closest_match = (cmd.name, distance);
                    }
                }
            }
        }
        if !match_found {
            let _ = irc.privmsg(
                channel.name(),
                &format!(
                    "Unknown command: {}. Did you mean '{}'?",
                    command, closest_match.0
                ),
            );
        }
    }
    fn delegate_non_command(
        &mut self,
        irc: &Arc<Irc>,
        channel: &Arc<Channel>,
        sender: &Arc<ChannelUser>,
        message: &str,
    ) {
        for plugin in self.plugins.values_mut() {
            let plugin = plugin.plugin.clone();
            let message = message.to_owned();
            let irc = Arc::clone(irc);
            let channel = Arc::clone(channel);
            let sender = Arc::clone(sender);
            std::thread::spawn(move || {
                plugin
                    .lock()
                    .unwrap()
                    .channel_msg(&message, Context::new(&irc, &channel, &sender));
            });
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

fn is_valid_command(message: &str, prefix: &str) -> bool {
    // A valid command is `prefix` immediately succeeded by an alphabetic character
    let ml = message.len();
    let pl = prefix.len();
    if ml > pl && &message.as_bytes()[..pl] == prefix.as_bytes() {
        if let Some(ch) = message[pl..].chars().next() {
            return ch.is_alphabetic();
        }
    }
    false
}

/// Thread-safe wrapper around `Core` that allows it to be shared between
/// the IRC dispatch loop and the IPC listener, which are on different threads.
#[derive(Clone)]
pub struct SharedCore(pub(crate) Arc<Mutex<Core>>);

impl SharedCore {
    pub fn new(config: Arc<Mutex<Config>>) -> Self {
        SharedCore(Arc::new(Mutex::new(Core::new(config))))
    }
    pub(crate) fn lock(&self) -> MutexGuard<Core> {
        self.0.lock().unwrap()
    }
}

impl Listener for SharedCore {
    fn welcome(&mut self, irc: Arc<Irc>) {
        let mut core = self.0.lock().unwrap();
        for c in &core.config.lock().unwrap().bot.channels {
            irc.join(c, None).unwrap();
        }
        core.irc_bridge.init(irc);
    }
    fn channel_msg(
        &mut self,
        irc: Arc<Irc>,
        channel: Arc<Channel>,
        sender: Arc<ChannelUser>,
        message: &str,
    ) {
        self.lock().channel_msg(&irc, &channel, &sender, message);
    }
}
