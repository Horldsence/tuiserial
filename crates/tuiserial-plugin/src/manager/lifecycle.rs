//! Lifecycle hooks and plugin state management.
//!
//! Handles serial port connect/disconnect/exit lifecycle events
//! and explicit enable/disable operations.

use std::panic::{AssertUnwindSafe, catch_unwind};

use tuiserial_core::{
    AppError, ErrorContext, NotificationLevel, PluginErrorKind, RecoveryStrategy, SerialConfig,
};

use crate::runtime::PluginRuntime;

use super::{PluginManager, extract_panic_message};

// ── Plugin enable/disable ───────────────────────────────────────

impl PluginManager {
    /// Enable a plugin that is currently in the `disabled/` directory.
    ///
    /// Moves the plugin from `disabled/<name>/` back to `<plugin_dir>/<name>/`,
    /// creates a `PluginRuntime`, loads it, and inserts it into the active
    /// plugin list in sorted order.
    pub fn enable_plugin(&mut self, name: &str) -> Result<(), AppError> {
        let disabled_dir = self.plugin_dir.join("disabled");
        let src_dir = disabled_dir.join(name);

        if !src_dir.exists() || !src_dir.is_dir() {
            return Err(AppError::Plugin {
                plugin: name.into(),
                kind: PluginErrorKind::Script(format!(
                    "Plugin '{}' not found in disabled directory",
                    name
                )),
                ctx: ErrorContext::new("plugin", "enable", RecoveryStrategy::None),
            });
        }

        // Verify the disabled plugin has a valid entry point
        let has_entry = src_dir.join("plugin.ts").exists() || src_dir.join("plugin.js").exists();
        if !has_entry {
            return Err(AppError::Plugin {
                plugin: name.into(),
                kind: PluginErrorKind::Script(format!(
                    "Plugin '{}' in disabled/ has no plugin.ts or plugin.js",
                    name
                )),
                ctx: ErrorContext::new("plugin", "enable", RecoveryStrategy::None),
            });
        }

        let dest_dir = self.plugin_dir.join(name);

        // If the destination already exists (e.g., the plugin was errored but
        // still in the active list), unload and remove it first.
        if dest_dir.exists() {
            // Remove from active plugins list if present
            if let Some(pos) = self.plugins.iter().position(|p| p.name == name) {
                let mut plugin = self.plugins.remove(pos);
                let _ = catch_unwind(AssertUnwindSafe(|| plugin.unload()));
            }
            std::fs::remove_dir_all(&dest_dir).map_err(|e| AppError::Plugin {
                plugin: name.into(),
                kind: PluginErrorKind::Io(e.to_string()),
                ctx: ErrorContext::new("plugin", "enable (cleanup)", RecoveryStrategy::None),
            })?;
        }

        // Move from disabled/ to plugin dir
        std::fs::rename(&src_dir, &dest_dir).map_err(|e| AppError::Plugin {
            plugin: name.into(),
            kind: PluginErrorKind::Io(e.to_string()),
            ctx: ErrorContext::new("plugin", "enable (move)", RecoveryStrategy::None),
        })?;

        // Detect entry point
        let source_path = if dest_dir.join("plugin.ts").exists() {
            dest_dir.join("plugin.ts")
        } else {
            dest_dir.join("plugin.js")
        };

        // Create and load the runtime
        let mut runtime =
            PluginRuntime::new(name, source_path.clone(), dest_dir.clone()).map_err(|e| {
                AppError::Plugin {
                    plugin: name.into(),
                    kind: PluginErrorKind::from(e),
                    ctx: ErrorContext::new(
                        "plugin",
                        "enable (create)",
                        RecoveryStrategy::DisableComponent,
                    ),
                }
            })?;

        catch_unwind(AssertUnwindSafe(|| runtime.load()))
            .map_err(|panic_info| {
                let msg = extract_panic_message(&panic_info);
                AppError::Plugin {
                    plugin: name.into(),
                    kind: PluginErrorKind::Panic {
                        hook: "enable load".into(),
                        message: msg,
                    },
                    ctx: ErrorContext::new(
                        "plugin",
                        "enable (load)",
                        RecoveryStrategy::DisableComponent,
                    ),
                }
            })?
            .map_err(|e| AppError::Plugin {
                plugin: name.into(),
                kind: PluginErrorKind::from(e),
                ctx: ErrorContext::new(
                    "plugin",
                    "enable (load)",
                    RecoveryStrategy::DisableComponent,
                ),
            })?;

        // Remove from failed_plugins if present
        self.failed_plugins.retain(|fp| fp.name != name);

        // Insert in sorted position
        let pos = self
            .plugins
            .binary_search_by(|p| p.name.cmp(&runtime.name))
            .unwrap_or_else(|i| i);
        self.plugins.insert(pos, runtime);

        // Log the action
        if let Some(plugin) = self.plugins.iter().find(|p| p.name == name) {
            plugin.append_log(
                NotificationLevel::Info,
                format!("[plugin: {}] enabled by user", name),
            );
        }

        Ok(())
    }

