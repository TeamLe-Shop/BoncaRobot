#[macro_use]
extern crate plugin_api;
extern crate url;

use plugin_api::prelude::*;

struct SearchFixPlugin;

impl Plugin for SearchFixPlugin {
    fn new() -> Self {
        SearchFixPlugin
    }
    fn channel_msg(&mut self,
                   irc: Arc<Irc>,
                   channel: Arc<Channel>,
                   _sender: Arc<ChannelUser>,
                   message: &str) {
        let beginning = "/l/?kh=-1&uddg=";
        if let Some(idx) = message.find(beginning) {
            let url = &message[idx + beginning.len()..];
            let decoded = url::percent_encoding::percent_decode(url.as_bytes())
                .decode_utf8()
                .unwrap_or_else(|_| "invalid url".into());
            let _ = irc.privmsg(channel.name(), &decoded);
        }
    }
}

plugin_export!(SearchFixPlugin);
