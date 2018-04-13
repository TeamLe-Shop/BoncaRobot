extern crate http_request_common;
#[macro_use]
extern crate plugin_api;
extern crate scraper;
extern crate titlefetch;
extern crate url;

use http_request_common::fetch_string;
use plugin_api::prelude::*;
use std::error::Error;
use titlefetch::get_title;

pub fn parse_first_result(body: &str) -> Result<String, Box<Error>> {
    use scraper::{Html, Selector};

    let html = Html::parse_document(body);
    let sel = Selector::parse("div.b_algoheader").unwrap();
    let mut results = html.select(&sel);
    let result = match results.next() {
        Some(result) => result,
        None => return Ok("Your Bing-Fu fails you.".into()),
    };
    let sel = Selector::parse("a").unwrap();
    let a = match result.select(&sel).next() {
        Some(a) => a,
        None => return Err("What the shit. No link in the result.".into()),
    };
    let href = a.value()
        .attr("href")
        .ok_or("<a> should have a href, but it doesn't")?;
    Ok(href.to_owned())
}

struct SearchPlugin;

impl SearchPlugin {
    fn search(_this: &mut Plugin, arg: &str, ctx: Context) {
        if arg.is_empty() {
            ctx.send_channel("You need to search for something bro.");
            return;
        }
        match fetch_string("https://www.bing.com/search?q=", arg) {
            Ok((body, status)) => {
                if status.is_success() {
                    match parse_first_result(&body) {
                        Ok(result) => {
                            ctx.send_channel(&result);
                            let title = get_title(&result);
                            ctx.send_channel(&title);
                        }
                        Err(e) => {
                            ctx.send_channel(&format!("Error: {}", e));
                        }
                    }
                } else {
                    ctx.send_channel(&format!("HTTP status: {}", status));
                }
            }
            Err(e) => {
                ctx.send_channel(&format!("Error when searching: {}", e));
            }
        }
    }
    fn ytsearch(_this: &mut Plugin, arg: &str, ctx: Context) {
        if arg.is_empty() {
            ctx.send_channel("FLAVA FLAVA FOR MY PEOPLE PEOPLE, COME ON KID, HERE COMES THE FINAL");
            return;
        }
        match fetch_string("https://www.youtube.com/results?search_query=", arg) {
            Ok((body, status)) => {
                if status.is_success() {
                    match extract_yt(&body) {
                        Ok(link) => {
                            let ytlink = format!("https://youtu.be/{}", link);
                            ctx.send_channel(&ytlink);
                            let title = get_title(&ytlink);
                            ctx.send_channel(&title);
                        }
                        Err(e) => ctx.send_channel(&format!("Error extracting: {}", e)),
                    }
                } else {
                    ctx.send_channel(&format!("HTTP status: {}", status))
                }
            }
            Err(e) => ctx.send_channel(&format!("Error when yting: {}", e)),
        }
    }
}

fn extract_yt(input: &str) -> Result<&str, Box<Error>> {
    let link = input.find("/watch?v=").ok_or("No yt link found")?;
    let from_watch = &input[link..];
    let quot = from_watch.find('"').ok_or("Unterminated link")?;
    Ok(input
        .get(link + 9..link + quot)
        .ok_or("Link slicing fail.")?)
}

impl Plugin for SearchPlugin {
    fn new() -> Self {
        SearchPlugin
    }
    fn register(&self, meta: &mut PluginMeta) {
        meta.command("search", "Bing search", Self::search);
        meta.command("ytsearch", "Jewtube search", Self::ytsearch);
    }
}

plugin_export!(SearchPlugin);
