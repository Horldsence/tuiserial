//! Plugin manager — discovers, loads, and orchestrates all plugins.
//!
//! The manager scans a plugin directory, creates a `PluginRuntime` for each
//! discovered plugin, and manages the data pipeline (RX/TX processing) and
//! lifecycle events.

use std::path::Path;
use std::path::PathBuf;

use tuiserial_core::{AppState, NotificationLevel};

pub(crate) mod discovery;
pub(crate) mod lifecycle;
pub(crate) mod pipeline;
pub(crate) mod recovery;

/// Extract a human-readable message from a `catch_unwind` error payload.
pub(crate) fn extract_panic_message(panic_info: &Box<dyn std::any::Any + Send>) -> String {
    panic_info
        .downcast_ref::<&str>()
        .map(|s| s.to_string())
        .or_else(|| panic_info.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "unknown panic".to_string())
}

/// Manages all loaded plugins and their lifecycles.
pub struct PluginManager {
    pub(crate) plugins: Vec<crate::runtime::PluginRuntime>,
    pub(crate) plugin_dir: PathBuf,
    pub(crate) config_dir: PathBuf,
    pub(crate) load_errors: Vec<String>,
    pub(crate) failed_plugins: Vec<crate::types::FailedPlugin>,
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
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();
        Self {
            plugins: Vec::new(),
            plugin_dir,
            config_dir,
            load_errors: Vec::new(),
            failed_plugins: Vec::new(),
        }
    }

    /// Drain accumulated load errors (for display in app notifications).
    pub fn drain_load_errors(&mut self) -> Vec<String> {
        std::mem::take(&mut self.load_errors)
    }

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

    /// Return a reference to the plugin directory path.
    pub fn plugin_dir(&self) -> &Path {
        &self.plugin_dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;
    use tuiserial_core::SerialConfig;

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
