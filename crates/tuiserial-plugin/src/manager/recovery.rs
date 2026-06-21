//! Plugin error recovery — reload and retry failed plugins.
//!
//! Handles `reload_errored_plugins()` (batch re-load of all errored plugins)
//! and `retry_failed_plugin()` (single-plugin retry from the error state).

use std::panic::{AssertUnwindSafe, catch_unwind};

use tuiserial_core::{AppError, ErrorContext, PluginErrorKind, RecoveryStrategy};

use crate::runtime::PluginRuntime;
use crate::types::FailedPlugin;

use super::{PluginManager, extract_panic_message};

impl PluginManager {
    /// Reload only plugins that are in the error state (`has_error = true`).
    ///
    /// Returns a list of errors for plugins that still failed to reload.
    pub fn reload_errored_plugins(&mut self) -> Vec<AppError> {
        let mut errors = Vec::new();

        // Collect names of errored plugins
        let errored: Vec<(usize, String, std::path::PathBuf, std::path::PathBuf)> = self
            .plugins
            .iter()
            .enumerate()
            .filter(|(_, p)| p.has_error)
            .map(|(i, p)| {
                (
                    i,
                    p.name.clone(),
                    p.source_path.clone(),
                    p.plugin_dir.clone(),
                )
            })
            .collect();

        // Remove them from the list (in reverse order to keep indices valid)
        for (idx, _, _, _) in errored.iter().rev() {
            let mut plugin = self.plugins.remove(*idx);
            let _ = catch_unwind(AssertUnwindSafe(|| plugin.unload()));
        }

        // Remove from failed_plugins too (names might appear there from load errors)
        self.failed_plugins
            .retain(|fp| !errored.iter().any(|(_, name, _, _)| &fp.name == name));

        // Re-attempt loading each one
        for (_idx, name, source_path, plugin_dir) in errored {
            match PluginRuntime::new(&name, source_path, plugin_dir) {
                Ok(mut runtime) => {
                    match catch_unwind(AssertUnwindSafe(|| runtime.load())) {
                        Ok(Ok(())) => {
                            // Sort alphabetically when inserting
                            let pos = self
                                .plugins
                                .binary_search_by(|p| p.name.cmp(&runtime.name))
                                .unwrap_or_else(|i| i);
                            self.plugins.insert(pos, runtime);
                        }
                        Ok(Err(e)) => {
                            let kind: PluginErrorKind = e.into();
                            self.failed_plugins.push(FailedPlugin {
                                name: name.clone(),
                                error: kind.to_string(),
                            });
                            errors.push(AppError::Plugin {
                                plugin: name,
                                kind,
                                ctx: ErrorContext::new(
                                    "plugin",
                                    "reload",
                                    RecoveryStrategy::DisableComponent,
                                ),
                            });
                        }
                        Err(panic_info) => {
                            let msg = extract_panic_message(&panic_info);
                            self.failed_plugins.push(FailedPlugin {
                                name: name.clone(),
                                error: msg.clone(),
                            });
                            errors.push(AppError::Plugin {
                                plugin: name,
                                kind: PluginErrorKind::Panic {
                                    hook: "reload".into(),
                                    message: msg,
                                },
                                ctx: ErrorContext::new(
                                    "plugin",
                                    "reload",
                                    RecoveryStrategy::DisableComponent,
                                ),
                            });
                        }
                    }
                }
                Err(e) => {
                    let kind: PluginErrorKind = e.into();
                    self.failed_plugins.push(FailedPlugin {
                        name: name.clone(),
                        error: kind.to_string(),
                    });
                    errors.push(AppError::Plugin {
                        plugin: name,
                        kind,
                        ctx: ErrorContext::new(
                            "plugin",
                            "reload (create)",
                            RecoveryStrategy::DisableComponent,
                        ),
                    });
                }
            }
        }

        self.plugins.sort_by(|a, b| a.name.cmp(&b.name));
        errors
    }

