#[macro_use]
extern crate plugin_api;
extern crate split_whitespace_rest;

use plugin_api::prelude::*;
use split_whitespace_rest::SplitWhitespace;
use std::collections::HashMap;

struct Message {
    sender: String,
    content: String,
}

struct TellPlugin {
    messages: HashMap<String, Message>,
}

impl TellPlugin {
    fn tell(this: &mut Plugin, opts: ParsedOpts, ctx: Context) {
        let arg = &opts.free.join(" ");
        let this: &mut TellPlugin = this.downcast_mut().unwrap();

        let mut sw = SplitWhitespace::new(arg);

        match sw.next() {
            Some(to) => {
                let msg = sw.rest_as_slice();
                if msg.is_empty() {
                    ctx.send_channel("NEED A MESSAGE.");
                }
                let msg = Message {
                    sender: (*ctx.sender.nickname()).clone(),
                    content: msg.to_owned(),
                };
                this.messages.insert(to.to_owned(), msg);
                ctx.send_channel(&format!(
                    "{}: I'll pass that on to {}",
                    ctx.sender.nickname(),
                    to
                ));
            }
            None => ctx.send_channel("NEED A RECIPIENT."),
        }
    }
}

impl Plugin for TellPlugin {
    fn new() -> Self {
        Self {
            messages: HashMap::new(),
        }
    }
    fn register(&self, meta: &mut PluginMeta) {
        meta.add_simple_command("tell", "Leave a message for someone", Self::tell);
    }
    fn channel_msg(&mut self, _msg: &str, ctx: Context) {
        let nick = ctx.sender.nickname();
        let mut remove = None;
        if let Some(msg) = self.messages.get(&*nick) {
            ctx.send_channel(&format!("{}: <{}>: {}", nick, msg.sender, msg.content));
            remove = Some(&*nick);
        }
        if let Some(remove) = remove {
            self.messages.remove(remove);
        }
    }
}

plugin_export!(TellPlugin);