    /// Disable a plugin by moving it to the `disabled/` directory.
    ///
    /// Unloads the plugin (calls `onUnload` if defined), removes it from
    /// the active list, and moves its directory to `disabled/<name>/`.
    pub fn disable_plugin(&mut self, name: &str) -> Result<(), AppError> {
        // Find the plugin in the active list (regardless of error state)
        let pos = self
            .plugins
            .iter()
            .position(|p| p.name == name)
            .ok_or_else(|| AppError::Plugin {
                plugin: name.into(),
                kind: PluginErrorKind::Script(format!(
                    "Plugin '{}' not found in active plugins",
                    name
                )),
                ctx: ErrorContext::new("plugin", "disable", RecoveryStrategy::None),
            })?;

        // Unload and remove from list
        let mut plugin = self.plugins.remove(pos);
        let _ = catch_unwind(AssertUnwindSafe(|| plugin.unload()));

        // Also remove from failed_plugins
        self.failed_plugins.retain(|fp| fp.name != name);

        // Ensure disabled directory exists
        let disabled_dir = self.plugin_dir.join("disabled");
        if !disabled_dir.exists() {
            std::fs::create_dir_all(&disabled_dir).map_err(|e| AppError::Plugin {
                plugin: name.into(),
                kind: PluginErrorKind::Io(e.to_string()),
                ctx: ErrorContext::new(
                    "plugin",
                    "disable (create disabled dir)",
                    RecoveryStrategy::None,
                ),
            })?;
        }

        // If a disabled copy already exists, remove it first
        let dest_dir = disabled_dir.join(name);
        if dest_dir.exists() {
            std::fs::remove_dir_all(&dest_dir).map_err(|e| AppError::Plugin {
                plugin: name.into(),
                kind: PluginErrorKind::Io(e.to_string()),
                ctx: ErrorContext::new("plugin", "disable (cleanup)", RecoveryStrategy::None),
            })?;
        }

        let src_dir = self.plugin_dir.join(name);
        std::fs::rename(&src_dir, &dest_dir).map_err(|e| AppError::Plugin {
            plugin: name.into(),
            kind: PluginErrorKind::Io(e.to_string()),
            ctx: ErrorContext::new("plugin", "disable (move)", RecoveryStrategy::None),
        })?;

        Ok(())
    }
}

// ── Lifecycle hooks ──────────────────────────────────────────

impl PluginManager {
    /// Called when serial port connects.
    ///
    /// Returns a list of errors from plugins whose `onConnect` hook
    /// failed.  The caller should record these via
    /// `app.record_error()`.
    pub fn on_connect(&mut self, config: &SerialConfig) -> Vec<AppError> {
        let mut errors = Vec::new();
        for plugin in &mut self.plugins {
            if plugin.has_error || !plugin.hooks.on_connect {
                continue;
            }
            plugin.update_config(config);
            let result = catch_unwind(AssertUnwindSafe(|| plugin.call_lifecycle_hook("onConnect")));
            match result {
                Ok(Ok(())) => {}
                Ok(Err(e)) => {
                    let kind: PluginErrorKind = e.into();
                    errors.push(AppError::Plugin {
                        plugin: plugin.name.clone(),
                        kind,
                        ctx: ErrorContext::new(
                            "plugin",
                            "onConnect lifecycle hook",
                            RecoveryStrategy::Skip,
                        ),
                    });
                }
                Err(panic_info) => {
                    let msg = extract_panic_message(&panic_info);
                    errors.push(AppError::Plugin {
                        plugin: plugin.name.clone(),
                        kind: PluginErrorKind::Panic {
                            hook: "onConnect".into(),
                            message: msg,
                        },
                        ctx: ErrorContext::new(
                            "plugin",
                            "onConnect lifecycle hook",
                            RecoveryStrategy::DisableComponent,
                        ),
                    });
                }
            }
        }
        errors
    }

    /// Called when serial port disconnects.
    ///
    /// Returns a list of errors from plugins whose `onDisconnect` hook
    /// failed.
    pub fn on_disconnect(&mut self) -> Vec<AppError> {
        let mut errors = Vec::new();
        for plugin in &mut self.plugins {
            if plugin.has_error || !plugin.hooks.on_disconnect {
                continue;
            }
            let result = catch_unwind(AssertUnwindSafe(|| {
                plugin.call_lifecycle_hook("onDisconnect")
            }));
            match result {
                Ok(Ok(())) => {}
                Ok(Err(e)) => {
                    let kind: PluginErrorKind = e.into();
                    errors.push(AppError::Plugin {
                        plugin: plugin.name.clone(),
                        kind,
                        ctx: ErrorContext::new(
                            "plugin",
                            "onDisconnect lifecycle hook",
                            RecoveryStrategy::Skip,
                        ),
                    });
                }
                Err(panic_info) => {
                    let msg = extract_panic_message(&panic_info);
                    errors.push(AppError::Plugin {
                        plugin: plugin.name.clone(),
                        kind: PluginErrorKind::Panic {
                            hook: "onDisconnect".into(),
                            message: msg,
                        },
                        ctx: ErrorContext::new(
                            "plugin",
                            "onDisconnect lifecycle hook",
                            RecoveryStrategy::DisableComponent,
                        ),
                    });
                }
            }
        }
        errors
    }

    /// Called before app exit — calls onUnload for all plugins.
    pub fn on_app_exit(&mut self) {
        for plugin in &mut self.plugins {
            let _ = catch_unwind(AssertUnwindSafe(|| plugin.unload()));
        }
        self.plugins.clear();
    }
}
