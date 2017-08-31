//! Boncactl: Resident overlord @ ##newboston
//!
//! Boncactl is an IRC bot whose functionality is implemented through plugins.
//!
//! It can also be controlled locally through IPC using zeromq.

extern crate hiirc;
extern crate libloading;
extern crate plugin_api;
extern crate scaproust;
#[macro_use]
extern crate serde_derive;
extern crate toml;

mod config;
mod ipc_control;
mod core;
mod plugin_container;

use core::SharedCore;
use std::sync::{Arc, Mutex};
use std::thread;

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

    let core = SharedCore::new(config.clone());
    let core_clone = core.clone();
    thread::spawn(move || {
        let settings = hiirc::Settings::new(&server, &nick);
        settings
            .dispatch(core_clone)
            .unwrap_or_else(|e| panic!("Failed to dispatch: {:?}", e));
    });
    ipc_control::listen(&core.0, &*config);
}
