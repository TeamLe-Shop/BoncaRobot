//! Implementation of user configuration using TOML.

use std::collections::HashMap;
use std::error::Error;
use std::io;
use std::io::prelude::*;
use toml;

#[derive(Deserialize)]
pub struct Plugin {}

#[derive(Deserialize)]
pub struct Server {
    pub url: String,
}

#[derive(Deserialize)]
pub struct Bot {
    pub nick: String,
    pub channels: Vec<String>,
    #[serde(rename = "command-prefix")] pub cmd_prefix: String,
}

#[derive(Deserialize)]
pub struct Config {
    pub server: Server,
    pub bot: Bot,
    pub plugins: HashMap<String, Plugin>,
}

pub const PATH: &str = "boncarobot.toml";

fn load_file_to_string() -> Result<String, io::Error> {
    use std::fs::File;
    let mut file = File::open(PATH)?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    Ok(buf)
}

pub fn load() -> Result<Config, Box<Error>> {
    let text = load_file_to_string()?;
    let config = toml::from_str(&text)?;
    Ok(config)
}
