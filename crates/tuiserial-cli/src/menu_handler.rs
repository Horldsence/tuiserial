//! Menu action handler — dispatches menu bar actions to the appropriate logic.

use rust_i18n::t;
use tuiserial_core::{AppState, MenuAction, menu_def::MENU_BAR};
#[cfg(feature = "plugin")]
use tuiserial_core::{PluginLoadState, PluginModalMode};
#[cfg(feature = "plugin")]
use tuiserial_plugin::PluginManager;

use crate::handler::SerialHandler;

#[cfg(feature = "plugin")]
/// Sync plugin statuses from PluginManager to AppState.
pub fn sync_plugin_status(app: &mut AppState, manager: &PluginManager) {
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

/// Handle menu action execution.
pub fn handle_menu_action(
    app: &mut AppState,
    handler: &mut SerialHandler,
    #[cfg(feature = "plugin")] plugin_manager: &mut PluginManager,
    menu_idx: usize,
    item_idx: usize,
) -> bool {
    let action = match MENU_BAR.get_action(menu_idx, item_idx) {
        Some(a) => a,
        None => return false,
    };

    if action.is_separator() {
        return false;
    }

    match action {
        MenuAction::SaveConfig => {
            match app.save_config() {
                Ok(_) => app.add_success(t!("notify.config_saved").to_string()),
                Err(e) => app.add_error(format!("{}: {}", t!("notify.config_save_failed"), e)),
            }
            false
        }
        MenuAction::LoadConfig => {
            app.load_config();
            app.add_success(t!("notify.config_loaded").to_string());
            false
        }
        MenuAction::Exit => {
            if handler.is_connected() {
                handler.disconnect();
            }
            true
        }
        MenuAction::ToggleLanguage => {
            app.toggle_language();
            app.add_success(t!("notify.language_changed").to_string());
            false
        }
        MenuAction::ShowShortcuts => {
            app.show_shortcuts_help = !app.show_shortcuts_help;
            false
        }
        MenuAction::ShowAbout => {
            let about_text = if app.language == tuiserial_core::Language::English {
                "TuiSerial v0.2.0\nTerminal Serial Port Monitor\n\nA modern serial port debugging tool with mouse support, plugin system and internationalization."
            } else {
                "TuiSerial v0.2.0\n终端串口监控工具\n\n一个现代化的串口调试工具，支持插件系统、鼠标操作和国际化。"
            };
            app.add_info(about_text.to_string());
            false
        }
        MenuAction::NewSession
        | MenuAction::DuplicateSession
        | MenuAction::RenameSession
        | MenuAction::CloseSession => {
            app.add_info("Multi-session support coming soon!".to_string());
            false
        }
        MenuAction::ViewSingle
        | MenuAction::ViewSplitHorizontal
        | MenuAction::ViewSplitVertical
        | MenuAction::ViewGrid2x2
        | MenuAction::ViewNextPane
        | MenuAction::ViewPrevPane => {
            app.add_info("Layout management coming soon!".to_string());
            false
        }
        #[cfg(feature = "plugin")]
        MenuAction::PluginsInstall => {
            if !tuiserial_plugin::git::git_available() {
                app.add_error(t!("notify.plugin_git_missing").to_string());
                return false;
            }
            sync_plugin_status(app, plugin_manager);
            app.plugin_modal_mode = PluginModalMode::Registry;
            app.registry_search_query.clear();
            app.registry_scroll = 0;
            app.show_plugin_modal = true;

            app.registry_loading = true;
            match plugin_manager.get_registry() {
                Ok(registry) => {
                    app.registry_entries = registry;
                    app.registry_loading = false;
                }
                Err(e) => {
                    app.registry_loading = false;
                    app.add_error(format!("{}: {}", t!("notify.plugin_install_failed"), e));
                }
            }
            false
        }
        #[cfg(not(feature = "plugin"))]
        MenuAction::PluginsInstall => {
            app.add_error(t!("notify.plugin_disabled").to_string());
            false
        }
        #[cfg(feature = "plugin")]
        MenuAction::PluginsCheckUpdate => {
            if !tuiserial_plugin::git::git_available() {
                app.add_error(t!("notify.plugin_git_missing").to_string());
                return false;
            }
            app.add_info(t!("notify.plugin_checking").to_string());
            match plugin_manager.check_updates() {
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
            false
        }
        #[cfg(not(feature = "plugin"))]
        MenuAction::PluginsCheckUpdate => {
            app.add_error(t!("notify.plugin_disabled").to_string());
            false
        }
        #[cfg(feature = "plugin")]
        MenuAction::PluginsUpdateAll => {
            if !tuiserial_plugin::git::git_available() {
                app.add_error(t!("notify.plugin_git_missing").to_string());
                return false;
            }
            let (updated, errors) = plugin_manager.update_all();
            if updated > 0 {
                app.add_success(t!("notify.plugin_all_updated", count = updated));
            }
            for err in &errors {
                app.add_error(t!("notify.plugin_update_failed", error = err));
            }
            if updated == 0 && errors.is_empty() {
                app.add_success(t!("notify.plugin_up_to_date").to_string());
            }
            false
        }
        #[cfg(not(feature = "plugin"))]
        MenuAction::PluginsUpdateAll => {
            app.add_error(t!("notify.plugin_disabled").to_string());
            false
        }
        #[cfg(feature = "plugin")]
        MenuAction::PluginsReload => {
            match plugin_manager.reload_all() {
                Ok(n) => {
                    sync_plugin_status(app, plugin_manager);
                    app.add_success(format!("{} plugin(s) reloaded", n));
                }
                Err(e) => app.add_error(format!("Plugin reload error: {}", e)),
            }
            for err in plugin_manager.drain_load_errors() {
                app.add_error(err);
            }
            false
        }
        #[cfg(not(feature = "plugin"))]
        MenuAction::PluginsReload => {
            app.add_error(t!("notify.plugin_disabled").to_string());
            false
        }
        #[cfg(feature = "plugin")]
        MenuAction::PluginsManager => {
            if app.show_plugin_modal {
                app.show_plugin_modal = false;
            } else {
                sync_plugin_status(app, plugin_manager);
                app.plugin_modal_mode = PluginModalMode::Local;
                app.plugin_modal_scroll = 0;
                app.show_plugin_modal = true;
            }
            false
        }
        #[cfg(not(feature = "plugin"))]
        MenuAction::PluginsManager => {
            app.add_error(t!("notify.plugin_disabled").to_string());
            false
        }
        #[cfg(feature = "plugin")]
        MenuAction::PluginsList => {
            let plugins = plugin_manager.list_plugins();
            if plugins.is_empty() {
                app.add_info("No plugins loaded. Place plugins in ~/.config/tuiserial/plugins/<name>/plugin.ts".to_string());
            } else {
                for p in &plugins {
                    let status = if p.has_error { "⚠" } else { "✓" };
                    app.add_info(format!(
                        "{} {} (rx:{}, tx:{})",
                        status, p.name, p.hooks.on_rx, p.hooks.on_tx
                    ));
                }
            }
            false
        }
        #[cfg(not(feature = "plugin"))]
        MenuAction::PluginsList => {
            app.add_error(t!("notify.plugin_disabled").to_string());
            false
        }
        MenuAction::Separator => false,
    }
}
