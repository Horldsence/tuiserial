//! Keyboard event handler — routes key events to the appropriate sub-handler.

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use rust_i18n::t;
use tuiserial_core::{AppState, FocusedField, MenuState, PluginModalMode, menu_def::MENU_BAR};

use crate::handler::SerialHandler;
use crate::menu_handler::handle_menu_action;
use crate::plugin_adapter::PluginProxy;
use crate::plugin_adapter::filtered_registry_count;

/// Main keyboard event handler. Routes to sub-handlers based on application state.
/// Returns `true` if the application should exit.
pub fn handle_key_event(
    key: KeyEvent,
    app: &mut AppState,
    handler: &mut SerialHandler,
    plugin_proxy: &mut PluginProxy,
) -> bool {
    if key.kind != KeyEventKind::Press {
        return false;
    }

    // Menu navigation takes priority over everything else
    if let Some(exit) = handle_menu_navigation(key, app, handler, plugin_proxy) {
        return exit;
    }

    // Plugin modal keyboard
    if app.show_plugin_modal {
        return handle_plugin_modal_key(key, app, plugin_proxy);
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
        return crate::tx_handler::handle_tx_key_event(key, app, handler, plugin_proxy);
    }

    // Global shortcuts
    return crate::global_handler::handle_global_key(key, app, handler, plugin_proxy);
}

/// Handle menu bar and dropdown navigation. Returns `Some(exit)` when a key is handled
/// by the menu system, or `None` to let other handlers process the key.
fn handle_menu_navigation(
    key: KeyEvent,
    app: &mut AppState,
    handler: &mut SerialHandler,
    plugin_proxy: &mut PluginProxy,
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
                    Err(e) => app.add_error(format!("{}: {}", t!("notify.config_save_failed"), e)),
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
                    let should_exit =
                        handle_menu_action(app, handler, plugin_proxy, menu_idx, item_idx);
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
/// All plugin-specific actions are delegated to `PluginProxy`.
fn handle_plugin_modal_key(
    key: KeyEvent,
    app: &mut AppState,
    plugin_proxy: &mut PluginProxy,
) -> bool {
    match app.plugin_modal_mode {
        PluginModalMode::Local => match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('p') => {
                app.show_plugin_modal = false;
                false
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                plugin_proxy.reload_all_plugins(app);
                false
            }
            KeyCode::Char('e') => {
                plugin_proxy.enable_plugin_action(app);
                false
            }
            KeyCode::Char('d') => {
                plugin_proxy.disable_plugin_action(app);
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
            KeyCode::Esc | KeyCode::Char('q') => {
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
                let max = filtered_registry_count(app).saturating_sub(1);
                if app.registry_scroll < max {
                    app.registry_scroll += 1;
                }
                false
            }
            KeyCode::Enter => {
                plugin_proxy.install_from_registry(app);
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
