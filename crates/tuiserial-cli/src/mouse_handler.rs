//! Mouse event handler — handles mouse clicks, scroll, and drag events.

use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;
use tuiserial_core::{i18n::t, menu_def::MENU_BAR, AppState, DisplayMode, FocusedField, MenuState};
use tuiserial_ui::{get_clicked_field, get_ui_areas, is_inside, find_clicked_menu};
#[cfg(feature = "plugin")]
use tuiserial_plugin::PluginManager;

use crate::handler::SerialHandler;
use crate::input_utils::display_width;
use crate::menu_handler::handle_menu_action;

/// Handle mouse events (click, scroll, drag).
pub fn handle_mouse_event(
    mouse: MouseEvent,
    app: &mut AppState,
    handler: &mut SerialHandler,
    #[cfg(feature = "plugin")]
    plugin_manager: &mut PluginManager,
) {
    let col = mouse.column;
    let row = mouse.row;

    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            #[cfg(feature = "plugin")]
            handle_left_click(col, row, app, handler, plugin_manager);
            #[cfg(not(feature = "plugin"))]
            handle_left_click(col, row, app, handler);
        }
        MouseEventKind::Down(MouseButton::Right) => {
            handle_right_click(col, row, app);
        }
        MouseEventKind::Down(MouseButton::Middle) => {
            handle_middle_click(col, row, app);
        }
        MouseEventKind::ScrollUp => {
            handle_scroll_up(col, row, app);
        }
        MouseEventKind::ScrollDown => {
            handle_scroll_down(col, row, app);
        }
        MouseEventKind::Drag(MouseButton::Left) => {}
        _ => {}
    }
}

