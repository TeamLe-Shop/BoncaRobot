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
    let end = begin + urlq[begin..].find("&sa=").ok_or("Expected &sa= shit, but didn't find it.")?;
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
        let h3 = h3s.next().ok_or("There should be a h3 class=\"r\", but there isn't")?;
        let sel = Selector::parse("a").unwrap();
        let a = h3.select(&sel).next().ok_or("There should be a <a>, but there isn't")?;
        let href = a.value().attr("href").ok_or("<a> should have a href, but it doesn't")?;
        let href = url::percent_encoding::percent_decode(href.as_bytes()).decode_utf8()?;
        if href.starts_with("/search?q=") {
            continue;
        }
        return Ok(parse_href(&href)?.to_owned());
    }
}

#[test]
#[ignore]
fn test_parse_first_result_on_dump() {
    use std::fs::File;
    let mut f = File::open("../../dump.txt").unwrap();
    let mut body = String::new();
    f.read_to_string(&mut body).unwrap();
    println!("{:?}", parse_first_result(&body));
}

struct SearchPlugin;

impl Plugin for SearchPlugin {
    fn new() -> Self {
        SearchPlugin
    }
    fn channel_msg(&mut self,
                   irc: Arc<Irc>,
                   channel: Arc<Channel>,
                   _sender: Arc<ChannelUser>,
                   message: &str) {
        if message == "search" {
            let _ = irc.privmsg(channel.name(), "You need to search for something, retard.");
        }
        if message.starts_with("search ") {
            let wot = message[7..].trim();
            if wot.is_empty() {
                let _ = irc.privmsg(channel.name(), "Empty search? Impossible!");
            }
            match query_google(wot) {
                Ok(body) => {
                    use std::fs::File;

                    let path = "dump.txt";
                    let mut file = File::create(path).unwrap();
                    file.write_all(body.as_bytes()).unwrap();

                    match parse_first_result(&body) {
                        Ok(result) => {
                            let _ = irc.privmsg(channel.name(), &result);
                        }
                        Err(e) => {
                            let _ = irc.privmsg(channel.name(), &format!("Error: {}", e));
                        }
                    }
                }
                Err(e) => {
                    let _ = irc.privmsg(channel.name(), &format!("Error when googuring: {}", e));
                }
            }
        }
    }
}

plugin_export!(SearchPlugin);