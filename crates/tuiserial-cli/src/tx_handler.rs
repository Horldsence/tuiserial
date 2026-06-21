//! TX input keyboard handler — handles key events when the focus is on the TX input field.

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use rust_i18n::t;
use tuiserial_core::{AppState, TxMode};
#[cfg(feature = "plugin")]
use tuiserial_plugin::PluginManager;

use crate::handler::SerialHandler;
use crate::input_utils::rebuild_hex_input;

/// Handle key events when the TX input field is focused.
/// Returns `true` if the application should exit.
pub fn handle_tx_key_event(
    key: KeyEvent,
    app: &mut AppState,
    handler: &mut SerialHandler,
    #[cfg(feature = "plugin")] plugin_manager: &mut PluginManager,
) -> bool {
    if key.kind != KeyEventKind::Press {
        return false;
    }

    match key.code {
        KeyCode::Tab => {
            app.focus_next_field();
            false
        }
        KeyCode::BackTab => {
            app.focus_prev_field();
            false
        }
        KeyCode::Char(c) => {
            if app.tx_mode == TxMode::Hex {
                match c {
                    '0'..='9' | 'a'..='f' | 'A'..='F' => {
                        let upper = c.to_ascii_uppercase();
                        let byte_idx = app
                            .tx_input
                            .char_indices()
                            .nth(app.tx_cursor)
                            .map(|(i, _)| i)
                            .unwrap_or(app.tx_input.len());
                        app.tx_input.insert(byte_idx, upper);
                        app.tx_cursor += 1;
                        rebuild_hex_input(app);
                    }
                    ' ' => {}
                    _ => {}
                }
            } else {
                let byte_idx = app
                    .tx_input
                    .char_indices()
                    .nth(app.tx_cursor)
                    .map(|(i, _)| i)
                    .unwrap_or(app.tx_input.len());
                app.tx_input.insert(byte_idx, c);
                app.tx_cursor += 1;
            }
            false
        }
        KeyCode::Backspace => {
            if app.tx_cursor > 0 {
                let byte_idx = app
                    .tx_input
                    .char_indices()
                    .nth(app.tx_cursor - 1)
                    .map(|(i, _)| i)
                    .unwrap_or(0);
                if byte_idx < app.tx_input.len() {
                    app.tx_input.remove(byte_idx);
                }
                app.tx_cursor -= 1;
                if app.tx_mode == TxMode::Hex {
                    rebuild_hex_input(app);
                }
            }
            false
        }
        KeyCode::Up => {
            app.toggle_tx_mode();
            app.add_info(format!(
                "{}: {}",
                t!("notify.tx_mode"),
                match app.tx_mode {
                    TxMode::Hex => "HEX",
                    TxMode::Ascii => "ASCII",
                }
            ));
            false
        }
        KeyCode::Down => {
            app.toggle_tx_mode();
            app.add_info(format!(
                "{}: {}",
                t!("notify.tx_mode"),
                match app.tx_mode {
                    TxMode::Hex => "HEX",
                    TxMode::Ascii => "ASCII",
                }
            ));
            false
        }
        KeyCode::Delete => {
            let char_count = app.tx_input.chars().count();
            if app.tx_cursor < char_count {
                let byte_idx = app
                    .tx_input
                    .char_indices()
                    .nth(app.tx_cursor)
                    .map(|(i, _)| i)
                    .unwrap_or(app.tx_input.len());
                if byte_idx < app.tx_input.len() {
                    app.tx_input.remove(byte_idx);
                }
                if app.tx_mode == TxMode::Hex {
                    rebuild_hex_input(app);
                }
            }
            false
        }
        KeyCode::Left => {
            if app.tx_cursor > 0 {
                app.tx_cursor -= 1;
            }
            false
        }
        KeyCode::Right => {
            let char_count = app.tx_input.chars().count();
            if app.tx_cursor < char_count {
                app.tx_cursor += 1;
            }
            false
        }
        KeyCode::Home => {
            app.tx_cursor = 0;
            false
        }
        KeyCode::End => {
            app.tx_cursor = app.tx_input.chars().count();
            false
        }
        KeyCode::Enter => {
            if !app.tx_input.is_empty() {
                if handler.is_connected() {
                    let mut bytes: Result<Vec<u8>, tuiserial_serial::SerialError> =
                        match app.tx_mode {
                            TxMode::Ascii => Ok(app.tx_input.as_bytes().to_vec()),
                            TxMode::Hex => tuiserial_serial::hex_to_bytes(&app.tx_input),
                        };

                    if let Ok(ref mut data) = bytes {
                        data.extend_from_slice(app.tx_append_mode.as_bytes());
                    }

                    match bytes {
                        Ok(data) => {
                            #[cfg(feature = "plugin")]
                            let (processed, suppressed) =
                                plugin_manager.process_tx(data, &app.config);
                            #[cfg(not(feature = "plugin"))]
                            let (processed, suppressed) = (data, false);

                            if suppressed {
                                app.add_info("TX suppressed by plugin".to_string());
                                app.tx_input.clear();
                                app.tx_cursor = 0;
                                return false;
                            }
                            match handler.send(&processed) {
                                Ok(_sent) => {
                                    app.message_log.push_tx(processed.clone());
                                    let append_info = if app.tx_append_mode.as_bytes().is_empty() {
                                        String::new()
                                    } else {
                                        format!(" + {}", app.tx_append_mode.name())
                                    };
                                    app.add_success(format!(
                                        "{}{}",
                                        t!("notify.send_success"),
                                        append_info
                                    ));
                                    app.tx_input.clear();
                                    app.tx_cursor = 0;
                                    if app.auto_scroll {
                                        let lines_count = app.message_log.entries.len() as u16;
                                        app.scroll_offset = lines_count.saturating_sub(1);
                                    }
                                }
                                Err(e) => {
                                    app.add_error(format!("{}: {}", t!("notify.send_failed"), e));
                                }
                            }
                        }
                        Err(e) => {
                            app.add_error(format!("{}: {}", t!("notify.hex_format_error"), e));
                        }
                    }
                } else {
                    app.add_error(t!("notify.not_connected").to_string());
                }
            } else {
                app.add_warning(t!("notify.input_empty").to_string());
            }
            false
        }
        KeyCode::Esc => {
            app.tx_input.clear();
            app.tx_cursor = 0;
            false
        }
        _ => false,
    }
}
