extern crate librc;
#[macro_use]
extern crate plugin_api;

use librc::calc::Calc;
use plugin_api::prelude::*;

struct CalcPlugin {
    calc: Calc,
}


impl Plugin for CalcPlugin {
    fn new() -> Self {
        Self { calc: Calc::new() }
    }
    fn channel_msg(&mut self, msg: &str, ctx: Context) {
        if msg.starts_with("rc ") {
            let wot = &msg[3..];
            let mut response = String::new();
            for expr in wot.split(';') {
                match self.calc.eval(expr) {
                    Ok(num) => response.push_str(&num.to_string()),
                    Err(e) => response.push_str(&e.to_string()),
                }
                response.push_str(", ");
                let _ = ctx.irc.privmsg(ctx.channel.name(),
                                        &format!("{}: {}", ctx.sender.nickname(), response));
            }
        }
    }
}

plugin_export!(CalcPlugin);
