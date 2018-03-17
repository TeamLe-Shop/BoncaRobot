extern crate http_request_common;
#[macro_use]
extern crate plugin_api;
extern crate scraper;
extern crate url;

use http_request_common::fetch_string;
use plugin_api::prelude::*;
use std::error::Error;

const URLQ: &str = "/url?q=";

fn parse_urlq(urlq: &str) -> Result<&str, Box<Error>> {
    let begin = URLQ.len();
    let end = begin
        + urlq[begin..]
            .find("&sa=")
            .ok_or("Expected &sa= shit, but didn't find it.")?;
    Ok(&urlq[begin..end])
}

fn parse_href(href: &str) -> Result<&str, Box<Error>> {
    if href.starts_with(URLQ) {
        parse_urlq(href)
    } else {
        Ok(href)
    }
}

pub fn parse_first_result(body: &str) -> Result<String, Box<Error>> {
    use scraper::{Html, Selector};

    let html = Html::parse_document(body);
    let sel = Selector::parse("h3.r").unwrap();
    let mut h3s = html.select(&sel);
    loop {
        let h3 = match h3s.next() {
            Some(h3) => h3,
            None => return Ok("Your Google-Fu fails you.".into()),
        };
        let sel = Selector::parse("a").unwrap();
        // Fucking bullshit instant answer boxes. Can't be bothered to parse them.
        // Just skip to next h3.r result.
        let a = match h3.select(&sel).next() {
            Some(a) => a,
            None => continue,
        };
        let href = a.value()
            .attr("href")
            .ok_or("<a> should have a href, but it doesn't")?;
        let href = url::percent_encoding::percent_decode(href.as_bytes()).decode_utf8()?;
        if href.starts_with("/search?q=") || href.contains("books.google.hu") {
            continue;
        }
        return Ok(parse_href(&href)?.to_owned());
    }
}

struct SearchPlugin;

impl SearchPlugin {
    fn search(_this: &mut Plugin, arg: &str, ctx: Context) {
        if arg.is_empty() {
            ctx.send_channel("You need to search for something bro.");
            return;
        }
        match fetch_string("http://www.google.com/search?q=", arg) {
            Ok((body, status)) => {
                if status.is_success() {
                    match parse_first_result(&body) {
                        Ok(result) => {
                            ctx.send_channel(&result);
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
                ctx.send_channel(&format!("Error when googuring: {}", e));
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
                        Ok(link) => ctx.send_channel(&format!("https://youtu.be/{}", link)),
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
        meta.command("search", "Googuru search", Self::search);
        meta.command("ytsearch", "Jewtube search", Self::ytsearch);
    }
}

plugin_export!(SearchPlugin);
