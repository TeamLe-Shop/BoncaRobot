#![warn(missing_docs)]

//! The plugin API.

#[macro_use]
extern crate downcast_rs;
pub extern crate hiirc;

use crate::util::SplitChunks;
use downcast_rs::Downcast;

/// The most commonly used types when implementing a plugin.
pub mod prelude {
    pub use super::{Context, Plugin, PluginMeta};
    pub use hiirc::IrcWrite;
}

mod util;

use crate::prelude::*;

/// IRC context.
#[derive(Clone, Copy)]
pub struct Context<'a> {
    /// The hiirc Irc handle through which you can send commands and stuff.
    pub irc: &'a hiirc::Irc,
    /// The channel that the event happened on.
    pub channel: &'a hiirc::Channel,
    /// The user that caused the event.
    pub sender: &'a hiirc::ChannelUser,
}

impl<'a> Context<'a> {
    /// JUST DO IT.
    pub fn new(
        irc: &'a hiirc::Irc,
        channel: &'a hiirc::Channel,
        sender: &'a hiirc::ChannelUser,
    ) -> Self {
        Self {
            irc,
            channel,
            sender,
        }
    }
    /// Send a message to the channel belonging to this context.
    pub fn send_channel(&self, msg: &str) {
        // Even though IRC protocol message length limit is 512,
        // freenode seems to cut off messages starting after about 400 characters.
        for chunk in SplitChunks::new(msg, 400) {
            let chunk = chunk.trim();
            if !chunk.is_empty() {
                let _ = self.irc.privmsg(self.channel.name(), chunk);
            }
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    }
}

/// Type of the function that gets called when a command is invoked.
pub type CommandFn = fn(&mut Plugin, &str, Context);

/// A command that can be invoked by a user.
pub struct Command {
    /// Name of the command that is used for invocation.
    pub name: &'static str,
    /// The help string for this command.
    pub help: &'static str,
    /// The function that gets called when the command is invoked.
    pub fun: CommandFn,
}

/// Metadata for a plugin.
#[derive(Default)]
pub struct PluginMeta {
    /// The commands that this plugin has.
    pub commands: Vec<Command>,
}

impl PluginMeta {
    /// Add a command.
    pub fn command(&mut self, name: &'static str, help: &'static str, fun: CommandFn) {
        self.commands.push(Command { name, help, fun })
    }
}

/// Every plugin must implement this trait.
pub trait Plugin: Send + Downcast {
    /// Executed when a message is sent to a channel.
    fn channel_msg(&mut self, _msg: &str, _ctx: Context) {}
    /// Every plugin must be constructible without arguments.
    fn new() -> Self
    where
        Self: Sized;
    /// Register stuff for this plugin. For example, commands.
    fn register(&self, _meta: &mut PluginMeta) {}
}

impl_downcast!(Plugin);

/// Declare a type to be the plugin.
///
/// Only one type per crate can be the plugin.
#[macro_export]
macro_rules! plugin_export {
    ($plugin:tt) => {
        use std::sync::{Arc, Mutex};
        #[no_mangle]
        pub fn init() -> Arc<Mutex<Plugin>> {
            Arc::new(Mutex::new($plugin::new()))
        }
    };
}
