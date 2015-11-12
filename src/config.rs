use irc::client::prelude::Config as IrcConfig;
use std::io;
use std::io::prelude::*;
use toml::ParserError;
use std::error::Error;
use std::fmt;
use std::collections::HashMap;

pub struct Plugin {
    pub name: String,
    pub options: HashMap<String, String>,
}

pub struct Config {
    pub irc: IrcConfig,
    pub cmd_prefix: String,
    pub plugins: Vec<Plugin>,
}

#[derive(Debug)]
pub struct ParserErrors(Vec<ParserError>);

impl Error for ParserErrors {
    fn description(&self) -> &str {
        "TOML parser errors"
    }
}

impl fmt::Display for ParserErrors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(writeln!(f, "=== Errors while parsing TOML ==="));
        for e in &self.0 {
            try!(writeln!(f, "{}", e));
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum LoadError {
    Io(io::Error),
    Parser(ParserErrors),
}

impl Error for LoadError {
    fn description(&self) -> &str {
        "IRC configuration load error"
    }
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            LoadError::Io(ref err) => write!(f, "{}", err),
            LoadError::Parser(ref err) => write!(f, "{}", err),
        }
    }
}

impl From<io::Error> for LoadError {
    fn from(src: io::Error) -> Self {
        LoadError::Io(src)
    }
}

impl From<ParserErrors> for LoadError {
    fn from(src: ParserErrors) -> Self {
        LoadError::Parser(src)
    }
}

pub const PATH: &'static str = "boncarobot.toml";

pub fn load() -> Result<Config, LoadError> {
    use std::fs::File;
    use toml::{Parser, Value};

    let mut file = try!(File::open(PATH));
    let mut buf = String::new();
    try!(file.read_to_string(&mut buf));
    let mut parser = Parser::new(&buf);
    let table = match parser.parse() {
        Some(table) => table,
        None => return Err(LoadError::from(ParserErrors(parser.errors))),
    };
    let mut config = IrcConfig {
        server: Some("chat.freenode.net".to_owned()),
        nickname: Some("boncarobot".to_owned()),
        channels: Some(vec!["#boncarobot".to_owned()]),
        ..Default::default()
    };
    if let Some(&Value::Table(ref server)) = table.get("server") {
        if let Some(&Value::String(ref url)) = server.get("url") {
            config.server = Some(url.clone());
        }
    }
    let mut cmd_prefix = String::new();
    if let Some(&Value::Table(ref bot)) = table.get("bot") {
        if let Some(&Value::String(ref nick)) = bot.get("nick") {
            config.nickname = Some(nick.clone());
        }
        if let Some(&Value::Array(ref array)) = bot.get("channels") {
            let mut channels = Vec::new();

            for v in array.iter() {
                if let &Value::String(ref channel) = v {
                    channels.push(channel.clone());
                }
            }

            config.channels = Some(channels);
        }
        if let Some(&Value::String(ref string)) = bot.get("command-prefix") {
            cmd_prefix = string.clone();
        }
    }
    let mut plugins_vec = Vec::new();
    if let Some(&Value::Table(ref plugins)) = table.get("plugins") {
        for (name, plugin) in plugins {
            if let Value::Table(ref options) = *plugin {
                let mut options_hashmap = HashMap::new();
                for (k, v) in options {
                    if let &Value::String(ref string_value) = v {
                        options_hashmap.insert(k.clone(), string_value.clone());
                    } else {
                        panic!("Unexpected non-string plugin option {:?}.", v);
                    }
                }
                plugins_vec.push(Plugin {
                    name: name.clone(),
                    options: options_hashmap,
                })
            } else {
                panic!("Unexpected non-table plugin entry {:?}.", plugin);
            }
        }
    }
    Ok(Config {
        irc: config,
        cmd_prefix: cmd_prefix,
        plugins: plugins_vec,
    })
}
