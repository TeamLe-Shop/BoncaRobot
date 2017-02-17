#![warn(missing_docs)]

//! The plugin API.

pub extern crate hiirc;

/// The most commonly used types when implementing a plugin.
pub mod prelude {
    pub use super::{Plugin, Context};
    pub use hiirc::IrcWrite;
}

/// IRC context.
pub struct Context<'a> {
    /// The hiirc Irc handle through which you can send commands and stuff.
    pub irc: &'a hiirc::Irc,
    /// The channel that the event happened on.
    pub channel: &'a hiirc::Channel,
    /// The user that caused the event.
    pub sender: &'a hiirc::ChannelUser,
}

/// Every plugin must implement this trait.
pub trait Plugin: Send {
    /// Executed when a message is sent to a channel.
    fn channel_msg(&mut self, msg: &str, ctx: Context);
    /// Every plugin must be constructible without arguments.
    fn new() -> Self where Self: Sized;
}

/// Declare a type to be the plugin.
///
/// Only one type per crate can be the plugin.
#[macro_export]
macro_rules! plugin_export {
    ($plugin:tt) => {
        #[no_mangle]
        pub fn init() -> Box<Plugin> {
            Box::new($plugin::new())
        }
    }
}
