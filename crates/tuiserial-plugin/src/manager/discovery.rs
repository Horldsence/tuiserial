//! Plugin discovery — scans the plugin directory and loads found plugins.
//!
//! Handles `discover_and_load()` and `reload_all()`, which create a
//! `PluginRuntime` for each valid plugin directory found on disk.

use std::panic::{AssertUnwindSafe, catch_unwind};

use crate::runtime::PluginRuntime;
use crate::types::{FailedPlugin, PluginError};

use super::{PluginManager, extract_panic_message};

impl PluginManager {
    /// Scan plugin directory and load all discovered plugins.
    ///
    /// Each subdirectory containing a `plugin.ts` or `plugin.js` file
    /// is treated as a plugin. The directory name becomes the plugin name.
    /// Subdirectories under `disabled/` are skipped.
    pub fn discover_and_load(&mut self) -> Result<usize, PluginError> {
        if !self.plugin_dir.exists() {
            std::fs::create_dir_all(&self.plugin_dir)?;
        }

        // Unload existing plugins
        for plugin in &mut self.plugins {
            let _ = catch_unwind(AssertUnwindSafe(|| plugin.unload()));
        }
        self.plugins.clear();
        self.failed_plugins.clear();

        let mut loaded = 0;
        let entries = std::fs::read_dir(&self.plugin_dir)?;

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let dir_name = path.file_name().unwrap().to_string_lossy().to_string();

            if dir_name == "disabled" {
                continue;
            }

            for ext in &["ts", "js"] {
                let plugin_file = path.join(format!("plugin.{}", ext));
                if plugin_file.exists() {
                    match PluginRuntime::new(&dir_name, plugin_file, path.clone()) {
                        Ok(mut runtime) => {
                            match catch_unwind(AssertUnwindSafe(|| runtime.load())) {
                                Ok(Ok(())) => {
                                    loaded += 1;
                                    self.plugins.push(runtime);
                                }
                                Ok(Err(e)) => {
                                    let msg = format!("{}", e);
                                    self.load_errors
                                        .push(format!("Plugin '{}' load error: {}", dir_name, msg));
                                    self.failed_plugins.push(FailedPlugin {
                                        name: dir_name.clone(),
                                        error: msg,
                                    });
                                }
                                Err(panic_info) => {
                                    let msg = extract_panic_message(&panic_info);
                                    self.load_errors
                                        .push(format!("Plugin '{}' panicked: {}", dir_name, msg));
                                    self.failed_plugins.push(FailedPlugin {
                                        name: dir_name.clone(),
                                        error: msg,
                                    });
                                }
                            }
                        }
                        Err(e) => {
                            let msg = format!("{}", e);
                            self.load_errors
                                .push(format!("Plugin '{}' create error: {}", dir_name, msg));
                            self.failed_plugins.push(FailedPlugin {
                                name: dir_name.clone(),
                                error: msg,
                            });
                        }
                    }
                    break;
                }
            }
        }

        self.plugins.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(loaded)
    }

    /// Reload all plugins (re-scan directory).
    pub fn reload_all(&mut self) -> Result<usize, PluginError> {
        self.discover_and_load()
    }
}
