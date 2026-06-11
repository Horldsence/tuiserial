//! TuiSerial - Terminal User Interface for Serial Port Communication

use std::io;
use std::time::Duration;

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use tuiserial_core::AppState;
use tuiserial_ui::draw;
#[cfg(feature = "plugin")]
use tuiserial_plugin::PluginManager;
use tuiserial_serial::list_ports;

mod handler;
mod input_utils;
mod menu_handler;
mod tx_handler;
mod key_handler;
mod global_handler;
mod mouse_handler;

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

    // Load saved configuration
    app.load_config();

    // Initialize plugin manager
    #[cfg(feature = "plugin")]
    let mut plugin_manager = {
        let plugin_dir = dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(".config")
            .join("tuiserial")
            .join("plugins");
        let mut pm = PluginManager::new(plugin_dir);
        match pm.discover_and_load() {
            Ok(n) => {
                menu_handler::sync_plugin_status(&mut app, &pm);
                if n > 0 {
                    app.add_success(format!("{} plugin(s) loaded", n));
                }
            }
            Err(e) => {
                menu_handler::sync_plugin_status(&mut app, &pm);
                app.add_error(format!("Plugin load error: {}", e));
            }
        }
        pm
    };

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
        #[cfg(feature = "plugin")]
        plugin_manager.flush_plugin_logs(&mut app);
        terminal.draw(|f| draw(f, &app))?;

        // Apply native cursor state (set during rendering)
        {
            let areas = tuiserial_ui::get_ui_areas();
            if areas.show_cursor {
                execute!(
                    io::stdout(),
                    MoveTo(areas.cursor_x, areas.cursor_y),
                    Show
                )?;
            } else {
                execute!(io::stdout(), Hide)?;
            }
        }

        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    #[cfg(feature = "plugin")]
                    let should_exit =
                        key_handler::handle_key_event(key, &mut app, &mut handler, &mut plugin_manager);
                    #[cfg(not(feature = "plugin"))]
                    let should_exit =
                        key_handler::handle_key_event(key, &mut app, &mut handler);
                    if should_exit {
                        break;
                    }
                }
                Event::Mouse(mouse) => {
                    #[cfg(feature = "plugin")]
                    mouse_handler::handle_mouse_event(mouse, &mut app, &mut handler, &mut plugin_manager);
                    #[cfg(not(feature = "plugin"))]
                    mouse_handler::handle_mouse_event(mouse, &mut app, &mut handler);
                }
                Event::Resize(_, _) => {}
                Event::Paste(data) => {
                    input_utils::handle_paste_event(&data, &mut app);
                }
                _ => {}
            }
        }

        // Try to read from serial port if connected
        if handler.is_connected()
            && let Ok(data) = handler.read()
            && !data.is_empty() {
                    #[cfg(feature = "plugin")]
                    let (processed, suppressed) =
                        plugin_manager.process_rx(data, &app.config);
                    #[cfg(not(feature = "plugin"))]
                    let (processed, suppressed) = (data, false);
                    if !suppressed {
                        app.message_log.push_rx(processed);
                        if app.auto_scroll {
                            let lines_count = app.message_log.entries.len() as u16;
                            app.scroll_offset = lines_count.saturating_sub(1);
                        }
                    }
        }
    }

    if handler.is_connected() {
        #[cfg(feature = "plugin")]
        plugin_manager.on_disconnect();
        handler.disconnect();
    }
    #[cfg(feature = "plugin")]
    plugin_manager.on_app_exit();

    Ok(())
}
