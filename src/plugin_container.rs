use libloading::Library;
use plugin_api::{Plugin, PluginMeta};
use std::error::Error;
use std::mem::ManuallyDrop;
use std::sync::{Arc, Mutex};

pub struct PluginContainer {
    pub plugin: ManuallyDrop<Arc<Mutex<Plugin>>>,
    pub meta: ManuallyDrop<PluginMeta>,
    lib: ManuallyDrop<Library>,
}

impl PluginContainer {
    pub fn load(name: &str) -> Result<Self, Box<Error>> {
        use std::env::consts::{DLL_PREFIX, DLL_SUFFIX};
        #[cfg(debug_assertions)]
        let root = "target/debug";
        #[cfg(not(debug_assertions))]
        let root = "target/release";
        let path = format!(
            "{dir}/{prefix}{name}{suffix}",
            dir = root,
            prefix = DLL_PREFIX,
            name = name,
            suffix = DLL_SUFFIX
        );
        let lib = Library::new(path)?;
        let plugin = {
            let init = unsafe { lib.get::<fn() -> Arc<Mutex<Plugin>>>(b"init")? };
            init()
        };
        let mut meta = PluginMeta::default();
        plugin.lock().unwrap().register(&mut meta);
        Ok(Self {
            plugin: ManuallyDrop::new(plugin),
            meta: ManuallyDrop::new(meta),
            lib: ManuallyDrop::new(lib),
        })
    }
}

impl Drop for PluginContainer {
    fn drop(&mut self) {
        unsafe {
            // First drop the plugin, as it depends on both meta and lib
            ManuallyDrop::drop(&mut self.plugin);
            // Drop meta, it depends on lib
            ManuallyDrop::drop(&mut self.meta);
            // Finally drop the lib
            ManuallyDrop::drop(&mut self.lib);
        }
    }
}
