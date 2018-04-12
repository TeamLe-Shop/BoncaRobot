#[macro_use]
extern crate plugin_api;
extern crate regex;
#[macro_use]
extern crate lazy_static;
extern crate titlefetch;

use plugin_api::prelude::*;
use titlefetch::get_title;

struct LinkTitlePlugin;

impl Plugin for LinkTitlePlugin {
    fn new() -> Self {
        LinkTitlePlugin
    }
    fn channel_msg(&mut self, msg: &str, ctx: Context) {
        use regex::Regex;
        lazy_static! {
            static ref RE: Regex = Regex::new(r#"https?://(www\.)?[-a-zA-Z0-9@:%._\+~#=]{2,256}\.[a-z]{2,6}\b([-a-zA-Z0-9@:%_\+.~#?&//=]*)"#).unwrap();
        }
        if let Some(cap) = RE.captures_iter(msg).next() {
            let title = get_title(&cap[0]);
            if !title.is_empty() {
                ctx.send_channel(&title);
            }
        }
    }
}

plugin_export!(LinkTitlePlugin);
