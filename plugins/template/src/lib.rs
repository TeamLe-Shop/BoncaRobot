#[macro_use]
extern crate plugin_api;

use plugin_api::prelude::*;

struct TemplatePlugin;

impl Plugin for TemplatePlugin {
    fn new() -> Self {
        TemplatePlugin
    }
    fn channel_msg(&mut self, msg: &str, ctx: Context) {
        // Echo whatever back
        let _ = ctx.irc.privmsg(ctx.channel.name(),
                                &format!("{} said {}", ctx.sender.nickname(), msg));
    }
}

plugin_export!(TemplatePlugin);
