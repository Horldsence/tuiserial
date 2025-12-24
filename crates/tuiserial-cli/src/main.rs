//! TuiSerial - Terminal User Interface for Serial Port Communication
//!
//! A terminal-based serial port communication tool with a user-friendly interface
//! and full mouse interaction support.

use std::io;
use std::time::Duration;

use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
        MouseButton, MouseEvent, MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use tuiserial_core::{AppState, DisplayMode, FocusedField, TxMode};
use tuiserial_serial::list_ports;
use tuiserial_ui::{draw, get_clicked_field, get_ui_areas, is_inside};

mod handler;
use handler::SerialHandler;

fn main() -> io::Result<()> {
    color_eyre::install().ok();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    let result = run_app(terminal);

    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;

    result
}

fn run_app(mut terminal: Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    let mut app = AppState::default();
    let mut handler = SerialHandler::new();

    // Initialize available ports
    app.ports = list_ports();
    if !app.ports.is_empty() {
        app.config.port = app.ports[0].clone();
        app.port_list_state.select(Some(0));
    }

    loop {
        app.update_notifications();
        terminal.draw(|f| draw(f, &app))?;

        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    if handle_key_event(key, &mut app, &mut handler) {
                        break;
                    }
                }
                Event::Mouse(mouse) => {
                    handle_mouse_event(mouse, &mut app);
                }
                Event::Resize(_, _) => {
                    // Terminal auto-redraws on resize
                }
                _ => {}
            }
        }

        // Try to read from serial port if connected
        if handler.is_connected() {
            if let Ok(data) = handler.read() {
                if !data.is_empty() {
                    app.message_log.push_rx(data.clone());
                    if app.auto_scroll {
                        let lines_count = app.message_log.entries.len() as u16;
                        app.scroll_offset = lines_count.saturating_sub(1);
                    }
                }
            }
        }
    }

    if handler.is_connected() {
        handler.disconnect();
    }

    Ok(())
}

