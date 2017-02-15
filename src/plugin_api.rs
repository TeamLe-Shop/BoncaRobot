#![warn(missing_docs)]

//! The plugin API.

pub extern crate hiirc;

use std::sync::Arc;

/// The most commonly used types when implementing a plugin.
pub mod prelude {
    pub use super::Plugin;
    pub use hiirc::{Irc, Channel, ChannelUser, IrcWrite};
    pub use std::sync::Arc;
}

/// Every plugin must implement this trait.
pub trait Plugin: Send {
    /// Executed when a message is sent to a channel.
    fn channel_msg(&mut self,
                   irc: Arc<hiirc::Irc>,
                   channel: Arc<hiirc::Channel>,
                   sender: Arc<hiirc::ChannelUser>,
                   message: &str);
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
