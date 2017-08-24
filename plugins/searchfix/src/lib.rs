#[macro_use]
extern crate plugin_api;
extern crate url;

use plugin_api::prelude::*;

struct SearchFixPlugin;

impl Plugin for SearchFixPlugin {
    fn new() -> Self {
        SearchFixPlugin
    }
    fn channel_msg(&mut self, msg: &str, ctx: Context) {
        let beginning = "/l/?kh=-1&uddg=";
        if let Some(idx) = msg.find(beginning) {
            let url_begin_index = idx + beginning.len();
            let url_end_index = msg[url_begin_index..]
                .find(" ")
                .map(|idx| url_begin_index + idx)
                .unwrap_or(msg.len());
            let url = &msg[url_begin_index..url_end_index];
            let decoded = url::percent_encoding::percent_decode(url.as_bytes())
                .decode_utf8()
                .unwrap_or_else(|_| "invalid url".into());
            ctx.send_channel(&decoded);
        }
    }
}

plugin_export!(SearchFixPlugin);