fn handle_key_event(key: KeyEvent, app: &mut AppState, handler: &mut SerialHandler) -> bool {
    if key.kind != KeyEventKind::Press {
        return false;
    }

    // If we're in TX input mode, handle text input
    if app.focused_field == FocusedField::TxInput {
        match key.code {
            KeyCode::Tab => {
                app.focus_next_field();
                return false;
            }
            KeyCode::BackTab => {
                app.focus_prev_field();
                return false;
            }
            KeyCode::Char(c) => {
                app.tx_input.insert(app.tx_cursor, c);
                app.tx_cursor += 1;
                return false;
            }
            KeyCode::Backspace => {
                if app.tx_cursor > 0 {
                    app.tx_input.remove(app.tx_cursor - 1);
                    app.tx_cursor -= 1;
                }
                return false;
            }
            KeyCode::Up => {
                app.toggle_tx_mode();
                app.add_info(format!(
                    "发送模式: {}",
                    match app.tx_mode {
                        TxMode::Hex => "HEX",
                        TxMode::Ascii => "ASCII",
                    }
                ));
                return false;
            }
            KeyCode::Down => {
                app.toggle_tx_mode();
                app.add_info(format!(
                    "发送模式: {}",
                    match app.tx_mode {
                        TxMode::Hex => "HEX",
                        TxMode::Ascii => "ASCII",
                    }
                ));
                return false;
            }
            KeyCode::Delete => {
                if app.tx_cursor < app.tx_input.len() {
                    app.tx_input.remove(app.tx_cursor);
                }
                return false;
            }
            KeyCode::Left => {
                if app.tx_cursor > 0 {
                    app.tx_cursor -= 1;
                }
                return false;
            }
            KeyCode::Right => {
                if app.tx_cursor < app.tx_input.len() {
                    app.tx_cursor += 1;
                }
                return false;
            }
            KeyCode::Home => {
                app.tx_cursor = 0;
                return false;
            }
            KeyCode::End => {
                app.tx_cursor = app.tx_input.len();
                return false;
            }
            KeyCode::Enter => {
                // Send data
                if !app.tx_input.is_empty() {
                    if handler.is_connected() {
                        let mut bytes: Result<Vec<u8>, String> = match app.tx_mode {
                            TxMode::Ascii => Ok(app.tx_input.as_bytes().to_vec()),
                            TxMode::Hex => tuiserial_serial::hex_to_bytes(&app.tx_input),
                        };

                        // Append line ending if configured
                        if let Ok(ref mut data) = bytes {
                            data.extend_from_slice(app.tx_append_mode.as_bytes());
                        }

                        match bytes {
                            Ok(data) => match handler.send(&data) {
                                Ok(_sent) => {
                                    app.message_log.push_tx(data.clone());
                                    let append_info = if app.tx_append_mode.as_bytes().is_empty() {
                                        String::new()
                                    } else {
                                        format!(" + {}", app.tx_append_mode.name())
                                    };
                                    app.add_success(format!("已发送{}", append_info));
                                    app.tx_input.clear();
                                    app.tx_cursor = 0;
                                    if app.auto_scroll {
                                        let lines_count = app.message_log.entries.len() as u16;
                                        app.scroll_offset = lines_count.saturating_sub(1);
                                    }
                                }
                                Err(e) => {
                                    app.add_error(format!("发送失败: {}", e));
                                }
                            },
                            Err(e) => {
                                app.add_error(format!("HEX 格式错误: {}", e));
                            }
                        }
                    } else {
                        app.add_error("未连接串口");
                    }
                } else {
                    app.add_warning("输入内容为空");
                }
                return false;
            }
            KeyCode::Esc => {
                app.tx_input.clear();
                app.tx_cursor = 0;
                return false;
            }
            _ => {}
        }
        return false;
    }

    // Global/dropdown navigation
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => return true,

        // Connect/Disconnect
        KeyCode::Char('o') => {
            if handler.is_connected() {
                handler.disconnect();
                app.is_connected = false;
                app.unlock_config();
                app.add_info("已断开连接，配置已解锁");
            } else {
                // Validate configuration before connecting
                if app.config.port.is_empty() {
                    app.add_error("请先选择串口");
                } else {
                    match handler.connect(&app) {
                        Ok(_) => {
                            app.is_connected = true;
                            app.lock_config();
                            app.add_success(format!("已连接: {} (配置已锁定)", app.config.port));
                        }
                        Err(e) => {
                            app.is_connected = false;
                            app.unlock_config();
                            app.add_error(format!("连接失败: {}", e));
                        }
                    }
                }
            }
        }

        // Tab navigation between fields
        KeyCode::Tab => {
            app.focus_next_field();
        }
        KeyCode::BackTab => {
            app.focus_prev_field();
        }

        // Field-specific navigation - Up/Down
        KeyCode::Up | KeyCode::Char('k') => match app.focused_field {
            FocusedField::Port => {
                if !app.can_modify_config() {
                    app.add_warning("配置已锁定，请先断开连接");
                } else if let Some(idx) = app.port_list_state.selected() {
                    let new_idx = if idx > 0 {
                        idx - 1
                    } else {
                        app.ports.len().saturating_sub(1)
                    };
                    if app.select_port(new_idx) {
                        app.add_info(format!("选择串口: {}", app.config.port));
                    }
                }
            }
            FocusedField::BaudRate => {
                if !app.prev_baud_rate() {
                    app.add_warning("配置已锁定，请先断开连接");
                }
            }
            FocusedField::DataBits => {
                if !app.can_modify_config() {
                    app.add_warning("配置已锁定，请先断开连接");
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
                    app.add_warning("配置已锁定，请先断开连接");
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
                    app.add_warning("配置已锁定，请先断开连接");
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
                    app.add_warning("配置已锁定，请先断开连接");
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
                app.add_info(format!("显示模式: {}", mode_str));
            }
            _ => {}
        },

        KeyCode::Down | KeyCode::Char('j') => match app.focused_field {
            FocusedField::Port => {
                if !app.can_modify_config() {
                    app.add_warning("配置已锁定，请先断开连接");
                } else if let Some(idx) = app.port_list_state.selected() {
                    let new_idx = if idx < app.ports.len().saturating_sub(1) {
                        idx + 1
                    } else {
                        0
                    };
                    if app.select_port(new_idx) {
                        app.add_info(format!("选择串口: {}", app.config.port));
                    }
                }
            }
            FocusedField::BaudRate => {
                if !app.next_baud_rate() {
                    app.add_warning("配置已锁定，请先断开连接");
                }
            }
            FocusedField::DataBits => {
                if !app.can_modify_config() {
                    app.add_warning("配置已锁定，请先断开连接");
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
                    app.add_warning("配置已锁定，请先断开连接");
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
                    app.add_warning("配置已锁定，请先断开连接");
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
                    app.add_warning("配置已锁定，请先断开连接");
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
                app.add_info(format!("显示模式: {}", mode_str));
            }
            _ => {}
        },

        // Left/Right for BaudRate and other controls
        KeyCode::Right | KeyCode::Char('l') => match app.focused_field {
            FocusedField::BaudRate => {
                if !app.next_baud_rate() {
                    app.add_warning("配置已锁定，请先断开连接");
                }
            }
            _ => {}
        },

        KeyCode::Left | KeyCode::Char('h') => match app.focused_field {
            FocusedField::BaudRate => {
                if !app.prev_baud_rate() {
                    app.add_warning("配置已锁定，请先断开连接");
                }
            }
            _ => {}
        },

        // Display mode toggle (HEX/TEXT)
        KeyCode::Char('x') => {
            app.toggle_display_mode();
            let mode_str = match app.display_mode {
                DisplayMode::Hex => "HEX",
                DisplayMode::Text => "TEXT",
            };
            app.add_info(format!("切换显示模式: {}", mode_str));
        }

        // Auto scroll toggle
        KeyCode::Char('a') => {
            app.auto_scroll = !app.auto_scroll;
            let status = if app.auto_scroll { "启用" } else { "禁用" };
            app.add_info(format!("自动滚动: {}", status));
        }

        // Clear buffer
        KeyCode::Char('c') => {
            app.message_log.clear();
            app.add_info("已清空消息记录");
        }

        // Parity toggle
        KeyCode::Char('p') => {
            if app.toggle_parity() {
                let parity_str = format!("{:?}", app.config.parity);
                app.add_info(format!("校验位: {}", parity_str));
            } else {
                app.add_warning("配置已锁定，请先断开连接");
            }
        }

        // Flow control toggle
        KeyCode::Char('f') => {
            if app.toggle_flow_control() {
                let flow_str = format!("{:?}", app.config.flow_control);
                app.add_info(format!("流控: {}", flow_str));
            } else {
                app.add_warning("配置已锁定，请先断开连接");
            }
        }

        // Append mode cycle
        KeyCode::Char('n') => {
            app.next_append_mode();
            app.add_info(format!("追加: {}", app.tx_append_mode.name()));
        }

        // Refresh ports list
        KeyCode::Char('r') => {
            app.ports = list_ports();
            if !app.ports.is_empty() && app.port_list_state.selected().is_none() {
                app.port_list_state.select(Some(0));
                app.config.port = app.ports[0].clone();
            }
            app.add_success("已刷新串口列表");
        }

        // Scroll navigation
        KeyCode::PageUp => {
            app.auto_scroll = false;
            app.scroll_offset = app.scroll_offset.saturating_sub(10);
        }
        KeyCode::PageDown => {
            app.scroll_offset = app.scroll_offset.saturating_add(10);
        }

        KeyCode::Home => {
            app.auto_scroll = false;
            app.scroll_offset = 0;
        }
        KeyCode::End => {
            app.auto_scroll = true;
            let lines = app.message_log.entries.len() as u16;
            app.scroll_offset = lines.saturating_sub(1);
        }

        _ => {}
    }

    false
}

fn handle_mouse_event(mouse: MouseEvent, app: &mut AppState) {
    let col = mouse.column;
    let row = mouse.row;

    match mouse.kind {
        // Left click - focus field and handle selection
        MouseEventKind::Down(MouseButton::Left) => {
            // Try to find which field was clicked
            if let Some(field) = get_clicked_field(col, row) {
                app.focused_field = field;

                // Handle list item selection in dropdowns
                let areas = get_ui_areas();
                match field {
                    FocusedField::Port => {
                        if !app.can_modify_config() {
                            app.add_warning("配置已锁定，请先断开连接");
                        } else if !app.ports.is_empty() && is_inside(areas.port, col, row) {
                            // Calculate which port was clicked (considering borders)
                            let relative_row = row.saturating_sub(areas.port.y + 1);
                            if relative_row < app.ports.len() as u16 {
                                if app.select_port(relative_row as usize) {
                                    app.add_info(format!("选择串口: {}", app.config.port));
                                }
                            }
                        }
                    }
                    FocusedField::BaudRate => {
                        if !app.can_modify_config() {
                            app.add_warning("配置已锁定，请先断开连接");
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
                            app.add_warning("配置已锁定，请先断开连接");
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
                            app.add_warning("配置已锁定，请先断开连接");
                        } else if is_inside(areas.parity, col, row) {
                            let relative_row = row.saturating_sub(areas.parity.y + 1);
                            if relative_row < app.parity_options.len() as u16 {
                                app.parity_state.select(Some(relative_row as usize));
                                app.config.parity = app.parity_options[relative_row as usize];
                                app.add_info(format!("校验位: {:?}", app.config.parity));
                            }
                        }
                    }
                    FocusedField::StopBits => {
                        if !app.can_modify_config() {
                            app.add_warning("配置已锁定，请先断开连接");
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
                            app.add_warning("配置已锁定，请先断开连接");
                        } else if is_inside(areas.flow_control, col, row) {
                            let relative_row = row.saturating_sub(areas.flow_control.y + 1);
                            if relative_row < app.flow_control_options.len() as u16 {
                                app.flow_control_state.select(Some(relative_row as usize));
                                app.config.flow_control =
                                    app.flow_control_options[relative_row as usize];
                                app.add_info(format!("流控: {:?}", app.config.flow_control));
                            }
                        }
                    }
                    FocusedField::TxInput => {
                        // Position cursor based on click position or select append mode
                        let areas = get_ui_areas();
                        if is_inside(areas.tx_area, col, row) {
                            // Check if click is in the right portion (append selector)
                            let tx_input_width = areas.tx_area.width.saturating_sub(12);
                            let relative_col = col.saturating_sub(areas.tx_area.x);

                            if relative_col >= tx_input_width {
                                // Clicked in append selector area
                                let relative_row = row.saturating_sub(areas.tx_area.y + 1);
                                if relative_row < app.append_mode_options.len() as u16 {
                                    app.append_mode_state.select(Some(relative_row as usize));
                                    app.tx_append_mode =
                                        app.append_mode_options[relative_row as usize];
                                    app.add_info(format!("追加: {}", app.tx_append_mode.name()));
                                }
                            } else {
                                // Clicked in input area - position cursor
                                let cursor_pos = relative_col
                                    .saturating_sub(1)
                                    .min(app.tx_input.len() as u16)
                                    as usize;
                                app.tx_cursor = cursor_pos;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        // Right click - context menu actions
        MouseEventKind::Down(MouseButton::Right) => {
            let areas = get_ui_areas();

            if is_inside(areas.log_area, col, row) {
                // Right click in log area - toggle display mode
                app.toggle_display_mode();
                let mode_str = match app.display_mode {
                    DisplayMode::Hex => "HEX",
                    DisplayMode::Text => "TEXT",
                };
                app.add_info(format!("切换显示模式: {}", mode_str));
            } else if is_inside(areas.tx_area, col, row) {
                // Right click in TX area - check which part
                let tx_input_width = areas.tx_area.width.saturating_sub(12);
                let relative_col = col.saturating_sub(areas.tx_area.x);

                if relative_col >= tx_input_width {
                    // Right click in append selector - cycle append mode
                    app.next_append_mode();
                    app.add_info(format!("追加: {}", app.tx_append_mode.name()));
                } else {
                    // Right click in input area - toggle TX mode
                    app.toggle_tx_mode();
                    app.add_info(format!(
                        "发送模式: {}",
                        match app.tx_mode {
                            TxMode::Hex => "HEX",
                            TxMode::Ascii => "ASCII",
                        }
                    ));
                }
            } else if is_inside(areas.control_area, col, row) {
                // Right click in control area - toggle auto scroll
                app.auto_scroll = !app.auto_scroll;
                let status = if app.auto_scroll { "启用" } else { "禁用" };
                app.add_info(format!("自动滚动: {}", status));
            }
        }

        // Middle click - clear log or input
        MouseEventKind::Down(MouseButton::Middle) => {
            let areas = get_ui_areas();

            if is_inside(areas.log_area, col, row) {
                // Middle click in log area - clear log
                app.message_log.clear();
                app.add_info("已清空消息记录");
            } else if is_inside(areas.tx_area, col, row) {
                // Middle click in TX area - clear input
                app.tx_input.clear();
                app.tx_cursor = 0;
                app.add_info("已清空输入");
            }
        }

        // Scroll up - navigate or scroll log
        MouseEventKind::ScrollUp => {
            let areas = get_ui_areas();

            if is_inside(areas.log_area, col, row) {
                // Scroll in log area
                app.auto_scroll = false;
                app.scroll_offset = app.scroll_offset.saturating_sub(3);
            } else if is_inside(areas.port, col, row) {
                // Scroll in port list
                if !app.can_modify_config() {
                    app.add_warning("配置已锁定，请先断开连接");
                } else if let Some(idx) = app.port_list_state.selected() {
                    let new_idx = if idx > 0 {
                        idx - 1
                    } else {
                        app.ports.len().saturating_sub(1)
                    };
                    app.select_port(new_idx);
                }
            } else if is_inside(areas.baud_rate, col, row) {
                // Scroll in baud rate list
                if !app.can_modify_config() {
                    app.add_warning("配置已锁定，请先断开连接");
                } else {
                    app.prev_baud_rate();
                }
            } else if is_inside(areas.tx_area, col, row) {
                // Scroll in TX area - cycle append mode
                app.prev_append_mode();
            }
        }

        // Scroll down - navigate or scroll log
        MouseEventKind::ScrollDown => {
            let areas = get_ui_areas();

            if is_inside(areas.log_area, col, row) {
                // Scroll in log area
                app.scroll_offset = app.scroll_offset.saturating_add(3);

                // Check if we scrolled to the end
                let lines = app.message_log.entries.len() as u16;
                let viewport_lines = areas.log_area.height.saturating_sub(2).max(1);
                let max_scroll = lines.saturating_sub(viewport_lines);
                if app.scroll_offset >= max_scroll {
                    app.auto_scroll = true;
                }
            } else if is_inside(areas.port, col, row) {
                // Scroll in port list
                if !app.can_modify_config() {
                    app.add_warning("配置已锁定，请先断开连接");
                } else if let Some(idx) = app.port_list_state.selected() {
                    let new_idx = if idx < app.ports.len().saturating_sub(1) {
                        idx + 1
                    } else {
                        0
                    };
                    app.select_port(new_idx);
                }
            } else if is_inside(areas.baud_rate, col, row) {
                // Scroll in baud rate list
                if !app.can_modify_config() {
                    app.add_warning("配置已锁定，请先断开连接");
                } else {
                    app.next_baud_rate();
                }
            } else if is_inside(areas.tx_area, col, row) {
                // Scroll in TX area - cycle append mode
                app.next_append_mode();
            }
        }

        // Drag events - for future implementation (e.g., selecting text)
        MouseEventKind::Drag(MouseButton::Left) => {
            // TODO: Implement text selection in log area
        }

        _ => {}
    }
}
