#[macro_use]
extern crate plugin_api;

use plugin_api::prelude::*;

struct TemplatePlugin;

impl Plugin for TemplatePlugin {
    fn new() -> Self {
        TemplatePlugin
    }
    fn channel_msg(&mut self,
                   irc: Arc<Irc>,
                   channel: Arc<Channel>,
                   sender: Arc<ChannelUser>,
                   message: &str) {
        // Echo whatever back
        let _ = irc.privmsg(channel.name(),
                            &format!("{} said {}", sender.nickname(), message));
    }
}

plugin_export!(TemplatePlugin);
