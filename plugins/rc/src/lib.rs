extern crate librc;
#[macro_use]
extern crate plugin_api;

use librc::calc::Calc;
use plugin_api::prelude::*;

struct CalcPlugin {
    calc: Calc,
}

impl CalcPlugin {
    fn rc(this: &mut Plugin, arg: &str, ctx: Context) {
        let this: &mut Self = this.downcast_mut().unwrap();
        let mut response = String::new();
        for expr in arg.split(';') {
            match this.calc.eval(expr) {
                Ok(num) => response.push_str(&num.to_string()),
                Err(e) => response.push_str(&e.to_string()),
            }
            response.push_str(", ");
            let _ = ctx.irc.privmsg(
                ctx.channel.name(),
                &format!("{}: {}", ctx.sender.nickname(), response),
            );
        }
    }
}

impl Plugin for CalcPlugin {
    fn new() -> Self {
        Self { calc: Calc::new() }
    }
    fn register(&self, meta: &mut PluginMeta) {
        meta.command(
            "rc",
            "Calculates shit with the epic RUSTY-CALCULATOR",
            Self::rc,
        );
    }
}

plugin_export!(CalcPlugin);
