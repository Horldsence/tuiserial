//! Basic example of using tuiserial-tabs
//!
//! This example demonstrates:
//! - Creating multiple sessions
//! - Switching between layouts
//! - Basic keyboard navigation
//!
//! Run with: cargo run --example basic

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};
use tuiserial_tabs::{
    calculate_tab_bar_height, draw_compact_tab_bar, draw_pane_border, TabsManager,
};

struct DemoApp {
    tabs_manager: TabsManager,
    should_quit: bool,
    last_key: String,
    tick_count: u64,
    start_time: Instant,
}

impl DemoApp {
    fn new() -> Self {
        let mut tabs_manager = TabsManager::new();

        // Add some demo sessions
        tabs_manager.add_session_with_port("COM1".to_string(), Some("Arduino Uno".to_string()));
        tabs_manager.add_session_with_port("COM3".to_string(), Some("ESP32".to_string()));
        tabs_manager.add_session_with_port("COM5".to_string(), Some("Sensor Board".to_string()));

        // Simulate some session states
        if let Some(session) = tabs_manager.sessions_mut().get_session_mut(0) {
            session.is_connected = true;
            session
                .message_log
                .push_rx(b"Hello from Arduino!\r\n".to_vec());
            session.message_log.push_tx(b"AT+CMD\r\n".to_vec());
        }

        if let Some(session) = tabs_manager.sessions_mut().get_session_mut(1) {
            session.is_connected = true;
        }

        Self {
            tabs_manager,
            should_quit: false,
            last_key: String::from("None"),
            tick_count: 0,
            start_time: Instant::now(),
        }
    }

    fn handle_key(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        self.last_key = format!("{:?} {:?}", key, modifiers);

        match (key, modifiers) {
            // Quit
            (KeyCode::Char('q'), m) if m.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }

            // Session management
            (KeyCode::Char('t'), m) if m.contains(KeyModifiers::CONTROL) => {
                let count = self.tabs_manager.sessions().len() + 1;
                self.tabs_manager
                    .add_session(Some(format!("Session {}", count)));
            }
            (KeyCode::Char('w'), m) if m.contains(KeyModifiers::CONTROL) => {
                let active_idx = self.tabs_manager.sessions().active_index();
                self.tabs_manager.remove_session(active_idx);
            }

            // Tab navigation
            (KeyCode::Tab, m) if m.contains(KeyModifiers::CONTROL) => {
                if m.contains(KeyModifiers::SHIFT) {
                    self.tabs_manager.sessions_mut().prev_session();
                } else {
                    self.tabs_manager.sessions_mut().next_session();
                }
            }
            (KeyCode::Left, m) if m.contains(KeyModifiers::CONTROL) => {
                self.tabs_manager.sessions_mut().prev_session();
            }
            (KeyCode::Right, m) if m.contains(KeyModifiers::CONTROL) => {
                self.tabs_manager.sessions_mut().next_session();
            }

            // Quick session switch (1-9)
            (KeyCode::Char(c), m) if m.contains(KeyModifiers::CONTROL) && c.is_numeric() => {
                if let Some(idx) = c.to_digit(10) {
                    let idx = (idx as usize).saturating_sub(1);
                    self.tabs_manager.sessions_mut().switch_to(idx);
                }
            }

            // Layout management
            (KeyCode::Char('l'), m) if m.contains(KeyModifiers::CONTROL) => {
                if m.contains(KeyModifiers::SHIFT) {
                    self.tabs_manager.prev_layout();
                } else {
                    self.tabs_manager.next_layout();
                }
            }

            // Pane navigation
            (KeyCode::Char('p'), m) if m.contains(KeyModifiers::CONTROL) => {
                if m.contains(KeyModifiers::SHIFT) {
                    self.tabs_manager.focus_prev_pane();
                } else {
                    self.tabs_manager.focus_next_pane();
                }
            }

            // Cycle session in focused pane
            (KeyCode::Char('n'), m) if m.contains(KeyModifiers::CONTROL) => {
                self.tabs_manager.cycle_focused_pane_session();
            }

            // Toggle connection (demo)
            (KeyCode::Char('c'), _) => {
                if let Some(session) = self.tabs_manager.focused_pane_session_mut() {
                    session.is_connected = !session.is_connected;
                    if session.is_connected {
                        session.add_success("Connected to port");
                    } else {
                        session.add_info("Disconnected from port");
                    }
                }
            }

            // Add demo message
            (KeyCode::Char('m'), _) => {
                if let Some(session) = self.tabs_manager.focused_pane_session_mut() {
                    session
                        .message_log
                        .push_rx(b"Demo message received\r\n".to_vec());
                }
            }

            _ => {}
        }
    }

    fn tick(&mut self) {
        self.tick_count += 1;
        self.tabs_manager.update_notifications();
    }
}

