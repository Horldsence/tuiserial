//! Plugin system facade — the **only** file that gates on `feature = "plugin"`.
//!
//! When the `plugin` feature is enabled, `PluginProxy` wraps a real
//! [`tuiserial_plugin::PluginManager`] and delegates all operations to it.
//! When the feature is disabled, every method is a no-op (passthrough for
//! pipelines, empty results for queries, "Plugin is Disabled" notifications
//! for user actions).
//!
//! All other modules import `PluginProxy` unconditionally, keeping their
//! code free of `#[cfg(feature = "plugin")]` annotations.

use std::path::Path;
#[cfg(feature = "plugin")]
use std::path::PathBuf;

use rust_i18n::t;
use tuiserial_core::{AppError, AppState, SerialConfig};
#[cfg(feature = "plugin")]
use tuiserial_core::{ErrorContext, PluginLoadState, PluginModalMode, RecoveryStrategy};

#[cfg(feature = "plugin")]
use tuiserial_plugin::PluginManager;

// ── PluginProxy ───────────────────────────────────────────────────────

/// Plugin system facade.
///
/// Construct via [`PluginProxy::init`] and pass to all handler functions
/// as `&mut PluginProxy`.  Callers never need to know whether the plugin
/// feature is actually compiled in.
#[cfg(feature = "plugin")]
pub struct PluginProxy {
    inner: PluginManager,
}

#[cfg(not(feature = "plugin"))]
pub struct PluginProxy;

impl PluginProxy {
    // ── Construction ──────────────────────────────────────────────

