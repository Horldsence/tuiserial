//! Global keyboard shortcut handler — handles keys when no modal is open and no text input is focused.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use rust_i18n::t;
use tuiserial_core::{AppState, DisplayMode, FocusedField};
use tuiserial_serial::list_ports;

use crate::handler::SerialHandler;
use crate::plugin_adapter::PluginProxy;

/// Handle global keyboard shortcuts (outside TX input, menu, or modals).
/// Returns `true` if the application should exit.
pub fn handle_global_key(
    key: KeyEvent,
    app: &mut AppState,
    handler: &mut SerialHandler,
    plugin_proxy: &mut PluginProxy,
) -> bool {
    match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => true,
        KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => true,
        KeyCode::Char('q') | KeyCode::Esc => true,

        KeyCode::Char('p') | KeyCode::Char('P') => {
            if app.show_plugin_modal {
                app.show_plugin_modal = false;
            } else {
                plugin_proxy.sync_status(app);
                app.plugin_modal_mode = tuiserial_core::PluginModalMode::Local;
                app.show_plugin_modal = true;
                app.plugin_modal_scroll = 0;
            }
            false
        }

        KeyCode::Char('o') => {
            if handler.is_connected() {
                for err in plugin_proxy.on_disconnect() {
                    app.record_error(err);
                }
                handler.disconnect();
                app.is_connected = false;
                app.unlock_config();
                log::info!("Disconnected from {}", app.config.port);
                app.add_info(t!("notify.disconnected_unlocked").to_string());
            } else {
                if app.config.port.is_empty() {
                    app.add_error(t!("notify.please_select_port").to_string());
                } else {
                    match handler.connect(app) {
                        Ok(_) => {
                            app.is_connected = true;
                            app.lock_config();
                            for err in plugin_proxy.on_connect(&app.config) {
                                app.record_error(err);
                            }
                            log::info!(
                                "Connected to {} @ {} baud",
                                app.config.port,
                                app.config.baud_rate
                            );
                            app.add_success(
                                t!("notify.connected_locked", port = &app.config.port).to_string(),
                            );
                        }
                        Err(e) => {
                            app.is_connected = false;
                            app.unlock_config();
                            let kind: tuiserial_core::SerialErrorKind = e.into();
                            app.record_error(tuiserial_core::AppError::Serial {
                                kind,
                                ctx: tuiserial_core::ErrorContext::new(
                                    "serial",
                                    "connect",
                                    tuiserial_core::RecoveryStrategy::Retry,
                                ),
                            });
                        }
                    }
                }
            }
            false
        }

        KeyCode::Tab => {
            app.focus_next_field();
            false
        }
        KeyCode::BackTab => {
            app.focus_prev_field();
            false
        }

        KeyCode::Up | KeyCode::Char('k') => {
            handle_field_up(app);
            false
        }
        KeyCode::Down | KeyCode::Char('j') => {
            handle_field_down(app);
            false
        }

        KeyCode::Right | KeyCode::Char('l') => {
            if app.focused_field == FocusedField::BaudRate && !app.next_baud_rate() {
                app.add_warning(t!("notify.config_locked_warning").to_string());
            }
            false
        }
        KeyCode::Left | KeyCode::Char('h') => {
            if app.focused_field == FocusedField::BaudRate && !app.prev_baud_rate() {
                app.add_warning(t!("notify.config_locked_warning").to_string());
            }
            false
        }

        KeyCode::Char('x') => {
            app.toggle_display_mode();
            let mode_str = match app.display_mode {
                DisplayMode::Hex => "HEX",
                DisplayMode::Text => "TEXT",
            };
            app.add_info(format!(
                "{}: {}",
                t!("notify.toggle_display_mode"),
                mode_str
            ));
            false
        }

        KeyCode::Char('a') => {
            app.auto_scroll = !app.auto_scroll;
            let status = if app.auto_scroll {
                t!("notify.enabled")
            } else {
                t!("notify.disabled")
            };
            app.add_info(format!("{}: {}", t!("notify.auto_scroll"), status));
            false
        }

        KeyCode::Char('c') => {
            app.message_log.clear();
            app.add_info(t!("notify.log_cleared").to_string());
            false
        }

        KeyCode::Char('f') => {
            if app.toggle_flow_control() {
                let flow_str = format!("{:?}", app.config.flow_control);
                app.add_info(format!("{}: {}", t!("notify.flow_control"), flow_str));
            } else {
                app.add_warning(t!("notify.config_locked_warning").to_string());
            }
            false
        }

        KeyCode::Char('n') => {
            app.next_append_mode();
            app.add_info(format!(
                "{}: {}",
                t!("notify.append_mode"),
                app.tx_append_mode.name()
            ));
            false
        }

        KeyCode::Char('r') => {
            app.ports = list_ports();
            if !app.ports.is_empty() && app.port_list_state.selected().is_none() {
                app.port_list_state.select(Some(0));
                app.config.port = app.ports[0].clone();
            }
            app.add_success(t!("notify.ports_refreshed").to_string());
            false
        }

        KeyCode::PageUp => {
            app.auto_scroll = false;
            app.scroll_offset = app.scroll_offset.saturating_sub(10);
            false
        }
        KeyCode::PageDown => {
            app.scroll_offset = app.scroll_offset.saturating_add(10);
            false
        }
        KeyCode::Home => {
            app.auto_scroll = false;
            app.scroll_offset = 0;
            false
        }
        KeyCode::End => {
            app.auto_scroll = true;
            let lines = app.message_log.entries.len() as u16;
            app.scroll_offset = lines.saturating_sub(1);
            false
        }

        _ => false,
    }
}

