#[macro_use]
extern crate plugin_api;
extern crate reqwest;
extern crate scraper;
extern crate url;

use plugin_api::prelude::*;
use std::error::Error;
use std::io::prelude::*;

pub fn fetch_page(link: &str) -> Result<String, Box<Error>> {
    let mut resp = reqwest::get(link)?;

    if !resp.status().is_success() {
        return Err("Something went wrong with the request".into());
    }

    let mut content = Vec::new();
    resp.read_to_end(&mut content)?;
    Ok(String::from_utf8_lossy(&content).into_owned())
}


fn find_title(body: &str) -> Result<String, Box<Error>> {
    use scraper::{Html, Selector};

    let html = Html::parse_document(body);
    let sel = Selector::parse("title").unwrap();
    let mut titles = html.select(&sel);
    let title = titles.next().ok_or("No title found")?;
    Ok(title.text().collect())
}

fn get_title(link: &str) -> String {
    let page = match fetch_page(link) {
        Ok(page) => page,
        Err(e) => return format!("[error: {}]", e),
    };
    match find_title(&page) {
        Ok(title) => title,
        Err(e) => format!("[error: {}", e),
    }
}

struct LinkTitlePlugin;

impl Plugin for LinkTitlePlugin {
    fn new() -> Self {
        LinkTitlePlugin
    }
    fn channel_msg(&mut self, msg: &str, ctx: Context) {
        for word in msg.split_whitespace() {
            if word.starts_with("http://") || word.starts_with("https://") {
                let title = get_title(word);
                ctx.send_channel(&title);
                // Stop after first link
                return;
            }
        }
    }
}

plugin_export!(LinkTitlePlugin);