    /// Retry loading a single failed plugin by name.
    ///
    /// Returns `Ok(())` if the plugin loaded successfully, or an
    /// `AppError` describing why it still failed.
    pub fn retry_failed_plugin(&mut self, name: &str) -> Result<(), AppError> {
        // First try to find it among loaded-but-errored plugins
        if let Some(pos) = self
            .plugins
            .iter()
            .position(|p| p.name == name && p.has_error)
        {
            let mut plugin = self.plugins.remove(pos);
            let _ = catch_unwind(AssertUnwindSafe(|| plugin.unload()));
            // Re-load
            let source_path = plugin.source_path.clone();
            let plugin_dir = plugin.plugin_dir.clone();
            drop(plugin);

            let mut runtime = PluginRuntime::new(name, source_path, plugin_dir).map_err(|e| {
                AppError::Plugin {
                    plugin: name.into(),
                    kind: PluginErrorKind::from(e),
                    ctx: ErrorContext::new(
                        "plugin",
                        "retry (create)",
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
                            hook: "retry load".into(),
                            message: msg,
                        },
                        ctx: ErrorContext::new(
                            "plugin",
                            "retry (load)",
                            RecoveryStrategy::DisableComponent,
                        ),
                    }
                })?
                .map_err(|e| AppError::Plugin {
                    plugin: name.into(),
                    kind: PluginErrorKind::from(e),
                    ctx: ErrorContext::new(
                        "plugin",
                        "retry (load)",
                        RecoveryStrategy::DisableComponent,
                    ),
                })?;

            // Insert in sorted position
            let pos = self
                .plugins
                .binary_search_by(|p| p.name.cmp(&runtime.name))
                .unwrap_or_else(|i| i);
            self.plugins.insert(pos, runtime);
            return Ok(());
        }

        // Try failed_plugins
        if let Some(fpos) = self.failed_plugins.iter().position(|fp| fp.name == name) {
            self.failed_plugins.remove(fpos);
            // Look on disk for the plugin source
            let plugin_dir = self.plugin_dir.join(name);
            for ext in &["ts", "js"] {
                let plugin_file = plugin_dir.join(format!("plugin.{ext}"));
                if plugin_file.exists() {
                    let mut runtime =
                        PluginRuntime::new(name, plugin_file, plugin_dir).map_err(|e| {
                            AppError::Plugin {
                                plugin: name.into(),
                                kind: PluginErrorKind::from(e),
                                ctx: ErrorContext::new(
                                    "plugin",
                                    "retry (create from failed)",
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
                                    hook: "retry load".into(),
                                    message: msg,
                                },
                                ctx: ErrorContext::new(
                                    "plugin",
                                    "retry (load from failed)",
                                    RecoveryStrategy::DisableComponent,
                                ),
                            }
                        })?
                        .map_err(|e| AppError::Plugin {
                            plugin: name.into(),
                            kind: PluginErrorKind::from(e),
                            ctx: ErrorContext::new(
                                "plugin",
                                "retry (load from failed)",
                                RecoveryStrategy::DisableComponent,
                            ),
                        })?;
                    let pos = self
                        .plugins
                        .binary_search_by(|p| p.name.cmp(&runtime.name))
                        .unwrap_or_else(|i| i);
                    self.plugins.insert(pos, runtime);
                    return Ok(());
                }
            }
            return Err(AppError::Plugin {
                plugin: name.into(),
                kind: PluginErrorKind::Script("plugin source not found".into()),
                ctx: ErrorContext::new("plugin", "retry (not found)", RecoveryStrategy::None),
            });
        }

        Err(AppError::Plugin {
            plugin: name.into(),
            kind: PluginErrorKind::Script("plugin not in error list".into()),
            ctx: ErrorContext::new("plugin", "retry (unknown)", RecoveryStrategy::None),
        })
    }
}
