//! Plugin status reporting — collects per-plugin state for the UI modal.
//!
//! Methods in this module populate `PluginLoadStatus` and `PluginInfo` from
//! the manager's internal state, and provide metadata access helpers.

use tuiserial_core::{PluginLoadState, PluginLoadStatus, PluginMetadataSimple};

use crate::manager::PluginManager;
use crate::types::{PluginInfo, PluginMetadata};

impl PluginManager {
    /// Get detailed plugin statuses for the plugin manager modal.
    ///
    /// Returns a `PluginLoadStatus` for every plugin directory found
    /// (loaded, error, and disabled plugins), with hook info, error
    /// messages, and metadata.
    pub fn get_plugin_statuses(&self) -> Vec<PluginLoadStatus> {
        let mut statuses: Vec<PluginLoadStatus> = Vec::new();

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

        for f in &self.failed_plugins {
            statuses.push(PluginLoadStatus {
                name: f.name.clone(),
                state: PluginLoadState::Error,
                has_rx_hook: false,
                has_tx_hook: false,
                has_connect_hook: false,
                has_disconnect_hook: false,
                error_message: Some(f.error.clone()),
                metadata: self.read_metadata(&f.name).map(|m| PluginMetadataSimple {
                    version: m.version,
                    description: m.description,
                    author: m.author,
                }),
            });
        }

        let disabled_dir = self.plugin_dir.join("disabled");
        if disabled_dir.exists()
            && let Ok(entries) = std::fs::read_dir(&disabled_dir)
        {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let name = path.file_name().unwrap().to_string_lossy().to_string();
                let has_file = path.join("plugin.ts").exists() || path.join("plugin.js").exists();
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

        statuses.sort_by(|a, b| a.name.cmp(&b.name));
        statuses
    }

    /// Get list of plugin info for UI display.
    pub fn list_plugins(&self) -> Vec<PluginInfo> {
        let mut infos: Vec<PluginInfo> = self
            .plugins
            .iter()
            .map(|p| PluginInfo {
                name: p.name.clone(),
                enabled: !p.has_error,
                hooks: p.hooks.clone(),
                has_error: p.has_error,
                error_message: p.error_message.clone(),
            })
            .collect();

        for f in &self.failed_plugins {
            infos.push(PluginInfo {
                name: f.name.clone(),
                enabled: false,
                hooks: Default::default(),
                has_error: true,
                error_message: Some(f.error.clone()),
            });
        }

        infos
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
            && let Some(repo) = meta.repo
        {
            return Some(repo);
        }
        let dir = self.plugin_dir.join(plugin_name);
        if crate::git::is_git_repo(&dir) {
            return crate::git::git_remote_url(&dir).ok();
        }
        None
    }
}
