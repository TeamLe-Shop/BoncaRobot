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
        for word in msg.split_whitespace() {
            if word.starts_with("http://") || word.starts_with("https://") {
                let title = get_title(word);
                if !title.is_empty() {
                    ctx.send_channel(&title);
                }
                // Stop after first link
                return;
            }
        }
    }
}

plugin_export!(LinkTitlePlugin);
