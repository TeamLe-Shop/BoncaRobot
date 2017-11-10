extern crate json;
#[macro_use]
extern crate plugin_api;
extern crate reqwest;
extern crate split_whitespace_rest;

use plugin_api::prelude::*;
use std::error::Error;
use std::io::prelude::*;
use split_whitespace_rest::SplitWhitespace;

pub fn query(query: &str) -> Result<String, Box<Error>> {
    let msg = format!(
        "http://api.urbandictionary.com/v0/define?term={}",
        query.replace(' ', "%20").replace('&', "%26")
    );

    let mut resp = reqwest::get(&msg)?;

    if !resp.status().is_success() {
        return Err("Something went wrong with the request".into());
    }

    let mut content = Vec::new();
    resp.read_to_end(&mut content)?;
    Ok(String::from_utf8_lossy(&content).into_owned())
}

struct UdPlugin;

impl UdPlugin {
    fn udn(_this: &mut Plugin, arg: &str, ctx: Context) {
        let mut sw = SplitWhitespace::new(arg);
        let entry = match sw.next() {
            Some(en) => en,
            None => {
                ctx.send_channel("Need entry num bro.");
                return;
            }
        };
        let entry: usize = match entry.parse() {
            Ok(n) => n,
            Err(_) => {
                ctx.send_channel("Very clever troll bro. Gimme number for entry plox.");
                return;
            }
        };
        Self::udlookup(sw.rest_as_slice(), entry, ctx);
    }
    fn ud(_this: &mut Plugin, arg: &str, ctx: Context) {
        Self::udlookup(arg, 0, ctx);
    }
    fn udlookup(arg: &str, entry: usize, ctx: Context) {
        if arg.is_empty() {
            ctx.send_channel("You need to search for something bro.");
            return;
        }
        match query(arg) {
            Ok(body) => {
                let json = match json::parse(&body) {
                    Ok(json) => json,
                    Err(e) => {
                        ctx.send_channel(&format!("Phailed parsing json ({})", e));
                        return;
                    }
                };
                let mut entry = match json["list"][entry]["definition"].as_str() {
                    Some(entry) => entry,
                    None => {
                        ctx.send_channel("ENGLISH, MOTHERFUCKER.");
                        return;
                    }
                };
                let too_large = entry.len() > 400;
                if too_large {
                    entry = &entry[..400];
                }
                for line in entry.lines() {
                    ctx.send_channel(line);
                }
                if too_large {
                    ctx.send_channel(&format!(
                        "http://www.urbandictionary.com/define.php?term={}",
                        arg.replace(' ', "%20").replace('&', "%26")
                    ));
                }
            }
            Err(e) => {
                ctx.send_channel(&format!("Error when uding: {}", e));
            }
        }
    }
}

impl Plugin for UdPlugin {
    fn new() -> Self {
        UdPlugin
    }
    fn register(&self, meta: &mut PluginMeta) {
        meta.command("ud", "Urban dictionary lookup", Self::ud);
        meta.command("udn", "Urban dictionary lookup (entry n)", Self::udn);
    }
}

plugin_export!(UdPlugin);
