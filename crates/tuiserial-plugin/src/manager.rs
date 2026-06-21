//! Plugin manager — discovers, loads, and orchestrates all plugins.
//!
//! The manager scans a plugin directory, creates a `PluginRuntime` for each
//! discovered plugin, and manages the data pipeline (RX/TX processing) and
//! lifecycle events.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::PathBuf;

use tuiserial_core::{AppState, NotificationLevel, SerialConfig};

use crate::runtime::PluginRuntime;
use crate::status::FailedPlugin;
use crate::types::{PluginError, PluginResult};

/// Manages all loaded plugins and their lifecycles.
pub struct PluginManager {
    pub(crate) plugins: Vec<PluginRuntime>,
    pub(crate) plugin_dir: PathBuf,
    pub(crate) config_dir: PathBuf,
    pub(crate) load_errors: Vec<String>,
    pub(crate) failed_plugins: Vec<FailedPlugin>,
}

impl PluginManager {
    /// Create a new plugin manager.
    ///
    /// `plugin_dir` is the directory where plugins are stored, typically
    /// `~/.config/tuiserial/plugins/`.
    ///
    /// The parent of `plugin_dir` (i.e. `~/.config/tuiserial/`) is used as
    /// the config root for the registry cache and other shared data.
    pub fn new(plugin_dir: PathBuf) -> Self {
        let config_dir = plugin_dir
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .to_path_buf();
        Self {
            plugins: Vec::new(),
            plugin_dir,
            config_dir,
            load_errors: Vec::new(),
            failed_plugins: Vec::new(),
        }
    }

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
                                    let msg = panic_info
                                        .downcast_ref::<&str>()
                                        .map(|s| s.to_string())
                                        .or_else(|| panic_info.downcast_ref::<String>().cloned())
                                        .unwrap_or_else(|| "unknown panic".to_string());
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

    /// Drain accumulated load errors (for display in app notifications).
    pub fn drain_load_errors(&mut self) -> Vec<String> {
        std::mem::take(&mut self.load_errors)
    }

    // ── Lifecycle hooks ──────────────────────────────────────────

    /// Called when serial port connects.
    pub fn on_connect(&mut self, config: &SerialConfig) {
        for plugin in &mut self.plugins {
            if plugin.has_error || !plugin.hooks.on_connect {
                continue;
            }
            plugin.update_config(config);
            let _ = catch_unwind(AssertUnwindSafe(|| plugin.call_lifecycle_hook("onConnect")));
        }
    }

    /// Called when serial port disconnects.
    pub fn on_disconnect(&mut self) {
        for plugin in &mut self.plugins {
            if plugin.has_error || !plugin.hooks.on_disconnect {
                continue;
            }
            let _ = catch_unwind(AssertUnwindSafe(|| {
                plugin.call_lifecycle_hook("onDisconnect")
            }));
        }
    }

    /// Called before app exit — calls onUnload for all plugins.
    pub fn on_app_exit(&mut self) {
        for plugin in &mut self.plugins {
            let _ = catch_unwind(AssertUnwindSafe(|| plugin.unload()));
        }
        self.plugins.clear();
    }

    // ── Data pipeline ────────────────────────────────────────────

    /// Process received data through all plugins with onRx hooks.
    ///
    /// Each plugin's onRx is called in order. If a plugin returns
    /// `Modified`, the modified data is passed to the next plugin.
    /// If a plugin returns `Suppressed`, processing stops and the
    /// data is dropped.
    ///
    /// Returns `(final_data, suppressed)`.
    pub fn process_rx(&mut self, data: Vec<u8>, config: &SerialConfig) -> (Vec<u8>, bool) {
        self.process_pipeline("onRx", data, config)
    }

    /// Process outgoing data through all plugins with onTx hooks.
    ///
    /// Same pipeline semantics as `process_rx`.
    pub fn process_tx(&mut self, data: Vec<u8>, config: &SerialConfig) -> (Vec<u8>, bool) {
        self.process_pipeline("onTx", data, config)
    }

    /// Internal pipeline runner.
    fn process_pipeline(
        &mut self,
        hook_name: &str,
        mut data: Vec<u8>,
        config: &SerialConfig,
    ) -> (Vec<u8>, bool) {
        for plugin in &mut self.plugins {
            let has_hook = match hook_name {
                "onRx" => plugin.hooks.on_rx,
                "onTx" => plugin.hooks.on_tx,
                _ => false,
            };

            if plugin.has_error || !has_hook {
                continue;
            }

            plugin.update_config(config);

            match catch_unwind(AssertUnwindSafe(|| plugin.call_data_hook(hook_name, &data))) {
                Ok(PluginResult::PassThrough) => {}
                Ok(PluginResult::Modified(new_data)) => {
                    data = new_data;
                }
                Ok(PluginResult::Suppressed) => {
                    return (data, true);
                }
                Ok(PluginResult::Error(msg)) => {
                    plugin.has_error = true;
                    plugin.error_message = Some(msg.clone());
                    plugin.append_log(
                        NotificationLevel::Error,
                        format!("[plugin: {}] {}", plugin.name, msg),
                    );
                }
                Err(panic_info) => {
                    let msg = panic_info
                        .downcast_ref::<&str>()
                        .map(|s| s.to_string())
                        .or_else(|| panic_info.downcast_ref::<String>().cloned())
                        .unwrap_or_else(|| "unknown panic".to_string());
                    plugin.has_error = true;
                    plugin.error_message = Some(format!("panic in {}: {}", hook_name, msg));
                    plugin.append_log(
                        NotificationLevel::Error,
                        format!("[plugin: {}] panic in {}: {}", plugin.name, hook_name, msg),
                    );
                }
            }
        }

        (data, false)
    }