    /// Create the plugin proxy and perform initial discovery.
    ///
    /// Feature on:  creates a `PluginManager`, scans the plugin directory,
    ///              loads all discovered plugins, syncs status into `app`,
    ///              and drains any load errors into `app` notifications.
    /// Feature off: returns an empty proxy (no-op).
    #[cfg(feature = "plugin")]
    pub fn init(app: &mut AppState) -> Self {
        let plugin_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".config")
            .join("tuiserial")
            .join("plugins");
        let mut pm = PluginManager::new(plugin_dir);
        match pm.discover_and_load() {
            Ok(n) => {
                Self::sync_status_inner(app, &pm);
                if n > 0 {
                    app.add_success(t!("notify.plugins_loaded", count = n));
                }
            }
            Err(e) => {
                Self::sync_status_inner(app, &pm);
                let kind: tuiserial_core::PluginErrorKind = e.into();
                app.record_error(AppError::Plugin {
                    plugin: "<init>".into(),
                    kind,
                    ctx: ErrorContext::new("plugin", "discover_and_load", RecoveryStrategy::Retry),
                });
            }
        }
        for err in pm.drain_load_errors() {
            app.add_error(err);
        }
        Self { inner: pm }
    }

    #[cfg(not(feature = "plugin"))]
    pub fn init(_app: &mut AppState) -> Self {
        Self
    }

    // ── Status sync ───────────────────────────────────────────────

    /// Copy plugin statuses from the manager (or clear them when feature
    /// is disabled).
    pub fn sync_status(&self, app: &mut AppState) {
        #[cfg(feature = "plugin")]
        Self::sync_status_inner(app, &self.inner);
        #[cfg(not(feature = "plugin"))]
        {
            app.plugin_statuses.clear();
            app.plugin_total_count = 0;
            app.plugin_loaded_count = 0;
            app.plugin_error_count = 0;
        }
    }

    #[cfg(feature = "plugin")]
    fn sync_status_inner(app: &mut AppState, manager: &PluginManager) {
        let statuses = manager.get_plugin_statuses();
        let total = statuses.len();
        let loaded = statuses
            .iter()
            .filter(|s| s.state == PluginLoadState::Loaded)
            .count();
        let errors = statuses
            .iter()
            .filter(|s| s.state == PluginLoadState::Error)
            .count();
        app.plugin_statuses = statuses;
        app.plugin_total_count = total;
        app.plugin_loaded_count = loaded;
        app.plugin_error_count = errors;
    }

    // ── Pipeline ──────────────────────────────────────────────────

    /// Process received serial data through plugin `onRx` hooks.
    /// Returns `(possibly_modified_data, suppressed)`.
    /// Feature off: identity passthrough — `(data, false)`.
    pub fn process_rx(&mut self, data: Vec<u8>, config: &SerialConfig) -> (Vec<u8>, bool) {
        #[cfg(feature = "plugin")]
        {
            self.inner.process_rx(data, config)
        }
        #[cfg(not(feature = "plugin"))]
        {
            let _ = config;
            (data, false)
        }
    }

    /// Process outgoing data through plugin `onTx` hooks.
    /// Returns `(possibly_modified_data, suppressed)`.
    /// Feature off: identity passthrough — `(data, false)`.
    pub fn process_tx(&mut self, data: Vec<u8>, config: &SerialConfig) -> (Vec<u8>, bool) {
        #[cfg(feature = "plugin")]
        {
            self.inner.process_tx(data, config)
        }
        #[cfg(not(feature = "plugin"))]
        {
            let _ = config;
            (data, false)
        }
    }

    /// Flush buffered plugin log messages into the app notification queue.
    /// Feature off: no-op.
    pub fn flush_plugin_logs(&mut self, app: &mut AppState) {
        #[cfg(feature = "plugin")]
        self.inner.flush_plugin_logs(app);
        #[cfg(not(feature = "plugin"))]
        let _ = app;
    }

    // ── Lifecycle ─────────────────────────────────────────────────

    /// Call `onConnect` on all loaded plugins.
    /// Returns a list of errors (empty when feature is off).
    pub fn on_connect(&mut self, config: &SerialConfig) -> Vec<AppError> {
        #[cfg(feature = "plugin")]
        {
            self.inner.on_connect(config)
        }
        #[cfg(not(feature = "plugin"))]
        {
            let _ = config;
            Vec::new()
        }
    }

    /// Call `onDisconnect` on all loaded plugins.
    /// Returns a list of errors (empty when feature is off).
    pub fn on_disconnect(&mut self) -> Vec<AppError> {
        #[cfg(feature = "plugin")]
        {
            self.inner.on_disconnect()
        }
        #[cfg(not(feature = "plugin"))]
        {
            Vec::new()
        }
    }

    /// Call `onUnload` on all plugins and clear the plugin list.
    /// Feature off: no-op.
    pub fn on_app_exit(&mut self) {
        #[cfg(feature = "plugin")]
        self.inner.on_app_exit();
    }

    // ── Plugin directory ──────────────────────────────────────────

    /// Return the plugin directory path.
    #[cfg(feature = "plugin")]
    pub fn plugin_dir(&self) -> &Path {
        self.inner.plugin_dir()
    }

    #[cfg(not(feature = "plugin"))]
    #[allow(dead_code)]
    pub fn plugin_dir(&self) -> &Path {
        Path::new(".")
    }

    // ── High-level action methods ─────────────────────────────────
    //
    // Each method bundles the full user-facing workflow for a menu
    // action or modal key.  The no-op versions push the "Plugin is
    // Disabled" notification via `app`.

    /// Open the local plugin manager modal.
    pub fn open_local_modal(&self, app: &mut AppState) {
        #[cfg(feature = "plugin")]
        {
            self.sync_status(app);
            app.plugin_modal_mode = PluginModalMode::Local;
            app.plugin_modal_scroll = 0;
            app.show_plugin_modal = true;
        }
        #[cfg(not(feature = "plugin"))]
        {
            app.add_error(t!("notify.plugin_disabled").to_string());
        }
    }

    /// Open the plugin registry (install) modal.
    pub fn open_registry_modal(&mut self, app: &mut AppState) {
        #[cfg(feature = "plugin")]
        {
            if !tuiserial_plugin::git::git_available() {
                app.add_error(t!("notify.plugin_git_missing").to_string());
                return;
            }
            self.sync_status(app);
            app.plugin_modal_mode = PluginModalMode::Registry;
            app.registry_search_query.clear();
            app.registry_scroll = 0;
            app.show_plugin_modal = true;

            app.registry_loading = true;
            match self.inner.get_registry() {
                Ok(registry) => {
                    app.registry_entries = registry;
                    app.registry_loading = false;
                }
                Err(e) => {
                    app.registry_loading = false;
                    app.add_error(format!("{}: {}", t!("notify.plugin_install_failed"), e));
                }
            }
        }
        #[cfg(not(feature = "plugin"))]
        {
            app.add_error(t!("notify.plugin_disabled").to_string());
        }
    }

    /// Reload all plugins.  Syncs status and adds notifications.
    pub fn reload_all_plugins(&mut self, app: &mut AppState) {
        #[cfg(feature = "plugin")]
        {
            match self.inner.reload_all() {
                Ok(n) => {
                    self.sync_status(app);
                    app.add_success(format!("{} plugin(s) reloaded", n));
                }
                Err(e) => app.add_error(format!("Plugin reload error: {}", e)),
            }
            for err in self.inner.drain_load_errors() {
                app.add_error(err);
            }
        }
        #[cfg(not(feature = "plugin"))]
        {
            app.add_error(t!("notify.plugin_disabled").to_string());
        }
    }

    /// Enable the plugin at `app.plugin_modal_scroll` (must be in Disabled state).
    pub fn enable_plugin_action(&mut self, app: &mut AppState) {
        #[cfg(feature = "plugin")]
        {
            let scroll = app.plugin_modal_scroll;
            let target = app.plugin_statuses.get(scroll).and_then(|ps| {
                if ps.state == PluginLoadState::Disabled {
                    Some(ps.name.clone())
                } else {
                    None
                }
            });
            if let Some(name) = target {
                match self.inner.enable_plugin(&name) {
                    Ok(()) => {
                        self.sync_status(app);
                        app.add_success(format!("Plugin '{}' enabled", name));
                    }
                    Err(err) => {
                        app.add_error(format!("Failed to enable '{}': {}", name, err));
                    }
                }
            }
        }
        #[cfg(not(feature = "plugin"))]
        {
            app.add_error(t!("notify.plugin_disabled").to_string());
        }
    }

    /// Disable the plugin at `app.plugin_modal_scroll` (must be Loaded or Error).
    pub fn disable_plugin_action(&mut self, app: &mut AppState) {
        #[cfg(feature = "plugin")]
        {
            let scroll = app.plugin_modal_scroll;
            let target = app.plugin_statuses.get(scroll).and_then(|ps| {
                if ps.state == PluginLoadState::Loaded || ps.state == PluginLoadState::Error {
                    Some(ps.name.clone())
                } else {
                    None
                }
            });
            if let Some(name) = target {
                match self.inner.disable_plugin(&name) {
                    Ok(()) => {
                        self.sync_status(app);
                        app.add_success(format!("Plugin '{}' disabled", name));
                    }
                    Err(err) => {
                        app.add_error(format!("Failed to disable '{}': {}", name, err));
                    }
                }
            }
        }
        #[cfg(not(feature = "plugin"))]
        {
            app.add_error(t!("notify.plugin_disabled").to_string());
        }
    }

    /// Install a plugin from the registry cache.
    /// Uses `app.registry_scroll` to select the entry.
    pub fn install_from_registry(&mut self, app: &mut AppState) {
        #[cfg(feature = "plugin")]
        {
            let filtered = filtered_registry_entries(app);
            if let Some(entry) = filtered.get(app.registry_scroll) {
                let target = self.plugin_dir().join(&entry.name);
                if target.exists() {
                    app.add_info(format!("{}: already installed", entry.name));
                } else {
                    match self.inner.install_plugin_from_cache(&entry.name) {
                        Ok(()) => {
                            app.add_success(t!("notify.plugin_installed", name = &entry.name));
                            self.sync_status(app);
                        }
                        Err(e) => app
                            .add_error(t!("notify.plugin_install_failed", error = &e.to_string())),
                    }
                }
            }
        }
        #[cfg(not(feature = "plugin"))]
        {
            app.add_error(t!("notify.plugin_disabled").to_string());
        }
    }

    /// Check for plugin updates.
    pub fn check_updates_action(&mut self, app: &mut AppState) {
        #[cfg(feature = "plugin")]
        {
            if !tuiserial_plugin::git::git_available() {
                app.add_error(t!("notify.plugin_git_missing").to_string());
                return;
            }
            app.add_info(t!("notify.plugin_checking").to_string());
            match self.inner.check_updates() {
                Ok(statuses) => {
                    if statuses.is_empty() {
                        app.add_info("No git-managed plugins found".to_string());
                    } else {
                        let mut has_update = false;
                        for s in &statuses {
                            if s.has_update {
                                has_update = true;
                                app.add_info(t!(
                                    "notify.plugin_update_available",
                                    name = &s.name,
                                    current = &s.current_commit,
                                    latest = &s.latest_commit
                                ));
                            }
                        }
                        if !has_update {
                            app.add_success(t!("notify.plugin_up_to_date").to_string());
                        }
                    }
                }
                Err(e) => app.add_error(format!("Check failed: {}", e)),
            }
        }
        #[cfg(not(feature = "plugin"))]
        {
            app.add_error(t!("notify.plugin_disabled").to_string());
        }
    }

    /// Update all plugins (git pull --ff-only).
    pub fn update_all_action(&mut self, app: &mut AppState) {
        #[cfg(feature = "plugin")]
        {
            if !tuiserial_plugin::git::git_available() {
                app.add_error(t!("notify.plugin_git_missing").to_string());
                return;
            }
            let (updated, errors) = self.inner.update_all();
            if updated > 0 {
                app.add_success(t!("notify.plugin_all_updated", count = updated));
            }
            for err in &errors {
                app.add_error(t!("notify.plugin_update_failed", error = err));
            }
            if updated == 0 && errors.is_empty() {
                app.add_success(t!("notify.plugin_up_to_date").to_string());
            }
        }
        #[cfg(not(feature = "plugin"))]
        {
            app.add_error(t!("notify.plugin_disabled").to_string());
        }
    }

    /// List all loaded plugins and their hooks.
    pub fn list_plugins_action(&self, app: &mut AppState) {
        #[cfg(feature = "plugin")]
        {
            let plugins = self.inner.list_plugins();
            if plugins.is_empty() {
                app.add_info(
                    "No plugins loaded. Place plugins in ~/.config/tuiserial/plugins/<name>/plugin.ts"
                        .to_string(),
                );
            } else {
                for p in &plugins {
                    let status = if p.has_error { "⚠" } else { "✓" };
                    app.add_info(format!(
                        "{} {} (rx:{}, tx:{})",
                        status, p.name, p.hooks.on_rx, p.hooks.on_tx
                    ));
                }
            }
        }
        #[cfg(not(feature = "plugin"))]
        {
            app.add_error(t!("notify.plugin_disabled").to_string());
        }
    }
}

