extern crate json;
#[macro_use]
extern crate plugin_api;
extern crate reqwest;
extern crate split_whitespace_rest;

use json::JsonValue;
use plugin_api::prelude::*;
use split_whitespace_rest::SplitWhitespace;
use std::error::Error;
use std::io::prelude::*;

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
        udlookup(sw.rest_as_slice(), entry, ctx);
    }
    fn ud(_this: &mut Plugin, arg: &str, ctx: Context) {
        udlookup(arg, 0, ctx);
    }
    fn udc(_this: &mut Plugin, arg: &str, ctx: Context) {
        let mut sw = SplitWhitespace::new(arg);
        let needle = match sw.next() {
            Some(en) => en,
            None => {
                ctx.send_channel("Need needle, bro.");
                return;
            }
        };
        ud_lookup_matching(sw.rest_as_slice(), needle, ctx);
    }
}

fn with_json<F: Fn(JsonValue)>(arg: &str, ctx: Context, fun: F) {
    if arg.is_empty() {
        ctx.send_channel("You need to search for something bro.");
        return;
    }
    match query(arg) {
        Ok(body) => match json::parse(&body) {
            Ok(json) => fun(json),
            Err(e) => {
                ctx.send_channel(&format!("Phailed parsing json ({})", e));
                return;
            }
        },
        Err(e) => {
            ctx.send_channel(&format!("Error when uding: {}", e));
        }
    }
}

fn udlookup(arg: &str, index: usize, ctx: Context) {
    with_json(arg, ctx, |json| {
        let entry = &json["list"][index];
        match entry["definition"].as_str() {
            Some(def) => display_def(def, entry["example"].as_str(), arg, ctx),
            None => {
                ctx.send_channel("ENGLISH, MOTHERFUCKER.");
                return;
            }
        };
    });
}

fn ud_lookup_matching(arg: &str, needle: &str, ctx: Context) {
    with_json(arg, ctx, |json| {
        let entries = &json["list"];
        for v in entries.members() {
            if let Some(def) = v["definition"].as_str() {
                if def.to_lowercase().contains(&needle.to_lowercase()) {
                    display_def(def, v["example"].as_str(), arg, ctx);
                    return;
                }
            }
        }
        ctx.send_channel("No such luck, bro.");
    })
}

fn display_def(mut def: &str, example: Option<&str>, arg: &str, ctx: Context) {
    let too_large = def.len() > 400;
    if too_large {
        def = &def[..400];
    }
    for line in def.lines() {
        ctx.send_channel(line);
    }
    if let Some(example) = example {
        for line in example.lines() {
            ctx.send_channel(&format!("> {}", line));
        }
    }
    if too_large {
        ctx.send_channel(&format!(
            "http://www.urbandictionary.com/define.php?term={}",
            arg.replace(' ', "%20").replace('&', "%26")
        ));
    }
}

impl Plugin for UdPlugin {
    fn new() -> Self {
        UdPlugin
    }
    fn register(&self, meta: &mut PluginMeta) {
        meta.command("ud", "Urban dictionary lookup", Self::ud);
        meta.command("udn", "Urban dictionary lookup (entry n)", Self::udn);
        meta.command("udc", "Search urban haystack for needle", Self::udc);
    }
}

plugin_export!(UdPlugin);