    // ── Log flushing ─────────────────────────────────────────────

    /// Drain accumulated log messages from all plugins and add them
    /// to the app's notification queue.
    pub fn flush_plugin_logs(&mut self, app: &mut AppState) {
        for plugin in &mut self.plugins {
            for (level, msg) in plugin.drain_log_messages() {
                match level {
                    NotificationLevel::Info => app.add_info(msg),
                    NotificationLevel::Warning => app.add_warning(msg),
                    NotificationLevel::Error => app.add_error(msg),
                    NotificationLevel::Success => app.add_success(msg),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_plugin_file(dir: &std::path::Path, name: &str, content: &str) -> std::path::PathBuf {
        let plugin_dir = dir.join(name);
        std::fs::create_dir_all(&plugin_dir).unwrap();
        let file_path = plugin_dir.join("plugin.js");
        let mut f = std::fs::File::create(&file_path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
        file_path
    }

    #[test]
    fn test_discover_no_hooks_plugin() {
        let tmp = TempDir::new().unwrap();
        create_plugin_file(tmp.path(), "empty", "var x = 1;");

        let mut manager = PluginManager::new(tmp.path().to_path_buf());
        let count = manager.discover_and_load().unwrap();
        assert_eq!(count, 1);
        assert_eq!(manager.plugins.len(), 1);
        assert!(manager.plugins[0].hooks.is_empty());
    }

    #[test]
    fn test_discover_with_hooks() {
        let tmp = TempDir::new().unwrap();
        create_plugin_file(
            tmp.path(),
            "my-plugin",
            r#"
            function onLoad() { tuiserial.log.info("loaded"); }
            function onRx(data) { return data; }
            "#,
        );

        let mut manager = PluginManager::new(tmp.path().to_path_buf());
        let count = manager.discover_and_load().unwrap();
        assert_eq!(count, 1);
        assert!(manager.plugins[0].hooks.on_load);
        assert!(manager.plugins[0].hooks.on_rx);
        assert!(!manager.plugins[0].hooks.on_tx);
    }

    #[test]
    fn test_rx_pipeline_passthrough() {
        let tmp = TempDir::new().unwrap();
        create_plugin_file(
            tmp.path(),
            "pass",
            r#"
            function onRx(data) { return null; }
            "#,
        );

        let mut manager = PluginManager::new(tmp.path().to_path_buf());
        manager.discover_and_load().unwrap();

        let original = vec![0x48, 0x65, 0x6C, 0x6C, 0x6F]; // "Hello"
        let (result, suppressed) = manager.process_rx(original.clone(), &SerialConfig::default());
        assert!(!suppressed);
        assert_eq!(result, original);
    }

    #[test]
    fn test_rx_pipeline_modify() {
        let tmp = TempDir::new().unwrap();
        create_plugin_file(
            tmp.path(),
            "mod",
            r#"
            function onRx(data) {
                var result = [];
                for (var i = 0; i < data.length; i++) { result.push(data[i]); }
                result.push(0);
                return result;
            }
            "#,
        );

        let mut manager = PluginManager::new(tmp.path().to_path_buf());
        manager.discover_and_load().unwrap();

        let original = vec![0x48, 0x65];
        let (result, suppressed) = manager.process_rx(original.clone(), &SerialConfig::default());
        assert!(!suppressed);
        assert_eq!(result, vec![0x48, 0x65, 0x00]);
    }

    #[test]
    fn test_rx_pipeline_suppress() {
        let tmp = TempDir::new().unwrap();
        create_plugin_file(
            tmp.path(),
            "drop",
            r#"
            function onRx(data) { return []; }
            "#,
        );

        let mut manager = PluginManager::new(tmp.path().to_path_buf());
        manager.discover_and_load().unwrap();

        let original = vec![0x48, 0x65];
        let (_result, suppressed) = manager.process_rx(original, &SerialConfig::default());
        assert!(suppressed);
    }

    #[test]
    fn test_empty_plugin_dir() {
        let tmp = TempDir::new().unwrap();
        let mut manager = PluginManager::new(tmp.path().to_path_buf());
        let count = manager.discover_and_load().unwrap();
        assert_eq!(count, 0);
    }
}
