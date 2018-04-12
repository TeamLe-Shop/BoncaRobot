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
        fn with_find_any<F: Fn(usize)>(haystack: &str, needles: &[&str], fun: F) {
            for needle in needles {
                if let Some(pos) = haystack.find(needle) {
                    fun(pos)
                }
            }
        }
        with_find_any(msg, &["http:", "https://"], |url_begin| {
            let from_url = &msg[url_begin..];
            let url_end = from_url
                .find(|c: char| c.is_whitespace() || c == '>')
                .unwrap_or(from_url.len());
            /*let title = get_title(&from_url[..url_end]);
            if !title.is_empty() {
                ctx.send_channel(&title);
            }*/
            ctx.send_channel(&format!("Extracted link \"{}\" from message", &from_url[..url_end]));
        });
    }
}

plugin_export!(LinkTitlePlugin);