fn ui(f: &mut Frame, app: &DemoApp) {
    let tabs_manager = &app.tabs_manager;

    // Calculate tab bar height
    let tab_height = calculate_tab_bar_height(tabs_manager.sessions().len(), false);

    // Main layout
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),          // Title bar
            Constraint::Length(tab_height), // Tab bar
            Constraint::Min(10),            // Content
            Constraint::Length(5),          // Status bar
        ])
        .split(f.area());

    // Title bar
    draw_title_bar(f, main_chunks[0], app);

    // Tab bar
    if tab_height > 0 {
        draw_compact_tab_bar(f, main_chunks[1], tabs_manager.sessions());
    }

    // Content area with panes
    draw_panes(f, app, main_chunks[2]);

    // Status bar
    draw_status_bar(f, main_chunks[3], app);
}

fn draw_title_bar(f: &mut Frame, area: Rect, _app: &DemoApp) {
    let title = " tuiserial-tabs Demo - Press Ctrl+Q to quit ";
    let title_widget = Paragraph::new(title)
        .style(
            Style::default()
                .fg(Color::White)
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    f.render_widget(title_widget, area);
}

fn draw_panes(f: &mut Frame, app: &DemoApp, area: Rect) {
    let tabs_manager = &app.tabs_manager;
    let pane_areas = tabs_manager.panes().calculate_areas(area);

    for (pane_idx, pane_area) in pane_areas.iter().enumerate() {
        if let Some(session) = tabs_manager.session_for_pane(pane_idx) {
            let is_focused = tabs_manager.is_pane_focused(pane_idx);

            // Draw pane border
            let inner = draw_pane_border(
                f,
                *pane_area,
                &session.name,
                is_focused,
                session.is_connected,
            );

            // Draw session content
            draw_session_content(f, session, inner, is_focused);
        }
    }
}

fn draw_session_content(
    f: &mut Frame,
    session: &tuiserial_tabs::SerialSession,
    area: Rect,
    is_focused: bool,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8), // Config info
            Constraint::Min(5),    // Log area
            Constraint::Length(3), // TX area
        ])
        .split(area);

    // Config info
    draw_config_info(f, session, chunks[0]);

    // Log area
    draw_log_area(f, session, chunks[1], is_focused);

    // TX area
    draw_tx_info(f, session, chunks[2]);
}

fn draw_config_info(f: &mut Frame, session: &tuiserial_tabs::SerialSession, area: Rect) {
    let config = &session.config;
    let info = vec![
        Line::from(vec![
            Span::styled("Port: ", Style::default().fg(Color::Cyan)),
            Span::raw(&config.port),
        ]),
        Line::from(vec![
            Span::styled("Baud: ", Style::default().fg(Color::Cyan)),
            Span::raw(format!("{}", config.baud_rate)),
        ]),
        Line::from(vec![
            Span::styled("Data: ", Style::default().fg(Color::Cyan)),
            Span::raw(format!(
                "{}-{:?}-{}",
                config.data_bits, config.parity, config.stop_bits
            )),
        ]),
        Line::from(vec![
            Span::styled("Status: ", Style::default().fg(Color::Cyan)),
            if session.is_connected {
                Span::styled("Connected", Style::default().fg(Color::Green))
            } else {
                Span::styled("Disconnected", Style::default().fg(Color::Red))
            },
        ]),
    ];

    let paragraph =
        Paragraph::new(info).block(Block::default().borders(Borders::ALL).title(" Config "));
    f.render_widget(paragraph, area);
}

