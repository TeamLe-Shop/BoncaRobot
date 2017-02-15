//! The plugin API.
//!
//! A `Box<Plugin>` is expected to be returned by an unmagled function named `init`.

pub extern crate hiirc;

use std::sync::Arc;

pub mod prelude {
    pub use super::Plugin;
    pub use hiirc::{Irc, Channel, ChannelUser, IrcWrite};
    pub use std::sync::Arc;
}

pub trait Plugin: Send {
    fn channel_msg(&mut self,
                   irc: Arc<hiirc::Irc>,
                   channel: Arc<hiirc::Channel>,
                   sender: Arc<hiirc::ChannelUser>,
                   message: &str);
    fn new() -> Self where Self: Sized;
}

#[macro_export]
macro_rules! plugin_export {
    ($plugin:tt) => {
        #[no_mangle]
        pub fn init() -> Box<Plugin> {
            Box::new($plugin::new())
        }
    }
}
