//! Keyboard event handler — routes key events to the appropriate sub-handler.

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use rust_i18n::t;
use tuiserial_core::{AppState, FocusedField, MenuState, PluginModalMode, menu_def::MENU_BAR};
#[cfg(feature = "plugin")]
use tuiserial_plugin::PluginManager;

use crate::handler::SerialHandler;
use crate::menu_handler::handle_menu_action;
#[cfg(feature = "plugin")]
use crate::menu_handler::sync_plugin_status;

/// Main keyboard event handler. Routes to sub-handlers based on application state.
/// Returns `true` if the application should exit.
pub fn handle_key_event(
    key: KeyEvent,
    app: &mut AppState,
    handler: &mut SerialHandler,
    #[cfg(feature = "plugin")] plugin_manager: &mut PluginManager,
) -> bool {
    if key.kind != KeyEventKind::Press {
        return false;
    }

    // Menu navigation takes priority over everything else
    #[cfg(feature = "plugin")]
    if let Some(exit) = handle_menu_navigation(key, app, handler, plugin_manager) {
        return exit;
    }
    #[cfg(not(feature = "plugin"))]
    if let Some(exit) = handle_menu_navigation(key, app, handler) {
        return exit;
    }

    // Plugin modal keyboard
    if app.show_plugin_modal {
        #[cfg(feature = "plugin")]
        return handle_plugin_modal_key(key, app, plugin_manager);
        #[cfg(not(feature = "plugin"))]
        return handle_plugin_modal_key(key, app);
    }

    // Help overlay — consume all keys while showing
    if app.show_shortcuts_help {
        match key.code {
            KeyCode::Esc | KeyCode::F(1) | KeyCode::Char('q') | KeyCode::Char('?') => {
                app.show_shortcuts_help = false;
            }
            _ => {}
        }
        return false;
    }

    // TX input mode
    if app.focused_field == FocusedField::TxInput {
        #[cfg(feature = "plugin")]
        return crate::tx_handler::handle_tx_key_event(key, app, handler, plugin_manager);
        #[cfg(not(feature = "plugin"))]
        return crate::tx_handler::handle_tx_key_event(key, app, handler);
    }

    // Global shortcuts
    #[cfg(feature = "plugin")]
    return crate::global_handler::handle_global_key(key, app, handler, plugin_manager);
    #[cfg(not(feature = "plugin"))]
    crate::global_handler::handle_global_key(key, app, handler)
}

/// Handle menu bar and dropdown navigation. Returns `Some(exit)` when a key is handled
/// by the menu system, or `None` to let other handlers process the key.
fn handle_menu_navigation(
    key: KeyEvent,
    app: &mut AppState,
    handler: &mut SerialHandler,
    #[cfg(feature = "plugin")] plugin_manager: &mut PluginManager,
) -> Option<bool> {
    match app.menu_state {
        MenuState::None => {
            if key.code == KeyCode::F(10) {
                app.menu_state = MenuState::MenuBar(0);
                app.focused_field = FocusedField::LogArea;
                return Some(false);
            }
            if key.code == KeyCode::F(1) || key.code == KeyCode::Char('?') {
                app.show_shortcuts_help = !app.show_shortcuts_help;
                return Some(false);
            }
            if key.code == KeyCode::Char('s') && key.modifiers.contains(KeyModifiers::CONTROL) {
                match app.save_config() {
                    Ok(_) => app.add_success(t!("notify.config_saved").to_string()),
                    Err(e) => app.add_error(format!(
                        "{}: {}",
                        t!("notify.config_save_failed"),
                        e
                    )),
                }
                return Some(false);
            }
            if key.code == KeyCode::Char('o') && key.modifiers.contains(KeyModifiers::CONTROL) {
                app.load_config();
                app.add_success(t!("notify.config_loaded").to_string());
                return Some(false);
            }
            None
        }
        MenuState::MenuBar(selected) => {
            let menu_count = MENU_BAR.menu_count();
            match key.code {
                KeyCode::Left => {
                    app.menu_state = MenuState::MenuBar(if selected == 0 {
                        menu_count - 1
                    } else {
                        selected - 1
                    });
                }
                KeyCode::Right => {
                    app.menu_state = MenuState::MenuBar((selected + 1) % menu_count);
                }
                KeyCode::Enter | KeyCode::Down => {
                    app.menu_state = MenuState::Dropdown(selected, 0);
                }
                KeyCode::Esc => {
                    app.menu_state = MenuState::None;
                }
                _ => return None,
            }
            Some(false)
        }
        MenuState::Dropdown(menu_idx, item_idx) => {
            let item_count = MENU_BAR.get_item_count(menu_idx);

            match key.code {
                KeyCode::Up => {
                    let new_idx = if item_idx == 0 {
                        item_count - 1
                    } else {
                        item_idx - 1
                    };
                    app.menu_state = MenuState::Dropdown(menu_idx, new_idx);
                }
                KeyCode::Down => {
                    app.menu_state = MenuState::Dropdown(menu_idx, (item_idx + 1) % item_count);
                }
                KeyCode::Left => {
                    let menu_count = MENU_BAR.menu_count();
                    let new_menu = if menu_idx == 0 {
                        menu_count - 1
                    } else {
                        menu_idx - 1
                    };
                    app.menu_state = MenuState::Dropdown(new_menu, 0);
                }
                KeyCode::Right => {
                    let menu_count = MENU_BAR.menu_count();
                    let new_menu = (menu_idx + 1) % menu_count;
                    app.menu_state = MenuState::Dropdown(new_menu, 0);
                }
                KeyCode::Enter => {
                    #[cfg(feature = "plugin")]
                    let should_exit =
                        handle_menu_action(app, handler, plugin_manager, menu_idx, item_idx);
                    #[cfg(not(feature = "plugin"))]
                    let should_exit = handle_menu_action(app, handler, menu_idx, item_idx);
                    app.menu_state = MenuState::None;
                    return Some(should_exit);
                }
                KeyCode::Esc => {
                    app.menu_state = MenuState::MenuBar(menu_idx);
                }
                _ => return None,
            }
            Some(false)
        }
    }
}