fn draw_log_area(
    f: &mut Frame,
    session: &tuiserial_tabs::SerialSession,
    area: Rect,
    is_focused: bool,
) {
    let entries = &session.message_log.entries;
    let lines: Vec<Line> = entries
        .iter()
        .rev()
        .take(area.height.saturating_sub(2) as usize)
        .rev()
        .map(|entry| {
            let direction_color = match entry.direction {
                tuiserial_core::log::LogDirection::Rx => Color::Green,
                tuiserial_core::log::LogDirection::Tx => Color::Yellow,
            };

            let direction_symbol = match entry.direction {
                tuiserial_core::log::LogDirection::Rx => "← ",
                tuiserial_core::log::LogDirection::Tx => "→ ",
            };

            let data_str = String::from_utf8_lossy(&entry.data).to_string();
            Line::from(vec![
                Span::styled(direction_symbol, Style::default().fg(direction_color)),
                Span::raw(data_str.trim_end().to_string()),
            ])
        })
        .collect();

    let title = if is_focused {
        " Log (Focused) "
    } else {
        " Log "
    };

    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(title),
    );
    f.render_widget(paragraph, area);
}

fn draw_tx_info(f: &mut Frame, session: &tuiserial_tabs::SerialSession, area: Rect) {
    let info = format!(
        "TX: {} | Mode: {:?} | Append: {}",
        session.tx_input,
        session.tx_mode,
        session.tx_append_mode.name()
    );

    let paragraph =
        Paragraph::new(info).block(Block::default().borders(Borders::ALL).title(" TX "));
    f.render_widget(paragraph, area);
}

fn draw_status_bar(f: &mut Frame, area: Rect, app: &DemoApp) {
    let elapsed = app.start_time.elapsed();
    let layout_mode = app.tabs_manager.layout_mode();
    let layout_name = layout_mode.name();

    let help_text = vec![
        Line::from("Keyboard Shortcuts:"),
        Line::from(vec![
            Span::styled("Ctrl+T", Style::default().fg(Color::Yellow)),
            Span::raw(": New  "),
            Span::styled("Ctrl+W", Style::default().fg(Color::Yellow)),
            Span::raw(": Close  "),
            Span::styled("Ctrl+Tab", Style::default().fg(Color::Yellow)),
            Span::raw(": Switch  "),
            Span::styled("Ctrl+L", Style::default().fg(Color::Yellow)),
            Span::raw(": Layout  "),
            Span::styled("C", Style::default().fg(Color::Yellow)),
            Span::raw(": Connect  "),
            Span::styled("M", Style::default().fg(Color::Yellow)),
            Span::raw(": Add Msg"),
        ]),
        Line::from(vec![
            Span::styled("Layout: ", Style::default().fg(Color::Cyan)),
            Span::raw(layout_name),
            Span::raw(format!(
                "  |  Sessions: {}  |  Uptime: {:?}  |  Last Key: {}",
                app.tabs_manager.sessions().len(),
                elapsed,
                app.last_key
            )),
        ]),
    ];

    let paragraph =
        Paragraph::new(help_text).block(Block::default().borders(Borders::ALL).title(" Status "));
    f.render_widget(paragraph, area);
}

fn main() -> Result<(), Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = DemoApp::new();

    // Main loop
    let tick_rate = Duration::from_millis(100);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui(f, &app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                app.handle_key(key.code, key.modifiers);
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.tick();
            last_tick = Instant::now();
        }

        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
