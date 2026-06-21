//! TuiSerial - Terminal User Interface for Serial Port Communication

use std::io;
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use tuiserial_core::AppState;
use tuiserial_serial::list_ports;
use tuiserial_ui::draw;

use rust_i18n::i18n;
// Initialize i18n translations at compile time
i18n!("../../locales", fallback = "en");

mod global_handler;
mod handler;
mod input_utils;
mod key_handler;
mod menu_handler;
mod mouse_handler;
mod plugin_adapter;
mod tx_handler;

use handler::SerialHandler;
use plugin_adapter::PluginProxy;

fn main() -> Result<()> {
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

fn run_app(mut terminal: Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    let mut app = AppState::default();
    let mut handler = SerialHandler::new();

    // Load saved configuration
    app.load_config();

    // Initialize locale from saved language preference
    rust_i18n::set_locale(app.language.code());

    // Initialize plugin manager (no-op when feature is disabled)
    let mut plugin_proxy = PluginProxy::init(&mut app);

    // Initialize available ports
    app.ports = list_ports();
    if !app.ports.is_empty() {
        if app.config.port.is_empty() {
            app.config.port = app.ports[0].clone();
            app.port_list_state.select(Some(0));
        } else if let Some(idx) = app.ports.iter().position(|p| p == &app.config.port) {
            app.port_list_state.select(Some(idx));
        } else if !app.ports.is_empty() {
            app.config.port = app.ports[0].clone();
            app.port_list_state.select(Some(0));
        }
    }

    loop {
        app.update_notifications();
        plugin_proxy.flush_plugin_logs(&mut app);
        terminal.draw(|f| draw(f, &app))?;

        // Apply native cursor state (set during rendering)
        {
            let areas = tuiserial_ui::get_ui_areas();
            if areas.show_cursor {
                execute!(io::stdout(), MoveTo(areas.cursor_x, areas.cursor_y), Show)?;
            } else {
                execute!(io::stdout(), Hide)?;
            }
        }

        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    let should_exit = key_handler::handle_key_event(
                        key,
                        &mut app,
                        &mut handler,
                        &mut plugin_proxy,
                    );
                    if should_exit {
                        break;
                    }
                }
                Event::Mouse(mouse) => {
                    mouse_handler::handle_mouse_event(
                        mouse,
                        &mut app,
                        &mut handler,
                        &mut plugin_proxy,
                    );
                }
                Event::Resize(_, _) => {}
                Event::Paste(data) => {
                    input_utils::handle_paste_event(&data, &mut app);
                }
                _ => {}
            }
        }

        // Try to read from serial port if connected
        if handler.is_connected() {
            match handler.read() {
                Ok(data) if !data.is_empty() => {
                    handler.reset_read_errors();
                    let (processed, suppressed) = plugin_proxy.process_rx(data, &app.config);
                    if !suppressed {
                        app.message_log.push_rx(processed);
                        if app.auto_scroll {
                            let lines_count = app.message_log.entries.len() as u16;
                            app.scroll_offset = lines_count.saturating_sub(1);
                        }
                    }
                }
                Ok(_) => {
                    // Timeout or empty read — normal, reset error counter.
                    handler.reset_read_errors();
                }
                Err(e) => {
                    let (app_error, should_disconnect) = handler.handle_read_error(e);
                    app.record_error(app_error);
                    if should_disconnect {
                        app.add_error("Auto-disconnecting due to repeated serial errors");
                        for err in plugin_proxy.on_disconnect() {
                            app.record_error(err);
                        }
                        handler.disconnect();
                        app.is_connected = false;
                    }
                }
            }
        }
    }

    if handler.is_connected() {
        for err in plugin_proxy.on_disconnect() {
            app.record_error(err);
        }
        handler.disconnect();
    }
    plugin_proxy.on_app_exit();

    Ok(())
}
