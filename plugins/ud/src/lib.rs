extern crate json;
#[macro_use]
extern crate plugin_api;
extern crate reqwest;

use plugin_api::prelude::*;
use std::error::Error;
use std::io::prelude::*;

struct SplitChunks<'a> {
    text: &'a str,
    size: usize,
}

impl<'a> SplitChunks<'a> {
    fn new(text: &'a str, size: usize) -> Self {
        Self { text, size }
    }
}

#[test]
fn test_split_chunks() {
    let text = "I am a cool guy lol";
    let mut chunks = SplitChunks::new(text, 5);
    assert_eq!(chunks.next(), Some("I am "));
    assert_eq!(chunks.next(), Some("a coo"));
    assert_eq!(chunks.next(), Some("l guy"));
    assert_eq!(chunks.next(), Some(" lol"));
    assert_eq!(chunks.next(), None);
}

impl<'a> Iterator for SplitChunks<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<&'a str> {
        let mut cursor = self.size;
        while !self.text.is_char_boundary(cursor) {
            cursor -= 1;
        }
        let chunk = &self.text[..cursor];
        if chunk.is_empty() {
            return None;
        }
        self.text = &self.text[cursor..];
        Some(chunk)
    }
}

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
                let entry = match json["list"][0]["definition"].as_str() {
                    Some(entry) => entry,
                    None => {
                        let _ = ctx.irc
                            .privmsg(ctx.channel.name(), "ENGLISH, MOTHERFUCKER.");
                        return;
                    }
                };
                for line in entry.lines() {
                    // Spit out text in chunks of 400
                    for chunk in SplitChunks::new(line, 400) {
                        let _ = ctx.irc.privmsg(ctx.channel.name(), chunk);
                    }
                }
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
