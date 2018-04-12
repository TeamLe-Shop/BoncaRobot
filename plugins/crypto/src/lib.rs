extern crate http_request_common;
extern crate json;
#[macro_use]
extern crate plugin_api;

use plugin_api::prelude::*;

struct CryptoPlugin;

impl CryptoPlugin {
    fn crypto(_this: &mut Plugin, arg: &str, ctx: Context) {
        let text = match http_request_common::fetch_string_on_success(
            "https://api.coinmarketcap.com/v1/ticker/?limit=0",
            "",
        ) {
            Ok(text) => text,
            Err(_) => {
                ctx.send_channel("Fetch phail.");
                return;
            }
        };
        let json = match json::parse(&text) {
            Ok(json) => json,
            Err(_) => {
                ctx.send_channel("Json fuckup.");
                return;
            }
        };
        for entry in json.members() {
            if entry["id"] == arg {
                ctx.send_channel("Wow, it exists, ok");
                return;
            }
        }
        ctx.send_channel("Go make your own cryptocurrency");
    }
}

impl Plugin for CryptoPlugin {
    fn new() -> Self {
        CryptoPlugin
    }
    fn register(&self, meta: &mut PluginMeta) {
        meta.command(
            "crypto",
            "Look up cryptocurrency prices or some shit",
            Self::crypto,
        );
    }
}

plugin_export!(CryptoPlugin);
