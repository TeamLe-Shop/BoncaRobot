extern crate http_request_common;
extern crate json;
#[macro_use]
extern crate plugin_api;

use json::JsonValue;
use plugin_api::prelude::*;
use std::error::Error;

pub fn query(query: &str) -> Result<String, Box<Error>> {
    http_request_common::fetch_string_on_success(
        "http://api.urbandictionary.com/v0/define?term=",
        query,
    )
}

struct UdPlugin;

impl UdPlugin {
    fn ud(_this: &mut Plugin, opts: ParsedOpts, ctx: Context) {
        let contains = opts.get_or_empty("contains");
        let fuck = opts.get_or_empty("fuck");
        let term = opts.free.join(" ");
        let n = opts
            .get_or_empty("number")
            .get(0)
            .map(|arg| arg.parse::<u8>().unwrap_or(0))
            .unwrap_or(0);
        ctx.send_channel(&format!(
            "contains: {:?}, fuck: {:?} n: {:?}",
            contains, fuck, n
        ));
        with_json(&term, ctx, |json| {
            let mut i = 0;
            let entries = &json["list"];
            for v in entries.members() {
                if !opts.given("loose") {
                    if v["word"].as_str().unwrap().to_lowercase() != term.to_lowercase() {
                        continue;
                    }
                }
                if let Some(def) = v["definition"].as_str() {
                    let mut all_contains_satisfied = true;
                    for c in contains {
                        if !def.to_lowercase().contains(&c.to_lowercase())
                            && !v["example"]
                                .as_str()
                                .unwrap_or("")
                                .to_lowercase()
                                .contains(&c.to_lowercase())
                        {
                            all_contains_satisfied = false;
                        }
                    }
                    let mut any_contains_fuck = false;
                    for f in fuck {
                        if def.to_lowercase().contains(&f.to_lowercase())
                            || v["example"]
                                .as_str()
                                .unwrap_or("")
                                .to_lowercase()
                                .contains(&f.to_lowercase())
                        {
                            any_contains_fuck = true;
                        }
                    }

                    if all_contains_satisfied && !any_contains_fuck {
                        if i == n {
                            display_def(
                                &format!("{}: {}", v["word"].as_str().unwrap_or("?"), def),
                                v["example"].as_str(),
                                &term,
                                ctx,
                            );
                            return;
                        }
                        i += 1;
                    }
                }
            }
            ctx.send_channel("ENGLISH MOTHERFUCKER, DO YOU SPEAK IT?");
        });
    }
}

fn with_json<F: Fn(JsonValue)>(arg: &str, ctx: Context, fun: F) {
    if arg.is_empty() {
        ctx.send_channel("You need to search for something bro.");
        return;
    }
    match query(arg) {
        Ok(body) => match json::parse(&body) {
            Ok(json) => fun(json),
            Err(e) => {
                ctx.send_channel(&format!("Phailed parsing json ({})", e));
                return;
            }
        },
        Err(e) => {
            ctx.send_channel(&format!("Error when uding: {}", e));
        }
    }
}

fn display_def(mut def: &str, example: Option<&str>, arg: &str, ctx: Context) {
    let too_large = def.len() > 400;
    let mut cutoff = 400;
    while !def.is_char_boundary(cutoff) {
        cutoff -= 1;
    }
    if too_large {
        def = &def[..cutoff];
    }
    for line in def.lines() {
        ctx.send_channel(line);
    }
    if let Some(example) = example {
        for line in example.lines() {
            ctx.send_channel(&format!("> {}", line));
        }
    }
    if too_large {
        ctx.send_channel(&format!(
            "http://www.urbandictionary.com/define.php?term={}",
            arg.replace(' ', "%20").replace('&', "%26")
        ));
    }
}

impl Plugin for UdPlugin {
    fn new() -> Self {
        UdPlugin
    }
    fn register(&self, meta: &mut PluginMeta) {
        let command = Command::new("ud", "Urban dictionary lookup", Self::ud)
            .opt('n', "number", "Get entry number n", true)
            .opt(
                'c',
                "contains",
                "Filter entries to those containing certain words",
                true,
            )
            .opt(
                'f',
                "fuck",
                "Filter entries to those lacking certain words",
                true,
            )
            .opt('l', "loose", "Allow non-exact entries", false);
        meta.add_command(command);
    }
}

plugin_export!(UdPlugin);