fn handle_field_up(app: &mut AppState) {
    match app.focused_field {
        FocusedField::Port => {
            if !app.can_modify_config() {
                app.add_warning(t!("notify.config_locked_warning").to_string());
            } else if let Some(idx) = app.port_list_state.selected() {
                let new_idx = if idx > 0 {
                    idx - 1
                } else {
                    app.ports.len().saturating_sub(1)
                };
                if app.select_port(new_idx) {
                    app.add_info(format!(
                        "{}: {}",
                        t!("notify.port_selected"),
                        app.config.port
                    ));
                }
            }
        }
        FocusedField::BaudRate => {
            if !app.prev_baud_rate() {
                app.add_warning(t!("notify.config_locked_warning").to_string());
            }
        }
        FocusedField::DataBits => {
            if !app.can_modify_config() {
                app.add_warning(t!("notify.config_locked_warning").to_string());
            } else if let Some(idx) = app.data_bits_state.selected() {
                let new_idx = if idx > 0 {
                    idx - 1
                } else {
                    app.data_bits_options.len() - 1
                };
                app.data_bits_state.select(Some(new_idx));
                app.config.data_bits = app.data_bits_options[new_idx];
            }
        }
        FocusedField::Parity => {
            if !app.can_modify_config() {
                app.add_warning(t!("notify.config_locked_warning").to_string());
            } else if let Some(idx) = app.parity_state.selected() {
                let new_idx = if idx > 0 {
                    idx - 1
                } else {
                    app.parity_options.len() - 1
                };
                app.parity_state.select(Some(new_idx));
                app.config.parity = app.parity_options[new_idx];
            }
        }
        FocusedField::StopBits => {
            if !app.can_modify_config() {
                app.add_warning(t!("notify.config_locked_warning").to_string());
            } else if let Some(idx) = app.stop_bits_state.selected() {
                let new_idx = if idx > 0 {
                    idx - 1
                } else {
                    app.stop_bits_options.len() - 1
                };
                app.stop_bits_state.select(Some(new_idx));
                app.config.stop_bits = app.stop_bits_options[new_idx];
            }
        }
        FocusedField::FlowControl => {
            if !app.can_modify_config() {
                app.add_warning(t!("notify.config_locked_warning").to_string());
            } else if let Some(idx) = app.flow_control_state.selected() {
                let new_idx = if idx > 0 {
                    idx - 1
                } else {
                    app.flow_control_options.len() - 1
                };
                app.flow_control_state.select(Some(new_idx));
                app.config.flow_control = app.flow_control_options[new_idx];
            }
        }
        FocusedField::LogArea => {
            app.toggle_display_mode();
            let mode_str = match app.display_mode {
                DisplayMode::Hex => "HEX",
                DisplayMode::Text => "TEXT",
            };
            app.add_info(format!("{}: {}", t!("notify.display_mode"), mode_str));
        }
        _ => {}
    }
}

fn handle_field_down(app: &mut AppState) {
    match app.focused_field {
        FocusedField::Port => {
            if !app.can_modify_config() {
                app.add_warning(t!("notify.config_locked_warning").to_string());
            } else if let Some(idx) = app.port_list_state.selected() {
                let new_idx = if idx < app.ports.len().saturating_sub(1) {
                    idx + 1
                } else {
                    0
                };
                if app.select_port(new_idx) {
                    app.add_info(format!(
                        "{}: {}",
                        t!("notify.port_selected"),
                        app.config.port
                    ));
                }
            }
        }
        FocusedField::BaudRate => {
            if !app.next_baud_rate() {
                app.add_warning(t!("notify.config_locked_warning").to_string());
            }
        }
        FocusedField::DataBits => {
            if !app.can_modify_config() {
                app.add_warning(t!("notify.config_locked_warning").to_string());
            } else if let Some(idx) = app.data_bits_state.selected() {
                let new_idx = if idx < app.data_bits_options.len() - 1 {
                    idx + 1
                } else {
                    0
                };
                app.data_bits_state.select(Some(new_idx));
                app.config.data_bits = app.data_bits_options[new_idx];
            }
        }
        FocusedField::Parity => {
            if !app.can_modify_config() {
                app.add_warning(t!("notify.config_locked_warning").to_string());
            } else if let Some(idx) = app.parity_state.selected() {
                let new_idx = if idx < app.parity_options.len() - 1 {
                    idx + 1
                } else {
                    0
                };
                app.parity_state.select(Some(new_idx));
                app.config.parity = app.parity_options[new_idx];
            }
        }
        FocusedField::StopBits => {
            if !app.can_modify_config() {
                app.add_warning(t!("notify.config_locked_warning").to_string());
            } else if let Some(idx) = app.stop_bits_state.selected() {
                let new_idx = if idx < app.stop_bits_options.len() - 1 {
                    idx + 1
                } else {
                    0
                };
                app.stop_bits_state.select(Some(new_idx));
                app.config.stop_bits = app.stop_bits_options[new_idx];
            }
        }
        FocusedField::FlowControl => {
            if !app.can_modify_config() {
                app.add_warning(t!("notify.config_locked_warning").to_string());
            } else if let Some(idx) = app.flow_control_state.selected() {
                let new_idx = if idx < app.flow_control_options.len() - 1 {
                    idx + 1
                } else {
                    0
                };
                app.flow_control_state.select(Some(new_idx));
                app.config.flow_control = app.flow_control_options[new_idx];
            }
        }
        FocusedField::LogArea => {
            app.toggle_display_mode();
            let mode_str = match app.display_mode {
                DisplayMode::Hex => "HEX",
                DisplayMode::Text => "TEXT",
            };
            app.add_info(format!("{}: {}", t!("notify.display_mode"), mode_str));
        }
        _ => {}
    }
}
