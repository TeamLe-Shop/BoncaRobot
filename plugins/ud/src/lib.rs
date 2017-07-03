extern crate hyper;
extern crate json;
#[macro_use]
extern crate plugin_api;

use plugin_api::prelude::*;
use std::error::Error;
use std::io::prelude::*;

pub fn query(query: &str) -> Result<String, Box<Error>> {
    let client = hyper::Client::new();

    let msg = format!("http://api.urbandictionary.com/v0/define?term={}", query);

    let mut res = client.get(&msg).send()?;
    if res.status != hyper::Ok {
        return Err("Something went wrong with the request".into());
    }
    let mut body = Vec::new();
    res.read_to_end(&mut body)?;
    Ok(String::from_utf8_lossy(&body).into_owned())
}

struct UdPlugin;

impl UdPlugin {
    fn ud(_this: &mut Plugin, arg: &str, ctx: Context) {
        if arg.is_empty() {
            let _ = ctx.irc
                .privmsg(ctx.channel.name(), "You need to search for something bro.");
            return;
        }
        match query(arg) {
            Ok(body) => {
                let json = match json::parse(&body) {
                    Ok(json) => json,
                    Err(e) => {
                        let _ = ctx.irc
                            .privmsg(ctx.channel.name(), &format!("Phailed parsing json ({})", e));
                        return;
                    }
                };
                let _ = ctx.irc.privmsg(
                    ctx.channel.name(),
                    json["list"][0]["definition"].as_str().unwrap(),
                );
            }
            Err(e) => {
                let _ = ctx.irc
                    .privmsg(ctx.channel.name(), &format!("Error when uding: {}", e));
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
