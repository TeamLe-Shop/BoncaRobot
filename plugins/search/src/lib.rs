extern crate hyper;
extern crate scraper;
extern crate url;
#[macro_use]
extern crate plugin_api;

use plugin_api::prelude::*;
use std::error::Error;
use std::io::prelude::*;

pub fn query_google(query: &str) -> Result<String, Box<Error>> {
    let client = hyper::Client::new();

    let msg = format!("http://www.google.com/search?q={}", query);

    let mut res = client.get(&msg).send()?;
    if res.status != hyper::Ok {
        return Err("Something went wrong with the request".into());
    }
    let mut body = Vec::new();
    res.read_to_end(&mut body)?;
    Ok(String::from_utf8_lossy(&body).into_owned())
}

const URLQ: &'static str = "/url?q=";

fn parse_urlq(urlq: &str) -> Result<&str, Box<Error>> {
    let begin = URLQ.len();
    let end = begin +
              urlq[begin..]
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
        let h3 = h3s.next()
            .ok_or("There should be a h3 class=\"r\", but there isn't")?;
        let sel = Selector::parse("a").unwrap();
        let a = h3.select(&sel)
            .next()
            .ok_or("There should be a <a>, but there isn't")?;
        let href = a.value()
            .attr("href")
            .ok_or("<a> should have a href, but it doesn't")?;
        let href = url::percent_encoding::percent_decode(href.as_bytes())
            .decode_utf8()?;
        if href.starts_with("/search?q=") {
            continue;
        }
        return Ok(parse_href(&href)?.to_owned());
    }
}

struct SearchPlugin;

impl SearchPlugin {
    fn search(_this: &mut Plugin, arg: &str, ctx: Context) {
        if arg.is_empty() {
            let _ = ctx.irc
                .privmsg(ctx.channel.name(), "You need to search for something bro.");
            return;
        }
        match query_google(arg) {
            Ok(body) => {

                match parse_first_result(&body) {
                    Ok(result) => {
                        let _ = ctx.irc.privmsg(ctx.channel.name(), &result);
                    }
                    Err(e) => {
                        let _ = ctx.irc
                            .privmsg(ctx.channel.name(), &format!("Error: {}", e));
                    }
                }
            }
            Err(e) => {
                let _ = ctx.irc
                    .privmsg(ctx.channel.name(), &format!("Error when googuring: {}", e));
            }
        }
    }
}

impl Plugin for SearchPlugin {
    fn new() -> Self {
        SearchPlugin
    }
    fn register(&self, meta: &mut PluginMeta) {
        meta.command("search", "Googuru search", Self::search);
    }
}

plugin_export!(SearchPlugin);
