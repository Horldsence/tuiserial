use std::io;
use std::time::Duration;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind, MouseEvent, MouseEventKind, MouseButton},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

mod model;
mod serial;
mod ui;
mod handler;

use model::{AppState, FocusedField, DisplayMode, LogDirection, TxMode};
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
    execute!(
        io::stdout(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    
    result
}

fn run_app(mut terminal: Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    let mut app = AppState::default();
    let mut handler = SerialHandler::new();
    
    app.ports = serial::list_ports();
    if !app.ports.is_empty() {
        app.config.port = app.ports[0].clone();
        app.port_list_state.select(Some(0));
    }

    loop {
        app.update_notifications();
        terminal.draw(|f| ui::draw(f, &app))?;

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
                app.add_info(format!("发送模式: {}", match app.tx_mode { TxMode::Hex => "HEX", TxMode::Ascii => "ASCII" }));
                return false;
            }
            KeyCode::Down => {
                app.toggle_tx_mode();
                app.add_info(format!("发送模式: {}", match app.tx_mode { TxMode::Hex => "HEX", TxMode::Ascii => "ASCII" }));
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
                        let bytes: Result<Vec<u8>, String> = match app.tx_mode {
                            TxMode::Ascii => Ok(app.tx_input.as_bytes().to_vec()),
                            TxMode::Hex => crate::serial::hex_to_bytes(&app.tx_input),
                        };

                        match bytes {
                            Ok(data) => {
                                match handler.send(&data) {
                                    Ok(_sent) => {
                                        app.message_log.push_tx(data.clone());
                                        app.add_success("已发送");
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
                                }
                            }
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
                app.add_info("已断开连接");
            } else {
                match handler.connect(&app) {
                    Ok(_) => {
                        app.is_connected = true;
                        app.add_success(format!("已连接: {}", app.config.port));
                    }
                    Err(e) => {
                        app.is_connected = false;
                        app.add_error(format!("连接失败: {}", e));
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
        KeyCode::Up | KeyCode::Char('k') => {
            match app.focused_field {
                FocusedField::Port => {
                    if let Some(idx) = app.port_list_state.selected() {
                        let new_idx = if idx > 0 { idx - 1 } else { app.ports.len().saturating_sub(1) };
                        app.port_list_state.select(Some(new_idx));
                        if !app.ports.is_empty() {
                            app.config.port = app.ports[new_idx].clone();
                        }
                    }
                }
                FocusedField::BaudRate => {
                    if let Some(idx) = app.baud_rate_state.selected() {
                        let new_idx = if idx > 0 { idx - 1 } else { app.baud_rate_options.len() - 1 };
                        app.baud_rate_state.select(Some(new_idx));
                        app.config.baud_rate = app.baud_rate_options[new_idx];
                    }
                }
                FocusedField::DataBits => {
                    if let Some(idx) = app.data_bits_state.selected() {
                        let new_idx = if idx > 0 { idx - 1 } else { app.data_bits_options.len() - 1 };
                        app.data_bits_state.select(Some(new_idx));
                        app.config.data_bits = app.data_bits_options[new_idx];
                    }
                }
                FocusedField::Parity => {
                    if let Some(idx) = app.parity_state.selected() {
                        let new_idx = if idx > 0 { idx - 1 } else { app.parity_options.len() - 1 };
                        app.parity_state.select(Some(new_idx));
                        app.config.parity = app.parity_options[new_idx];
                    }
                }
                FocusedField::StopBits => {
                    if let Some(idx) = app.stop_bits_state.selected() {
                        let new_idx = if idx > 0 { idx - 1 } else { app.stop_bits_options.len() - 1 };
                        app.stop_bits_state.select(Some(new_idx));
                        app.config.stop_bits = app.stop_bits_options[new_idx];
                    }
                }
                FocusedField::LogArea => {
                    app.toggle_display_mode();
                    let mode_str = match app.display_mode { DisplayMode::Hex => "HEX", DisplayMode::Text => "TEXT" };
                    app.add_info(format!("显示模式: {}", mode_str));
                }
                _ => {}
            }
        }

        KeyCode::Down | KeyCode::Char('j') => {
            match app.focused_field {
                FocusedField::Port => {
                    if let Some(idx) = app.port_list_state.selected() {
                        let new_idx = if idx < app.ports.len().saturating_sub(1) { idx + 1 } else { 0 };
                        app.port_list_state.select(Some(new_idx));
                        if !app.ports.is_empty() {
                            app.config.port = app.ports[new_idx].clone();
                        }
                    }
                }
                FocusedField::BaudRate => {
                    if let Some(idx) = app.baud_rate_state.selected() {
                        let new_idx = if idx < app.baud_rate_options.len() - 1 { idx + 1 } else { 0 };
                        app.baud_rate_state.select(Some(new_idx));
                        app.config.baud_rate = app.baud_rate_options[new_idx];
                    }
                }
                FocusedField::DataBits => {
                    if let Some(idx) = app.data_bits_state.selected() {
                        let new_idx = if idx < app.data_bits_options.len() - 1 { idx + 1 } else { 0 };
                        app.data_bits_state.select(Some(new_idx));
                        app.config.data_bits = app.data_bits_options[new_idx];
                    }
                }
                FocusedField::Parity => {
                    if let Some(idx) = app.parity_state.selected() {
                        let new_idx = if idx < app.parity_options.len() - 1 { idx + 1 } else { 0 };
                        app.parity_state.select(Some(new_idx));
                        app.config.parity = app.parity_options[new_idx];
                    }
                }
                FocusedField::StopBits => {
                    if let Some(idx) = app.stop_bits_state.selected() {
                        let new_idx = if idx < app.stop_bits_options.len() - 1 { idx + 1 } else { 0 };
                        app.stop_bits_state.select(Some(new_idx));
                        app.config.stop_bits = app.stop_bits_options[new_idx];
                    }
                }
                FocusedField::LogArea => {
                    app.toggle_display_mode();
                    let mode_str = match app.display_mode { DisplayMode::Hex => "HEX", DisplayMode::Text => "TEXT" };
                    app.add_info(format!("显示模式: {}", mode_str));
                }
                _ => {}
            }
        }

        // Left/Right for BaudRate and other controls
        KeyCode::Right | KeyCode::Char('l') => {
            match app.focused_field {
                FocusedField::BaudRate => {
                    if let Some(idx) = app.baud_rate_state.selected() {
                        let new_idx = if idx < app.baud_rate_options.len() - 1 { idx + 1 } else { idx };
                        app.baud_rate_state.select(Some(new_idx));
                        app.config.baud_rate = app.baud_rate_options[new_idx];
                    }
                }
                _ => {}
            }
        }

        KeyCode::Left | KeyCode::Char('h') => {
            match app.focused_field {
                FocusedField::BaudRate => {
                    if let Some(idx) = app.baud_rate_state.selected() {
                        let new_idx = if idx > 0 { idx - 1 } else { 0 };
                        app.baud_rate_state.select(Some(new_idx));
                        app.config.baud_rate = app.baud_rate_options[new_idx];
                    }
                }
                _ => {}
            }
        }

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
            app.toggle_parity();
            let parity_str = format!("{:?}", app.config.parity);
            app.add_info(format!("校验位: {}", parity_str));
        }

        // Flow control toggle
        KeyCode::Char('f') => {
            app.toggle_flow_control();
            let flow_str = format!("{:?}", app.config.flow_control);
            app.add_info(format!("流控: {}", flow_str));
        }

        // Scroll navigation
        KeyCode::PageUp => {
            app.scroll_offset = app.scroll_offset.saturating_sub(10);
        }
        KeyCode::PageDown => {
            app.scroll_offset = app.scroll_offset.saturating_add(10);
        }

        KeyCode::Home => {
            app.scroll_offset = 0;
        }
        KeyCode::End => {
            let lines = app.message_log.entries.len() as u16;
            app.scroll_offset = lines.saturating_sub(1);
        }

        _ => {}
    }

    false
}

fn handle_mouse_event(mouse: MouseEvent, app: &mut AppState) {
    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            // Click detection for UI elements
            // Calculate screen layout (simplified)
            let col = mouse.column;
            let row = mouse.row;
            
            // Left panel is 40 columns wide (serial config)
            if col < 40 {
                // Click in config panel - switch to appropriate field
                if row >= 3 && row <= 5 { app.focused_field = FocusedField::Port; }
                else if row >= 6 && row <= 9 { app.focused_field = FocusedField::BaudRate; }
                else if row >= 10 && row <= 12 { app.focused_field = FocusedField::DataBits; }
                else if row >= 13 && row <= 15 { app.focused_field = FocusedField::Parity; }
                else if row >= 16 && row <= 18 { app.focused_field = FocusedField::StopBits; }
                else if row >= 19 && row <= 21 { app.focused_field = FocusedField::FlowControl; }
            } else {
                // Right panel - click in log or tx area
                // Log area is roughly from row 1-12
                // TX area is roughly from row 13-18
                // Status area is roughly from row 19+
                
                if row >= 1 && row <= 12 {
                    app.focused_field = FocusedField::LogArea;
                } else if row >= 13 && row <= 18 {
                    app.focused_field = FocusedField::TxInput;
                }
            }
        }
        MouseEventKind::ScrollUp => {
            match app.focused_field {
                FocusedField::LogArea => {
                    app.scroll_offset = app.scroll_offset.saturating_sub(3);
                }
                _ => {}
            }
        }
        MouseEventKind::ScrollDown => {
            match app.focused_field {
                FocusedField::LogArea => {
                    app.scroll_offset = app.scroll_offset.saturating_add(3);
                }
                _ => {}
            }
        }
        _ => {}
    }
}
