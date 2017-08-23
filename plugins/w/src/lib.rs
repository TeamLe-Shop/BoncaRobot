extern crate json;
#[macro_use]
extern crate plugin_api;
extern crate reqwest;

use plugin_api::prelude::*;
use std::error::Error;
use std::io::prelude::*;

// Encode query into URL format acceptable by wikipedia
fn wikiencode(query: &str) -> String {
    query.replace(' ', "%20").replace('&', "%26")
}

pub fn query(query: &str) -> Result<String, Box<Error>> {
    let query = wikiencode(query);
    eprintln!("{}", query);

    let msg = format!(
        "https://en.wikipedia.org/w/api.php?format=json\
         &action=query&prop=extracts&exintro&explaintext\
         &exchars=300&redirects&titles={}",
        query
    );

    let mut resp = reqwest::get(&msg)?;

    if !resp.status().is_success() {
        return Err("Something went wrong with the request".into());
    }

    let mut content = Vec::new();
    resp.read_to_end(&mut content)?;
    Ok(String::from_utf8_lossy(&content).into_owned())
}

struct WPlugin;

impl WPlugin {
    fn w(_this: &mut Plugin, arg: &str, ctx: Context) {
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
                let pages = &json["query"]["pages"];
                // Just grab first page
                let page = match pages.entries().nth(0) {
                    Some((_k, v)) => v,
                    None => {
                        let _ = ctx.irc.privmsg(ctx.channel.name(), "No wiki page found.");
                        return;
                    }
                };
                match page["extract"].as_str() {
                    Some(extract) => {
                        for line in extract.lines() {
                            let _ = ctx.irc.privmsg(ctx.channel.name(), line);
                        }
                        let article_name = match json["query"]["redirects"][0]["to"].as_str() {
                            Some(redirect) => redirect,
                            None => arg,
                        };
                        let encoded = wikiencode(article_name);
                        let url = format!("https://en.wikipedia.org/wiki/{}", encoded);

                        let _ = ctx.irc.privmsg(ctx.channel.name(), &url);
                    }
                    None => {
                        let _ = ctx.irc.privmsg(ctx.channel.name(), r#"¯\_(ツ)_/¯"#);
                        return;
                    }
                }
            }
            Err(e) => {
                let _ = ctx.irc
                    .privmsg(ctx.channel.name(), &format!("Error when wikiing: {}", e));
            }
        }
    }
}

impl Plugin for WPlugin {
    fn new() -> Self {
        WPlugin
    }
    fn register(&self, meta: &mut PluginMeta) {
        meta.command("w", "Spam short description of a wiki article", Self::w);
    }
}

plugin_export!(WPlugin);