fn handle_left_click(
    col: u16,
    row: u16,
    app: &mut AppState,
    handler: &mut SerialHandler,
    #[cfg(feature = "plugin")]
    plugin_manager: &mut PluginManager,
) {
    let areas = get_ui_areas();

    if app.show_plugin_modal && !is_inside(areas.plugin_modal, col, row) {
        app.show_plugin_modal = false;
        return;
    }

    if is_inside(areas.menu_bar, col, row) {
        if let Some(menu_idx) = find_clicked_menu(col, row, areas.menu_bar, app.language) {
            match app.menu_state {
                MenuState::Dropdown(current_idx, _) if current_idx == menu_idx => {
                    app.menu_state = MenuState::None;
                }
                _ => {
                    app.menu_state = MenuState::Dropdown(menu_idx, 0);
                    app.focused_field = FocusedField::LogArea;
                }
            }
        }
        return;
    }

    if let MenuState::Dropdown(menu_idx, _) = app.menu_state {
        let menu = match MENU_BAR.get_menu(menu_idx) {
            Some(m) => m,
            None => {
                app.menu_state = MenuState::None;
                return;
            }
        };

        let items: Vec<String> = menu
            .items
            .iter()
            .map(|action| {
                if action.is_separator() {
                    String::new()
                } else {
                    t(action.label_key(), app.language).to_string()
                }
            })
            .collect();

        let max_width = items
            .iter()
            .map(|s| display_width(s.as_str()))
            .max()
            .unwrap_or(10) as u16
            + 6;
        let height = items.len() as u16 + 2;

        let x_offset = tuiserial_core::menu_def::calculate_menu_x_offset(menu_idx, app.language);

        let dropdown_area = Rect {
            x: areas.menu_bar.x + x_offset,
            y: areas.menu_bar.y + 1,
            width: max_width,
            height,
        };

        if is_inside(dropdown_area, col, row) {
            let relative_y = row - dropdown_area.y;
            if relative_y > 0 && relative_y <= items.len() as u16 {
                let item_idx = (relative_y - 1) as usize;

                if let Some(action) = MENU_BAR.get_action(menu_idx, item_idx)
                    && !action.is_separator() {
                        #[cfg(feature = "plugin")]
                        handle_menu_action(app, handler, plugin_manager, menu_idx, item_idx);
                        #[cfg(not(feature = "plugin"))]
                        handle_menu_action(app, handler, menu_idx, item_idx);
                        app.menu_state = MenuState::None;
                    }
                }
            return;
        } else {
            app.menu_state = MenuState::None;
            return;
        }
    }

    if let Some(field) = get_clicked_field(col, row) {
        app.focused_field = field;

        let areas = get_ui_areas();
        match field {
            FocusedField::Port => {
                if !app.can_modify_config() {
                    app.add_warning(t("notify.config_locked_warning", app.language).to_string());
                } else if !app.ports.is_empty() && is_inside(areas.port, col, row) {
                    let relative_row = row.saturating_sub(areas.port.y + 1);
                    if relative_row < app.ports.len() as u16
                        && app.select_port(relative_row as usize)
                    {
                        app.add_info(format!(
                            "{}: {}",
                            t("notify.port_selected", app.language),
                            app.config.port
                        ));
                    }
                }
            }
            FocusedField::BaudRate => {
                if !app.can_modify_config() {
                    app.add_warning(t("notify.config_locked_warning", app.language).to_string());
                } else if is_inside(areas.baud_rate, col, row) {
                    let relative_row = row.saturating_sub(areas.baud_rate.y + 1);
                    if relative_row < app.baud_rate_options.len() as u16 {
                        app.baud_rate_state.select(Some(relative_row as usize));
                        app.config.baud_rate = app.baud_rate_options[relative_row as usize];
                        app.add_info(format!("波特率: {}", app.config.baud_rate));
                    }
                }
            }
            FocusedField::DataBits => {
                if !app.can_modify_config() {
                    app.add_warning(t("notify.config_locked_warning", app.language).to_string());
                } else if is_inside(areas.data_bits, col, row) {
                    let relative_row = row.saturating_sub(areas.data_bits.y + 1);
                    if relative_row < app.data_bits_options.len() as u16 {
                        app.data_bits_state.select(Some(relative_row as usize));
                        app.config.data_bits = app.data_bits_options[relative_row as usize];
                        app.add_info(format!("数据位: {}", app.config.data_bits));
                    }
                }
            }
            FocusedField::Parity => {
                if !app.can_modify_config() {
                    app.add_warning(t("notify.config_locked_warning", app.language).to_string());
                } else if is_inside(areas.parity, col, row) {
                    let relative_row = row.saturating_sub(areas.parity.y + 1);
                    if relative_row < app.parity_options.len() as u16 {
                        app.parity_state.select(Some(relative_row as usize));
                        app.config.parity = app.parity_options[relative_row as usize];
                        app.add_info(format!(
                            "{}: {:?}",
                            t("notify.parity", app.language),
                            app.config.parity
                        ));
                    }
                }
            }
            FocusedField::StopBits => {
                if !app.can_modify_config() {
                    app.add_warning(t("notify.config_locked_warning", app.language).to_string());
                } else if is_inside(areas.stop_bits, col, row) {
                    let relative_row = row.saturating_sub(areas.stop_bits.y + 1);
                    if relative_row < app.stop_bits_options.len() as u16 {
                        app.stop_bits_state.select(Some(relative_row as usize));
                        app.config.stop_bits = app.stop_bits_options[relative_row as usize];
                        app.add_info(format!("停止位: {}", app.config.stop_bits));
                    }
                }
            }
            FocusedField::FlowControl => {
                if !app.can_modify_config() {
                    app.add_warning(t("notify.config_locked_warning", app.language).to_string());
                } else if is_inside(areas.flow_control, col, row) {
                    let relative_row = row.saturating_sub(areas.flow_control.y + 1);
                    if relative_row < app.flow_control_options.len() as u16 {
                        app.flow_control_state.select(Some(relative_row as usize));
                        app.config.flow_control = app.flow_control_options[relative_row as usize];
                        app.add_info(format!(
                            "{}: {:?}",
                            t("notify.flow_control", app.language),
                            app.config.flow_control
                        ));
                    }
                }
            }
            FocusedField::TxInput => {
                let areas = get_ui_areas();
                if is_inside(areas.tx_area, col, row) {
                    let tx_input_width = areas.tx_area.width.saturating_sub(12);
                    let relative_col = col.saturating_sub(areas.tx_area.x);

                    if relative_col >= tx_input_width {
                        let relative_row = row.saturating_sub(areas.tx_area.y + 1);
                        if relative_row < app.append_mode_options.len() as u16 {
                            app.append_mode_state.select(Some(relative_row as usize));
                            app.tx_append_mode = app.append_mode_options[relative_row as usize];
                            app.add_info(format!(
                                "{}: {}",
                                t("notify.append_mode", app.language),
                                app.tx_append_mode.name(app.language)
                            ));
                        }
                    } else {
                        let char_count = app.tx_input.chars().count();
                        let cursor_pos = relative_col.saturating_sub(1).min(char_count as u16) as usize;
                        app.tx_cursor = cursor_pos;
                    }
                }
            }
            _ => {}
        }
    }
}