/// Handle key events while the plugin modal is open.
#[cfg(feature = "plugin")]
fn handle_plugin_modal_key(
    key: KeyEvent,
    app: &mut AppState,
    plugin_manager: &mut PluginManager,
) -> bool {
    match app.plugin_modal_mode {
        PluginModalMode::Local => match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('p') => {
                app.show_plugin_modal = false;
                false
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                match plugin_manager.reload_all() {
                    Ok(n) => {
                        sync_plugin_status(app, plugin_manager);
                        app.add_success(format!("{} plugin(s) reloaded", n));
                    }
                    Err(e) => app.add_error(format!("Plugin reload error: {}", e)),
                }
                false
            }
            KeyCode::Up => {
                if app.plugin_modal_scroll > 0 {
                    app.plugin_modal_scroll = app.plugin_modal_scroll.saturating_sub(1);
                }
                false
            }
            KeyCode::Down => {
                let max = app.plugin_statuses.len().saturating_sub(1);
                if app.plugin_modal_scroll < max {
                    app.plugin_modal_scroll += 1;
                }
                false
            }
            _ => false,
        },
        PluginModalMode::Registry => match key.code {
            KeyCode::Esc => {
                app.show_plugin_modal = false;
                false
            }
            KeyCode::Char('q') => {
                app.show_plugin_modal = false;
                false
            }
            KeyCode::Up => {
                if app.registry_scroll > 0 {
                    app.registry_scroll = app.registry_scroll.saturating_sub(1);
                }
                false
            }
            KeyCode::Down => {
                let query = app.registry_search_query.to_lowercase();
                let filtered_count = if query.is_empty() {
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
                };
                let max = filtered_count.saturating_sub(1);
                if app.registry_scroll < max {
                    app.registry_scroll += 1;
                }
                false
            }
            KeyCode::Enter => {
                let query = app.registry_search_query.to_lowercase();
                let filtered: Vec<&tuiserial_core::RegistryEntry> = if query.is_empty() {
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
                };
                if let Some(entry) = filtered.get(app.registry_scroll) {
                    let target = plugin_manager.plugin_dir().join(&entry.name);
                    if target.exists() {
                        app.add_info(format!("{}: already installed", entry.name));
                    } else {
                        match plugin_manager.install_plugin_from_cache(&entry.name) {
                            Ok(()) => {
                                app.add_success(
                                    t!("notify.plugin_installed", name = &entry.name),
                                );
                                sync_plugin_status(app, plugin_manager);
                            }
                            Err(e) => app.add_error(
                                t!("notify.plugin_install_failed", error = &e.to_string()),
                            ),
                        }
                    }
                }
                false
            }
            KeyCode::Backspace => {
                app.registry_search_query.pop();
                app.registry_scroll = 0;
                false
            }
            KeyCode::Char(c) => {
                app.registry_search_query.push(c);
                app.registry_scroll = 0;
                false
            }
            _ => false,
        },
    }
}

/// Handle key events while the plugin modal is open (plugin feature disabled).
/// Navigation still works; plugin actions show a guidance message.
#[cfg(not(feature = "plugin"))]
fn handle_plugin_modal_key(key: KeyEvent, app: &mut AppState) -> bool {
    match app.plugin_modal_mode {
        PluginModalMode::Local => match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('p') => {
                app.show_plugin_modal = false;
                false
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                app.add_error(t!("notify.plugin_disabled").to_string());
                false
            }
            KeyCode::Up => {
                if app.plugin_modal_scroll > 0 {
                    app.plugin_modal_scroll = app.plugin_modal_scroll.saturating_sub(1);
                }
                false
            }
            KeyCode::Down => {
                let max = app.plugin_statuses.len().saturating_sub(1);
                if app.plugin_modal_scroll < max {
                    app.plugin_modal_scroll += 1;
                }
                false
            }
            _ => false,
        },
        PluginModalMode::Registry => match key.code {
            KeyCode::Esc => {
                app.show_plugin_modal = false;
                false
            }
            KeyCode::Char('q') => {
                app.show_plugin_modal = false;
                false
            }
            KeyCode::Up => {
                if app.registry_scroll > 0 {
                    app.registry_scroll = app.registry_scroll.saturating_sub(1);
                }
                false
            }
            KeyCode::Down => {
                let query = app.registry_search_query.to_lowercase();
                let filtered_count = if query.is_empty() {
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
                };
                let max = filtered_count.saturating_sub(1);
                if app.registry_scroll < max {
                    app.registry_scroll += 1;
                }
                false
            }
            KeyCode::Enter => {
                app.add_error(t!("notify.plugin_disabled").to_string());
                false
            }
            KeyCode::Backspace => {
                app.registry_search_query.pop();
                app.registry_scroll = 0;
                false
            }
            KeyCode::Char(c) => {
                app.registry_search_query.push(c);
                app.registry_scroll = 0;
                false
            }
            _ => false,
        },
    }
}
