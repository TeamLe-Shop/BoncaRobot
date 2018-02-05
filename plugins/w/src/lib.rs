extern crate http_request_common;
extern crate json;
#[macro_use]
extern crate plugin_api;

use http_request_common::fetch_string;
use plugin_api::prelude::*;
use std::error::Error;

// Encode query into URL format acceptable by wikipedia
fn wikiencode(query: &str) -> String {
    query.replace(' ', "%20").replace('&', "%26")
}

fn query_opensearch(what: &str) -> Result<String, Box<Error>> {
    fetch_string(
        "https://en.wikipedia.org/w/api.php?action=opensearch&format=json&search=",
        what,
    )
}

fn query_wp(what: &str) -> Result<String, Box<Error>> {
    let base = "https://en.wikipedia.org/w/api.php?format=json
                &action=query&prop=extracts&exintro&explaintext\
                &exchars=385&redirects&titles=";
    fetch_string(base, &what)
}

fn process_wp_result(result: Result<String, Box<Error>>, article_name: &str, ctx: Context) {
    match result {
        Ok(body) => {
            let json = match json::parse(&body) {
                Ok(json) => json,
                Err(e) => {
                    ctx.send_channel(&format!("Phailed parsing json ({})", e));
                    return;
                }
            };
            let pages = &json["query"]["pages"];
            // Just grab first page
            let page = match pages.entries().nth(0) {
                Some((_k, v)) => v,
                None => {
                    ctx.send_channel("No wiki page found.");
                    return;
                }
            };
            match page["extract"].as_str() {
                Some(extract) => {
                    for line in extract.lines() {
                        ctx.send_channel(line);
                    }
                    let encoded = wikiencode(article_name);
                    let url = format!("https://en.wikipedia.org/wiki/{}", encoded);

                    ctx.send_channel(&url);
                }
                None => {
                    ctx.send_channel(
                        "YOU BETRAYED ME, OPENSEARCH. HOW COULD YOU DARE? HOW COULD YOU DAAAARE!?",
                    );
                    return;
                }
            }
        }
        Err(e) => {
            ctx.send_channel(&format!("Error when wikiing: {}", e));
        }
    }
}

struct WPlugin;

impl WPlugin {
    fn w(_this: &mut Plugin, arg: &str, ctx: Context) {
        if arg.is_empty() {
            ctx.send_channel("You need to search for something bro.");
            return;
        }
        match query_opensearch(arg) {
            Ok(body) => {
                let json = match json::parse(&body) {
                    Ok(json) => json,
                    Err(e) => {
                        ctx.send_channel(&format!("Phailed parsing json ({})", e));
                        return;
                    }
                };
                match json[1][0].as_str() {
                    Some(name) => {
                        let wp_result = query_wp(name);
                        process_wp_result(wp_result, name, ctx);
                    }
                    None => {
                        ctx.send_channel(r#"¯\_(ツ)_/¯"#);
                        return;
                    }
                }
            }
            Err(e) => ctx.send_channel(&format!("Error when wikiing: {}", e)),
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
