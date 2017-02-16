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
        let uddg = "uddg=";
        if let Some(idx) = message.find(uddg) {
            let url = &message[idx + uddg.len()..];
            let decoded = url::percent_encoding::percent_decode(url.as_bytes())
                .decode_utf8()
                .unwrap_or_else(|_| "invalid url".into());
            let _ = irc.privmsg(channel.name(), &decoded);
        }
    }
}

plugin_export!(SearchFixPlugin);
