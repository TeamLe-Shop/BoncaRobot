#[macro_use]
extern crate plugin_api;

use plugin_api::prelude::*;

struct TemplatePlugin;

impl Plugin for TemplatePlugin {
    fn new() -> Self {
        TemplatePlugin
    }
}

plugin_export!(TemplatePlugin);
