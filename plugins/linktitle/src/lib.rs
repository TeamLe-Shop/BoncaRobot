#[macro_use]
extern crate plugin_api;
extern crate regex;
extern crate titlefetch;

use plugin_api::prelude::*;
use regex::Regex;
use titlefetch::get_title;
const RE: &str =
    r#"https?://(www\.)?[-a-zA-Z0-9@:%._\+~#=]{2,256}\.[a-z]{2,6}\b([-a-zA-Z0-9@:%_\+.~#?&//=]*)"#;

struct LinkTitlePlugin {
    regex: Regex,
}

impl Plugin for LinkTitlePlugin {
    fn new() -> Self {
        Self {
            regex: Regex::new(RE).unwrap(),
        }
    }
    fn channel_msg(&mut self, msg: &str, ctx: Context) {
        if let Some(cap) = self.regex.captures_iter(msg).next() {
            let title = get_title(&cap[0]);
            if !title.is_empty() {
                ctx.send_channel(&title);
            }
        }
    }
}

plugin_export!(LinkTitlePlugin);