fn handle_right_click(col: u16, row: u16, app: &mut AppState) {
    let areas = get_ui_areas();

    if is_inside(areas.log_area, col, row) {
        app.toggle_display_mode();
        let mode_str = match app.display_mode {
            DisplayMode::Hex => "HEX",
            DisplayMode::Text => "TEXT",
        };
        app.add_info(format!(
            "{}: {}",
            t("notify.toggle_display_mode", app.language),
            mode_str
        ));
    } else if is_inside(areas.tx_area, col, row) {
        let tx_input_width = areas.tx_area.width.saturating_sub(12);
        let relative_col = col.saturating_sub(areas.tx_area.x);

        if relative_col >= tx_input_width {
            app.next_append_mode();
            app.add_info(format!(
                "{}: {}",
                t("notify.append_mode", app.language),
                app.tx_append_mode.name(app.language)
            ));
        } else {
            app.toggle_tx_mode();
            app.add_info(format!(
                "{}: {}",
                t("notify.tx_mode", app.language),
                match app.tx_mode {
                    tuiserial_core::TxMode::Hex => "HEX",
                    tuiserial_core::TxMode::Ascii => "ASCII",
                }
            ));
        }
    } else if is_inside(areas.control_area, col, row) {
        app.auto_scroll = !app.auto_scroll;
        let status = if app.auto_scroll { "启用" } else { "禁用" };
        app.add_info(format!("自动滚动: {}", status));
    }
}

fn handle_middle_click(col: u16, row: u16, app: &mut AppState) {
    let areas = get_ui_areas();

    if is_inside(areas.log_area, col, row) {
        app.message_log.clear();
        app.add_info(t("notify.log_cleared", app.language).to_string());
    } else if is_inside(areas.tx_area, col, row) {
        app.tx_input.clear();
        app.tx_cursor = 0;
        app.add_info("已清空输入");
    }
}

fn handle_scroll_up(col: u16, row: u16, app: &mut AppState) {
    let areas = get_ui_areas();

    if is_inside(areas.log_area, col, row) {
        app.auto_scroll = false;
        app.scroll_offset = app.scroll_offset.saturating_sub(3);
    } else if is_inside(areas.port, col, row) {
        if !app.can_modify_config() {
            app.add_warning(t("notify.config_locked_warning", app.language).to_string());
        } else if let Some(idx) = app.port_list_state.selected() {
            let new_idx = if idx > 0 {
                idx - 1
            } else {
                app.ports.len().saturating_sub(1)
            };
            app.select_port(new_idx);
        }
    } else if is_inside(areas.baud_rate, col, row) {
        if !app.can_modify_config() {
            app.add_warning(t("notify.config_locked_warning", app.language).to_string());
        } else {
            app.prev_baud_rate();
        }
    } else if is_inside(areas.tx_area, col, row) {
        app.prev_append_mode();
    } else if is_inside(areas.plugin_modal, col, row)
        && app.plugin_modal_scroll > 0 {
            app.plugin_modal_scroll = app.plugin_modal_scroll.saturating_sub(1);
        }
    }

fn handle_scroll_down(col: u16, row: u16, app: &mut AppState) {
    let areas = get_ui_areas();

    if is_inside(areas.log_area, col, row) {
        app.scroll_offset = app.scroll_offset.saturating_add(3);

        let lines = app.message_log.entries.len() as u16;
        let viewport_lines = areas.log_area.height.saturating_sub(2).max(1);
        let max_scroll = lines.saturating_sub(viewport_lines);
        if app.scroll_offset >= max_scroll {
            app.auto_scroll = true;
        }
    } else if is_inside(areas.port, col, row) {
        if !app.can_modify_config() {
            app.add_warning(t("notify.config_locked_warning", app.language).to_string());
        } else if let Some(idx) = app.port_list_state.selected() {
            let new_idx = if idx < app.ports.len().saturating_sub(1) {
                idx + 1
            } else {
                0
            };
            app.select_port(new_idx);
        }
    } else if is_inside(areas.baud_rate, col, row) {
        if !app.can_modify_config() {
            app.add_warning(t("notify.config_locked_warning", app.language).to_string());
        } else {
            app.next_baud_rate();
        }
    } else if is_inside(areas.tx_area, col, row) {
        app.next_append_mode();
    } else if is_inside(areas.plugin_modal, col, row) {
        let max = app.plugin_statuses.len().saturating_sub(1);
        if app.plugin_modal_scroll < max {
            app.plugin_modal_scroll += 1;
        }
    }
}
