//! Plugin manager — discovers, loads, and orchestrates all plugins.
//!
//! The manager scans a plugin directory, creates a `PluginRuntime` for each
//! discovered plugin, and manages the data pipeline (RX/TX processing) and
//! lifecycle events.

use std::path::{Path, PathBuf};

use tuiserial_core::{
    AppState, NotificationLevel, PluginLoadState, PluginLoadStatus, PluginMetadataSimple,
    SerialConfig,
};

use crate::git;
use crate::registry;
use crate::runtime::PluginRuntime;
use crate::types::{
    PluginError, PluginInfo, PluginMetadata, PluginResult, PluginUpdateStatus, RegistryEntry,
};

/// Manages all loaded plugins and their lifecycles.
pub struct PluginManager {
    plugins: Vec<PluginRuntime>,
    plugin_dir: PathBuf,
    config_dir: PathBuf,
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
        }
    }

    /// Scan plugin directory and load all discovered plugins.
    ///
    /// Each subdirectory containing a `plugin.ts` or `plugin.js` file
    /// is treated as a plugin. The directory name becomes the plugin name.
    /// Subdirectories under `disabled/` are skipped.
    pub fn discover_and_load(&mut self) -> Result<usize, PluginError> {
        // Ensure plugin directory exists
        if !self.plugin_dir.exists() {
            std::fs::create_dir_all(&self.plugin_dir)?;
        }

        // Unload existing plugins
        for plugin in &mut self.plugins {
            plugin.unload();
        }
        self.plugins.clear();

        let mut loaded = 0;
        let entries = std::fs::read_dir(&self.plugin_dir)?;

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let dir_name = path.file_name().unwrap().to_string_lossy().to_string();

            // Skip the disabled directory
            if dir_name == "disabled" {
                continue;
            }

            // Look for plugin.ts or plugin.js in the directory
            for ext in &["ts", "js"] {
                let plugin_file = path.join(format!("plugin.{}", ext));
                if plugin_file.exists() {
                    match PluginRuntime::new(&dir_name, plugin_file, path.clone()) {
                        Ok(mut runtime) => {
                            match runtime.load() {
                                Ok(()) => {
                                    loaded += 1;
                                    self.plugins.push(runtime);
                                }
                                Err(e) => {
                                    // Log error but continue loading other plugins
                                    eprintln!(
                                        "Failed to load plugin '{}': {}",
                                        dir_name, e
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!(
                                "Failed to create runtime for '{}': {}",
                                dir_name, e
                            );
                        }
                    }
                    break; // Found a plugin file, don't check other extensions
                }
            }
        }

        // Sort plugins by name for deterministic ordering
        self.plugins.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(loaded)
    }

    /// Reload all plugins (re-scan directory).
    pub fn reload_all(&mut self) -> Result<usize, PluginError> {
        self.discover_and_load()
    }

    /// Get detailed plugin statuses for the plugin manager modal.
    ///
    /// Returns a `PluginLoadStatus` for every plugin directory found
    /// (loaded, error, and disabled plugins), with hook info, error
    /// messages, and metadata.
    pub fn get_plugin_statuses(&self) -> Vec<PluginLoadStatus> {
        let mut statuses: Vec<PluginLoadStatus> = Vec::new();

        // Collect loaded plugins
        for p in &self.plugins {
            let state = if p.has_error {
                PluginLoadState::Error
            } else {
                PluginLoadState::Loaded
            };
            statuses.push(PluginLoadStatus {
                name: p.name.clone(),
                state,
                has_rx_hook: p.hooks.on_rx,
                has_tx_hook: p.hooks.on_tx,
                has_connect_hook: p.hooks.on_connect,
                has_disconnect_hook: p.hooks.on_disconnect,
                error_message: p.error_message.clone(),
                metadata: self.read_metadata(&p.name).map(|m| PluginMetadataSimple {
                    version: m.version,
                    description: m.description,
                    author: m.author,
                }),
            });
        }

        // Also scan for disabled plugins (in disabled/ subdirectory)
        let disabled_dir = self.plugin_dir.join("disabled");
        if disabled_dir.exists()
            && let Ok(entries) = std::fs::read_dir(&disabled_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if !path.is_dir() {
                        continue;
                    }
                    let name = path
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .to_string();
                    // Check if it has a plugin.ts or plugin.js
                    let has_file =
                        path.join("plugin.ts").exists() || path.join("plugin.js").exists();
                    if has_file {
                        statuses.push(PluginLoadStatus {
                            name,
                            state: PluginLoadState::Disabled,
                            has_rx_hook: false,
                            has_tx_hook: false,
                            has_connect_hook: false,
                            has_disconnect_hook: false,
                            error_message: None,
                            metadata: None,
                        });
                    }
                }
            }

        // Sort by name
        statuses.sort_by(|a, b| a.name.cmp(&b.name));
        statuses
    }

    /// Get list of plugin info for UI display.
    pub fn list_plugins(&self) -> Vec<PluginInfo> {
        self.plugins
            .iter()
            .map(|p| PluginInfo {
                name: p.name.clone(),
                enabled: !p.has_error,
                hooks: p.hooks.clone(),
                has_error: p.has_error,
                error_message: p.error_message.clone(),
            })
            .collect()
    }

    /// Return a reference to the plugin directory path.
    pub fn plugin_dir(&self) -> &Path {
        &self.plugin_dir
    }

    /// Read `plugin.json` from a plugin directory.
    pub fn read_metadata(&self, plugin_name: &str) -> Option<PluginMetadata> {
        let path = self.plugin_dir.join(plugin_name).join("plugin.json");
        let bytes = std::fs::read(path).ok()?;
        serde_json::from_slice(&bytes).ok()
    }

    /// Try to infer the GitHub repo URL for a plugin.
    ///
    /// Checks `plugin.json` first, then git remote origin.
    pub fn infer_repo(&self, plugin_name: &str) -> Option<String> {
        if let Some(meta) = self.read_metadata(plugin_name)
            && let Some(repo) = meta.repo {
                return Some(repo);
        }
        let dir = self.plugin_dir.join(plugin_name);
        if git::is_git_repo(&dir) {
            return git::git_remote_url(&dir).ok();
        }
        None
    }

    // ── Plugin management ────────────────────────────────────────

    /// Clone (or fetch) a GitHub repository into a directory.
    ///
    /// If the target already has a `.git` directory it fetches and
    /// fast-forward pulls. Otherwise it clones.
    ///
    /// Returns the plugin name (derived from the repo URL).
    pub fn install_plugin(&self, repo_url: &str) -> Result<String, PluginError> {
        let name = repo_name_from_url(repo_url);
        let target = self.plugin_dir.join(&name);

        if target.exists() {
            return Err(PluginError::Git(format!(
                "plugin directory '{}' already exists",
                name
            )));
        }

        git::git_clone(repo_url, &target)?;
        Ok(name)
    }

    /// Return the path to the registry cache directory.
    ///
    /// This is `{config_dir}/registry-cache/` — a sparse checkout of the
    /// default plugin monorepo.
    pub fn registry_cache_dir(&self) -> PathBuf {
        self.config_dir.join("registry-cache")
    }

    /// Ensure the registry monorepo is cloned or up-to-date in the cache.
    ///
    /// On first call, clones the default registry repo with
    /// `--filter=blob:none --sparse`. On later calls, fetches and
    /// fast-forward pulls. After this returns successfully,
    /// `get_registry()` and `install_plugin_from_cache()` will work.
    pub fn fetch_registry(&self) -> Result<(), PluginError> {
        let cache = self.registry_cache_dir();

        if cache.join(".git").exists() {
            git::git_fetch(&cache)?;
            git::git_pull_ff(&cache)?;
        } else {
            let parent = cache.parent().unwrap_or(Path::new("."));
            std::fs::create_dir_all(parent)?;
            // If the directory exists but isn't a git repo, remove it
            if cache.exists() {
                std::fs::remove_dir_all(&cache)?;
            }
            git::git_clone_sparse(registry::DEFAULT_REGISTRY_REPO, &cache)?;
        }

        Ok(())
    }

    /// Get the list of available plugins from the registry.
    ///
    /// Fetches the registry first (clone or pull), then scans using
    /// git metadata so it works with sparse/partial clones.
    pub fn get_registry(&self) -> Result<Vec<RegistryEntry>, PluginError> {
        self.fetch_registry()?;
        let cache = self.registry_cache_dir();
        Ok(registry::scan_registry_git(&cache))
    }

    /// Install a plugin by copying it from the registry cache.
    ///
    /// Uses sparse-checkout to download only the requested plugin's files
    /// from the monorepo, then copies the folder to the plugins directory.
    pub fn install_plugin_from_cache(&self, name: &str) -> Result<(), PluginError> {
        let target = self.plugin_dir.join(name);
        if target.exists() {
            return Err(PluginError::Git(format!(
                "plugin directory '{}' already exists",
                name
            )));
        }

        let cache = self.registry_cache_dir();
        if !cache.join(".git").exists() {
            return Err(PluginError::Git(
                "registry cache not found — fetch it first".into(),
            ));
        }

        // Sparse-checkout the plugin folder and checkout to download its blobs
        git::git_sparse_checkout_set(&cache, name)?;
        git::git_checkout(&cache)?;

        let source = cache.join(name);
        if !source.exists() {
            return Err(PluginError::Git(format!(
                "plugin '{}' not found in registry",
                name
            )));
        }

        // Copy the plugin folder into the plugins directory
        copy_dir_recursive(&source, &target)?;

        Ok(())
    }

    /// Check all installed plugins for updates from their git remotes.
    ///
    /// Returns a list of `PluginUpdateStatus` for plugins that have remotes.
    pub fn check_updates(&self) -> Result<Vec<PluginUpdateStatus>, PluginError> {
        let mut results = Vec::new();

        for entry in std::fs::read_dir(&self.plugin_dir)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let dir_name = path.file_name().unwrap().to_string_lossy();
            if dir_name == "disabled" || !git::is_git_repo(&path) {
                continue;
            }

            let repo = match git::git_remote_url(&path) {
                Ok(r) => r,
                Err(_) => continue,
            };

            // Fetch to get latest remote status
            if let Err(_e) = git::git_fetch(&path) {
                results.push(PluginUpdateStatus {
                    name: dir_name.to_string(),
                    repo,
                    current_commit: String::new(),
                    latest_commit: String::new(),
                    has_update: false,
                });
                continue;
            }

            let current = git::git_head_commit(&path).unwrap_or_default();
            let latest = git::git_remote_commit(&path).unwrap_or_default();
            let behind = git::git_is_behind(&path).unwrap_or(false);

            results.push(PluginUpdateStatus {
                name: dir_name.to_string(),
                repo,
                current_commit: short_hash(&current),
                latest_commit: short_hash(&latest),
                has_update: behind,
            });
        }

        Ok(results)
    }

    /// Update a single plugin via `git pull --ff-only`.
    pub fn update_plugin(&self, name: &str) -> Result<(), PluginError> {
        let dir = self.plugin_dir.join(name);
        if !dir.exists() {
            return Err(PluginError::Git(format!("plugin '{}' not found", name)));
        }
        if !git::is_git_repo(&dir) {
            return Err(PluginError::Git(format!(
                "'{}' is not a git repository",
                name
            )));
        }
        git::git_fetch(&dir)?;
        git::git_pull_ff(&dir)?;
        Ok(())
    }

    /// Update all plugins that have git remotes.
    ///
    /// Returns `(updated_count, errors)`.
    pub fn update_all(&self) -> (usize, Vec<String>) {
        let mut updated = 0usize;
        let mut errors = Vec::new();

        let entries = match std::fs::read_dir(&self.plugin_dir) {
            Ok(e) => e,
            Err(e) => {
                errors.push(format!("failed to read plugin dir: {}", e));
                return (updated, errors);
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let name = path.file_name().unwrap().to_string_lossy();
            if name == "disabled" || !git::is_git_repo(&path) {
                continue;
            }

            match self.update_plugin(&name) {
                Ok(()) => updated += 1,
                Err(e) => errors.push(format!("{}: {}", name, e)),
            }
        }

        (updated, errors)
    }

    /// Remove a plugin by deleting its directory.
    pub fn remove_plugin(&self, name: &str) -> Result<(), PluginError> {
        let dir = self.plugin_dir.join(name);
        if !dir.exists() {
            return Err(PluginError::Git(format!("plugin '{}' not found", name)));
        }
        std::fs::remove_dir_all(&dir)?;
        Ok(())
    }

    // ── Lifecycle hooks ──────────────────────────────────────────

    /// Called when serial port connects.
    pub fn on_connect(&mut self, config: &SerialConfig) {
        for plugin in &mut self.plugins {
            if plugin.has_error || !plugin.hooks.on_connect {
                continue;
            }
            plugin.update_config(config);
            let _ = plugin.call_lifecycle_hook("onConnect");
        }
    }

    /// Called when serial port disconnects.
    pub fn on_disconnect(&mut self) {
        for plugin in &mut self.plugins {
            if plugin.has_error || !plugin.hooks.on_disconnect {
                continue;
            }
            let _ = plugin.call_lifecycle_hook("onDisconnect");
        }
    }

    /// Called before app exit — calls onUnload for all plugins.
    pub fn on_app_exit(&mut self) {
        for plugin in &mut self.plugins {
            plugin.unload();
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
    pub fn process_rx(
        &mut self,
        data: Vec<u8>,
        config: &SerialConfig,
    ) -> (Vec<u8>, bool) {
        self.process_pipeline("onRx", data, config)
    }

    /// Process outgoing data through all plugins with onTx hooks.
    ///
    /// Same pipeline semantics as `process_rx`.
    pub fn process_tx(
        &mut self,
        data: Vec<u8>,
        config: &SerialConfig,
    ) -> (Vec<u8>, bool) {
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

            match plugin.call_data_hook(hook_name, &data) {
                PluginResult::PassThrough => {
                    // Data unchanged, continue to next plugin
                }
                PluginResult::Modified(new_data) => {
                    data = new_data;
                }
                PluginResult::Suppressed => {
                    return (data, true);
                }
                PluginResult::Error(msg) => {
                    // Log error and skip this plugin in the future
                    plugin.has_error = true;
                    plugin.error_message = Some(msg.clone());
                    plugin.append_log(
                        NotificationLevel::Error,
                        format!("[plugin: {}] {}", plugin.name, msg),
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

// ── Helpers ─────────────────────────────────────────────────────

/// Extract a human-readable plugin name from a GitHub URL.
///
/// e.g. `https://github.com/user/my-plugin.git` → `my-plugin`
fn repo_name_from_url(url: &str) -> String {
    url.trim_end_matches('/')
        .trim_end_matches(".git")
        .rsplit('/')
        .next()
        .unwrap_or("unknown-plugin")
        .to_string()
}

/// Recursively copy a directory from `src` to `dst`.
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), std::io::Error> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

/// Truncate a commit hash to 7 characters for display.
fn short_hash(hash: &str) -> String {
    if hash.len() > 7 {
        hash[..7].to_string()
    } else {
        hash.to_string()
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
        let (result, suppressed) = manager.process_rx(
            original.clone(),
            &SerialConfig::default(),
        );
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
                // Add a 0x00 byte at the end
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
        let (result, suppressed) = manager.process_rx(
            original.clone(),
            &SerialConfig::default(),
        );
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
        let (_result, suppressed) = manager.process_rx(
            original,
            &SerialConfig::default(),
        );
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
