extern crate http_request_common;
#[macro_use]
extern crate plugin_api;
extern crate scraper;

use plugin_api::prelude::*;

fn find_title(body: &str) -> String {
    use scraper::{Html, Selector};

    let html = Html::parse_document(body);
    let sel = Selector::parse("title").unwrap();
    let mut titles = html.select(&sel);
    match titles.next() {
        Some(title) => title.text().collect(),
        None => String::new(),
    }
}

fn get_title(link: &str) -> String {
    let (page, status) = match http_request_common::fetch_string(link, "") {
        Ok(page) => page,
        Err(e) => return format!("[error: {}]", e),
    };
    let title = find_title(&page);
    if status.is_success() {
        title
    } else {
        format!("[{}] {}", status, title)
    }
}

struct LinkTitlePlugin;

impl Plugin for LinkTitlePlugin {
    fn new() -> Self {
        LinkTitlePlugin
    }
    fn channel_msg(&mut self, msg: &str, ctx: Context) {
        if let Some(url_begin) = msg.find("://") {
            let from_url = &msg[url_begin..];
            let url_end = from_url
                .find(|c: char| c.is_whitespace() || c == '>')
                .unwrap_or(from_url.len());
            let title = get_title(&from_url[..url_end]);
            if !title.is_empty() {
                ctx.send_channel(&title);
            }
        }
    }
}

plugin_export!(LinkTitlePlugin);
