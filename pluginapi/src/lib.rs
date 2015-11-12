//! Plugin API for BoncaRobot.
//!
//! Each plugin is contained in a shared library (dll/so).
//!
//! Each plugin must expose a function called `init()` that returns a `Box<Plugin>`.
//!
//! Typically you would have a custom type that implements `Plugin`, and return that in `init`.

extern crate irc;

use irc::client::conn::NetStream;
use std::io::{BufReader, BufWriter};

/// The IRC server that BoncaRobot uses.
pub type IrcServer = irc::client::prelude::IrcServer<BufReader<NetStream>, BufWriter<NetStream>>;

/// A BoncaRobot plugin.
pub trait Plugin {
    /// Handle a command
    fn handle_command(&mut self, target: &str, command: &str, irc: &IrcServer);
}