// ── Registry helpers ─────────────────────────────────────────────────
//
// These are pure functions on AppState data, no PluginManager dependency.
// They are used by PluginProxy::install_from_registry and by the key
// handler for scroll navigation in the registry modal.

/// Count registry entries matching the current search query.
pub fn filtered_registry_count(app: &AppState) -> usize {
    let query = app.registry_search_query.to_lowercase();
    if query.is_empty() {
        app.registry_entries.len()
    } else {
        app.registry_entries
            .iter()
            .filter(|e| {
                e.name.to_lowercase().contains(&query)
                    || e.description
                        .as_ref()
                        .map(|d| d.to_lowercase().contains(&query))
                        .unwrap_or(false)
            })
            .count()
    }
}

/// Get filtered registry entries matching the current search query.
/// Only used when the `plugin` feature is enabled (inside `install_from_registry`).
#[cfg(feature = "plugin")]
fn filtered_registry_entries(app: &AppState) -> Vec<&tuiserial_core::RegistryEntry> {
    let query = app.registry_search_query.to_lowercase();
    if query.is_empty() {
        app.registry_entries.iter().collect()
    } else {
        app.registry_entries
            .iter()
            .filter(|e| {
                e.name.to_lowercase().contains(&query)
                    || e.description
                        .as_ref()
                        .map(|d| d.to_lowercase().contains(&query))
                        .unwrap_or(false)
            })
            .collect()
    }
}
