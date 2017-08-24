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
mod listener;
mod plugin_container;

use listener::SyncBoncaListener;
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
            let mut lis = listener.lock();
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
