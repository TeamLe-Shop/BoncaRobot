extern crate json;
#[macro_use]
extern crate plugin_api;
extern crate reqwest;

use plugin_api::prelude::*;
use std::error::Error;
use std::io::prelude::*;

pub fn query(query: &str) -> Result<String, Box<Error>> {
    let msg = format!("http://api.urbandictionary.com/v0/define?term={}", query);

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
    fn ud(_this: &mut Plugin, arg: &str, ctx: Context) {
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
                let entry = match json["list"][0]["definition"].as_str() {
                    Some(entry) => entry,
                    None => {
                        ctx.send_channel("ENGLISH, MOTHERFUCKER.");
                        return;
                    }
                };
                for line in entry.lines() {
                    ctx.send_channel(line);
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
    }
}

plugin_export!(UdPlugin);
