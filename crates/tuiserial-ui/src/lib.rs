//! Terminal user interface components for tuiserial
//!
//! This crate provides the UI rendering logic using ratatui for displaying
//! serial port configuration, logs, and user interactions with full mouse support.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};
use tuiserial_core::{
    AppState, DisplayMode, FlowControl, FocusedField, LogDirection, NotificationLevel, Parity,
    TxMode,
};
use tuiserial_serial::{bytes_to_hex, bytes_to_string};

// Re-exports
pub use crossterm;
pub use ratatui;

/// UI area rectangles for mouse interaction
#[derive(Debug, Clone, Copy)]
pub struct UiAreas {
    pub port: Rect,
    pub baud_rate: Rect,
    pub data_bits: Rect,
    pub parity: Rect,
    pub stop_bits: Rect,
    pub flow_control: Rect,
    pub status_panel: Rect,
    pub log_area: Rect,
    pub tx_area: Rect,
    pub control_area: Rect,
    pub notification_area: Rect,
}

impl Default for UiAreas {
    fn default() -> Self {
        Self {
            port: Rect::default(),
            baud_rate: Rect::default(),
            data_bits: Rect::default(),
            parity: Rect::default(),
            stop_bits: Rect::default(),
            flow_control: Rect::default(),
            status_panel: Rect::default(),
            log_area: Rect::default(),
            tx_area: Rect::default(),
            control_area: Rect::default(),
            notification_area: Rect::default(),
        }
    }
}

// Global static for UI areas (thread-local would be better in production)
static mut UI_AREAS: UiAreas = UiAreas {
    port: Rect {
        x: 0,
        y: 0,
        width: 0,
        height: 0,
    },
    baud_rate: Rect {
        x: 0,
        y: 0,
        width: 0,
        height: 0,
    },
    data_bits: Rect {
        x: 0,
        y: 0,
        width: 0,
        height: 0,
    },
    parity: Rect {
        x: 0,
        y: 0,
        width: 0,
        height: 0,
    },
    stop_bits: Rect {
        x: 0,
        y: 0,
        width: 0,
        height: 0,
    },
    flow_control: Rect {
        x: 0,
        y: 0,
        width: 0,
        height: 0,
    },
    status_panel: Rect {
        x: 0,
        y: 0,
        width: 0,
        height: 0,
    },
    log_area: Rect {
        x: 0,
        y: 0,
        width: 0,
        height: 0,
    },
    tx_area: Rect {
        x: 0,
        y: 0,
        width: 0,
        height: 0,
    },
    control_area: Rect {
        x: 0,
        y: 0,
        width: 0,
        height: 0,
    },
    notification_area: Rect {
        x: 0,
        y: 0,
        width: 0,
        height: 0,
    },
};

/// Get the UI areas for mouse interaction
pub fn get_ui_areas() -> UiAreas {
    unsafe { UI_AREAS }
}

/// Check if a point is inside a rectangle
pub fn is_inside(rect: Rect, x: u16, y: u16) -> bool {
    x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height
}

/// Determine which field was clicked based on coordinates
pub fn get_clicked_field(x: u16, y: u16) -> Option<FocusedField> {
    let areas = get_ui_areas();

    if is_inside(areas.port, x, y) {
        Some(FocusedField::Port)
    } else if is_inside(areas.baud_rate, x, y) {
        Some(FocusedField::BaudRate)
    } else if is_inside(areas.data_bits, x, y) {
        Some(FocusedField::DataBits)
    } else if is_inside(areas.parity, x, y) {
        Some(FocusedField::Parity)
    } else if is_inside(areas.stop_bits, x, y) {
        Some(FocusedField::StopBits)
    } else if is_inside(areas.flow_control, x, y) {
        Some(FocusedField::FlowControl)
    } else if is_inside(areas.log_area, x, y) {
        Some(FocusedField::LogArea)
    } else if is_inside(areas.tx_area, x, y) {
        Some(FocusedField::TxInput)
    } else {
        None
    }
}

/// Main draw function - renders the entire application UI
pub fn draw(f: &mut Frame, app: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(15),   // Main content
            Constraint::Length(3), // Notification area
        ])
        .split(f.area());

    draw_main_content(f, app, chunks[0]);
    draw_notification_bar(f, app, chunks[1]);

    // Store notification area
    unsafe {
        UI_AREAS.notification_area = chunks[1];
    }
}

/// Draw the main content area (config panel + log/tx areas)
fn draw_main_content(f: &mut Frame, app: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(42), Constraint::Min(50)])
        .split(area);

    draw_config_panel(f, app, chunks[0]);
    draw_main_area(f, app, chunks[1]);
}

/// Draw the configuration panel on the left
fn draw_config_panel(f: &mut Frame, app: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Port
            Constraint::Length(5), // Baud
            Constraint::Length(3), // Data bits
            Constraint::Length(3), // Parity
            Constraint::Length(3), // Stop bits
            Constraint::Length(3), // Flow control
            Constraint::Min(10),   // Status (需要更多空间显示详细信息)
        ])
        .split(area);

    draw_port_dropdown(f, app, chunks[0]);
    draw_baud_rate_dropdown(f, app, chunks[1]);
    draw_data_bits_dropdown(f, app, chunks[2]);
    draw_parity_dropdown(f, app, chunks[3]);
    draw_stop_bits_dropdown(f, app, chunks[4]);
    draw_flow_control_dropdown(f, app, chunks[5]);
    draw_status_panel(f, app, chunks[6]);
}

/// Draw the main area on the right (log + tx + control)
fn draw_main_area(f: &mut Frame, app: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),   // Log area
            Constraint::Length(7), // TX area
            Constraint::Length(3), // Status bar
        ])
        .split(area);

    draw_log_area(f, app, chunks[0]);
    draw_tx_area(f, app, chunks[1]);
    draw_control_area(f, app, chunks[2]);
}

/// Draw the serial port selection dropdown
fn draw_port_dropdown(f: &mut Frame, app: &AppState, area: Rect) {
    // Store area for mouse interaction
    unsafe {
        UI_AREAS.port = area;
    }

    let focused = app.focused_field == FocusedField::Port;
    let is_locked = app.config_locked;

    let title = if is_locked {
        " 串口 [已锁定] "
    } else if focused {
        " 串口 [↑↓ 选择 | r 刷新] "
    } else {
        " 串口 "
    };

    let style = if is_locked {
        Style::default().fg(Color::DarkGray)
    } else if focused {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    if app.ports.is_empty() {
        let para = Paragraph::new("无可用串口\n按 r 刷新")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(style),
            )
            .style(Style::default().fg(Color::Red));
        f.render_widget(para, area);
        return;
    }

    let items: Vec<ListItem> = app
        .ports
        .iter()
        .map(|p| ListItem::new(p.as_str()))
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(style),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("► ");

    f.render_stateful_widget(list, area, &mut app.port_list_state.clone());
}

/// Draw the baud rate selection dropdown
fn draw_baud_rate_dropdown(f: &mut Frame, app: &AppState, area: Rect) {
    // Store area for mouse interaction
    unsafe {
        UI_AREAS.baud_rate = area;
    }

    let focused = app.focused_field == FocusedField::BaudRate;
    let is_locked = app.config_locked;

    let title = if is_locked {
        " 波特率 [已锁定] "
    } else if focused {
        " 波特率 [↑↓ 或 ←→ 变更] "
    } else {
        " 波特率 "
    };

    let style = if is_locked {
        Style::default().fg(Color::DarkGray)
    } else if focused {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let items: Vec<ListItem> = app
        .baud_rate_options
        .iter()
        .map(|b| ListItem::new(b.to_string()))
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(style),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("► ");

    f.render_stateful_widget(list, area, &mut app.baud_rate_state.clone());
}

/// Draw the data bits selection dropdown
fn draw_data_bits_dropdown(f: &mut Frame, app: &AppState, area: Rect) {
    // Store area for mouse interaction
    unsafe {
        UI_AREAS.data_bits = area;
    }

    let focused = app.focused_field == FocusedField::DataBits;
    let is_locked = app.config_locked;

    let title = if is_locked {
        " 数据位 [已锁定] "
    } else if focused {
        " 数据位 [↑↓ 选择] "
    } else {
        " 数据位 "
    };

    let style = if is_locked {
        Style::default().fg(Color::DarkGray)
    } else if focused {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let items: Vec<ListItem> = app
        .data_bits_options
        .iter()
        .map(|b| ListItem::new(format!("{} bits", b)))
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(style),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("► ");

    f.render_stateful_widget(list, area, &mut app.data_bits_state.clone());
}

/// Draw the parity selection dropdown
fn draw_parity_dropdown(f: &mut Frame, app: &AppState, area: Rect) {
    // Store area for mouse interaction
    unsafe {
        UI_AREAS.parity = area;
    }

    let focused = app.focused_field == FocusedField::Parity;
    let is_locked = app.config_locked;

    let title = if is_locked {
        " 校验位 [已锁定] "
    } else if focused {
        " 校验位 [↑↓ 或 p 切换] "
    } else {
        " 校验位 "
    };

    let style = if is_locked {
        Style::default().fg(Color::DarkGray)
    } else if focused {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let items: Vec<ListItem> = app
        .parity_options
        .iter()
        .map(|p| {
            let text = match p {
                Parity::None => "None",
                Parity::Even => "Even",
                Parity::Odd => "Odd",
            };
            ListItem::new(text)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(style),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("► ");

    f.render_stateful_widget(list, area, &mut app.parity_state.clone());
}

/// Draw the stop bits selection dropdown
fn draw_stop_bits_dropdown(f: &mut Frame, app: &AppState, area: Rect) {
    // Store area for mouse interaction
    unsafe {
        UI_AREAS.stop_bits = area;
    }

    let focused = app.focused_field == FocusedField::StopBits;
    let is_locked = app.config_locked;

    let title = if is_locked {
        " 停止位 [已锁定] "
    } else if focused {
        " 停止位 [↑↓ 选择] "
    } else {
        " 停止位 "
    };

    let style = if is_locked {
        Style::default().fg(Color::DarkGray)
    } else if focused {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let items: Vec<ListItem> = app
        .stop_bits_options
        .iter()
        .map(|b| ListItem::new(format!("{} bit", b)))
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(style),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("► ");

    f.render_stateful_widget(list, area, &mut app.stop_bits_state.clone());
}

/// Draw the flow control selection dropdown
fn draw_flow_control_dropdown(f: &mut Frame, app: &AppState, area: Rect) {
    // Store area for mouse interaction
    unsafe {
        UI_AREAS.flow_control = area;
    }

    let focused = app.focused_field == FocusedField::FlowControl;
    let is_locked = app.config_locked;

    let title = if is_locked {
        " 流控制 [已锁定] "
    } else if focused {
        " 流控制 [↑↓ 或 f 切换] "
    } else {
        " 流控制 "
    };

    let style = if is_locked {
        Style::default().fg(Color::DarkGray)
    } else if focused {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let items: Vec<ListItem> = app
        .flow_control_options
        .iter()
        .map(|fc| {
            let text = match fc {
                FlowControl::None => "None",
                FlowControl::Hardware => "Hardware",
                FlowControl::Software => "Software",
            };
            ListItem::new(text)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(style),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("► ");

    f.render_stateful_widget(list, area, &mut app.flow_control_state.clone());
}

/// Draw the connection status panel
fn draw_status_panel(f: &mut Frame, app: &AppState, area: Rect) {
    // Store area for mouse interaction
    unsafe {
        UI_AREAS.status_panel = area;
    }

    let status_color = if app.is_connected {
        Color::Green
    } else {
        Color::Red
    };

    let status_icon = if app.is_connected { "✓" } else { "✗" };
    let status_text = if app.is_connected {
        "已连接"
    } else {
        "未连接"
    };

    let config_status = if app.config_locked {
        ("🔒", "已锁定", Color::Yellow)
    } else {
        ("🔓", "可修改", Color::Green)
    };

    // Format parity display
    let parity_str = match app.config.parity {
        tuiserial_core::Parity::None => "N",
        tuiserial_core::Parity::Even => "E",
        tuiserial_core::Parity::Odd => "O",
    };

    let text = vec![
        Line::from(vec![
            Span::styled(
                status_icon,
                Style::default()
                    .fg(status_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(
                status_text,
                Style::default()
                    .fg(status_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(config_status.0, Style::default().fg(config_status.2)),
            Span::raw(" "),
            Span::styled(config_status.1, Style::default().fg(config_status.2)),
        ]),
        Line::raw(""),
        Line::from(vec![
            Span::styled("串口: ", Style::default().fg(Color::Cyan)),
            Span::raw(if app.config.port.is_empty() {
                "未选择"
            } else {
                &app.config.port
            }),
        ]),
        Line::from(vec![
            Span::styled("波特: ", Style::default().fg(Color::Cyan)),
            Span::raw(format!("{}", app.config.baud_rate)),
        ]),
        Line::from(vec![
            Span::styled("配置: ", Style::default().fg(Color::Cyan)),
            Span::raw(format!(
                "{}-{}-{}",
                app.config.data_bits, parity_str, app.config.stop_bits
            )),
        ]),
        Line::raw(""),
        Line::from(vec![
            Span::styled(
                "o",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" 连接  "),
            Span::styled(
                "r",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" 刷新"),
        ]),
        Line::from(vec![
            Span::styled(
                "q",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" 退出  "),
            Span::styled(
                "Tab",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" 切换"),
        ]),
    ];

    let para = Paragraph::new(text).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" 状态信息 ")
            .title_alignment(Alignment::Left),
    );

    f.render_widget(para, area);
}

/// Draw the log area showing received and transmitted data
fn draw_log_area(f: &mut Frame, app: &AppState, area: Rect) {
    // Store area for mouse interaction
    unsafe {
        UI_AREAS.log_area = area;
    }

    let focused = app.focused_field == FocusedField::LogArea;

    if app.message_log.entries.is_empty() {
        let status_msg = if app.is_connected {
            "等待接收数据..."
        } else {
            "未连接 - 请按 o 打开串口连接"
        };

        let help_text = vec![
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled(
                status_msg,
                Style::default()
                    .fg(if app.is_connected {
                        Color::Cyan
                    } else {
                        Color::Yellow
                    })
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled(
                "快捷键提示",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("  x - 切换 HEX/TEXT 显示模式"),
            Line::from("  c - 清空消息记录"),
            Line::from("  a - 自动滚动开关"),
            Line::from("  ↑↓ PgUp/PgDn - 滚动浏览"),
        ];

        let para = Paragraph::new(help_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(if focused {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    })
                    .title(format!(
                        " 消息 - {} ",
                        match app.display_mode {
                            DisplayMode::Hex => "HEX",
                            DisplayMode::Text => "TEXT",
                        }
                    ))
                    .title_alignment(Alignment::Left),
            )
            .alignment(Alignment::Center);
        f.render_widget(para, area);
        return;
    }

    let mut lines: Vec<Line> = Vec::new();
    for entry in app.message_log.entries.iter() {
        let (time_color, dir_str, dir_icon) = match entry.direction {
            LogDirection::Rx => (Color::Cyan, "RX", "◄"),
            LogDirection::Tx => (Color::Green, "TX", "►"),
        };

        let time_str = entry.timestamp.format("%H:%M:%S%.3f").to_string();
        let data_len = entry.data.len();

        let data_str = match app.display_mode {
            DisplayMode::Hex => bytes_to_hex(&entry.data),
            DisplayMode::Text => bytes_to_string(&entry.data),
        };

        let mut spans: Vec<Span> = Vec::new();
        // Timestamp
        spans.push(Span::styled(
            format!("[{}] ", time_str),
            Style::default().fg(Color::DarkGray),
        ));
        // Direction icon and label
        spans.push(Span::styled(
            format!("{} {} ", dir_icon, dir_str),
            Style::default().fg(time_color).add_modifier(Modifier::BOLD),
        ));
        // Data length
        spans.push(Span::styled(
            format!("({:>4} B) ", data_len),
            Style::default().fg(Color::Yellow),
        ));
        // Actual data
        spans.push(Span::styled(data_str, Style::default().fg(Color::White)));
        lines.push(Line::from(spans));
    }

    let display_mode_str = match app.display_mode {
        DisplayMode::Hex => "HEX",
        DisplayMode::Text => "TEXT",
    };

    let title = format!(
        " 消息 - {} | {} 条 [x 切换 | c 清空] ",
        display_mode_str,
        app.message_log.entries.len()
    );

    let total_lines = lines.len() as u16;
    let viewport_lines = area.height.saturating_sub(2).max(1);
    let max_scroll = total_lines.saturating_sub(viewport_lines);
    let scroll_top = if app.auto_scroll {
        max_scroll
    } else {
        app.scroll_offset.min(max_scroll)
    };

    let para = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(if focused {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                })
                .title(title)
                .title_alignment(Alignment::Left),
        )
        .scroll((scroll_top, 0));

    f.render_widget(para, area);
}

/// Draw the transmit input area
fn draw_tx_area(f: &mut Frame, app: &AppState, area: Rect) {
    // Store area for mouse interaction
    unsafe {
        UI_AREAS.tx_area = area;
    }

    // Split the TX area into input and append selector
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(30), Constraint::Length(12)])
        .split(area);

    draw_tx_input(f, app, chunks[0]);
    draw_append_selector(f, app, chunks[1]);
}

/// Draw the TX input box
fn draw_tx_input(f: &mut Frame, app: &AppState, area: Rect) {
    let focused = app.focused_field == FocusedField::TxInput;
    let mode_str = match app.tx_mode {
        TxMode::Hex => "HEX",
        TxMode::Ascii => "ASCII",
    };

    let mode_icon = match app.tx_mode {
        TxMode::Hex => "🔢",
        TxMode::Ascii => "📝",
    };

    let title = if focused {
        format!(
            " {} 发送 - {} [↑↓ 切换 | Enter 发送 | Esc 清空] ",
            mode_icon, mode_str
        )
    } else {
        format!(" {} 发送 - {} ", mode_icon, mode_str)
    };

    let style = if focused {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let cursor_line = if app.tx_input.is_empty() {
        if focused {
            Line::from(vec![
                Span::styled("▮", Style::default().fg(Color::Yellow)),
                Span::styled(" 输入数据...", Style::default().fg(Color::DarkGray)),
            ])
        } else {
            Line::from(Span::styled(
                "输入数据...",
                Style::default().fg(Color::DarkGray),
            ))
        }
    } else {
        let display_text = if focused {
            format!("{}▮", app.tx_input)
        } else {
            app.tx_input.clone()
        };
        Line::from(Span::styled(
            display_text,
            Style::default().fg(Color::White),
        ))
    };

    let help_text = match app.tx_mode {
        TxMode::Hex => "HEX: 按空格分隔字节 (例: 48 65 6C 6C 6F)",
        TxMode::Ascii => "ASCII: 直接输入文本内容",
    };

    let text = vec![
        Line::from(""),
        cursor_line,
        Line::from(""),
        Line::from(Span::styled(
            help_text,
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let para = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(style),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(para, area);
}

/// Draw the append mode selector
fn draw_append_selector(f: &mut Frame, app: &AppState, area: Rect) {
    let items: Vec<ListItem> = app
        .append_mode_options
        .iter()
        .map(|mode| {
            let display = mode.name();
            ListItem::new(display)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" 追加 ")
                .title_alignment(Alignment::Left),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("► ");

    f.render_stateful_widget(list, area, &mut app.append_mode_state.clone());
}

/// Draw the control/status bar at the bottom
fn draw_control_area(f: &mut Frame, app: &AppState, area: Rect) {
    // Store area for mouse interaction
    unsafe {
        UI_AREAS.control_area = area;
    }

    let auto_scroll_icon = if app.auto_scroll { "🔄" } else { "⏸" };

    let stats = vec![
        Span::styled(
            "TX: ",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{} ", app.message_log.tx_count),
            Style::default().fg(Color::White),
        ),
        Span::raw("│ "),
        Span::styled(
            "RX: ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{} ", app.message_log.rx_count),
            Style::default().fg(Color::White),
        ),
        Span::raw("│ "),
        Span::styled(
            format!(
                "{} {}",
                auto_scroll_icon,
                if app.auto_scroll {
                    "自动滚动"
                } else {
                    "手动滚动"
                }
            ),
            if app.auto_scroll {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Yellow)
            },
        ),
    ];

    let para = Paragraph::new(Line::from(stats))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" 统计信息 ")
                .title_alignment(Alignment::Left),
        )
        .alignment(Alignment::Left);

    f.render_widget(para, area);
}

/// Draw the notification bar at the bottom
fn draw_notification_bar(f: &mut Frame, app: &AppState, area: Rect) {
    if let Some(notification) = app.notifications.front() {
        let (color, emoji) = match notification.level {
            NotificationLevel::Error => (Color::Red, "❌"),
            NotificationLevel::Warning => (Color::Yellow, "⚠️"),
            NotificationLevel::Success => (Color::Green, "✅"),
            NotificationLevel::Info => (Color::Cyan, "ℹ️"),
        };

        // Calculate remaining time
        let elapsed = notification.created_at.elapsed().as_millis() as u64;
        let remaining = notification.duration_ms.saturating_sub(elapsed);
        let remaining_secs = (remaining / 1000) as f32;

        let text = Line::from(vec![
            Span::raw(" "),
            Span::styled(emoji, Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" "),
            Span::styled(
                &notification.message,
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(
                format!("[{:.1}s]", remaining_secs),
                Style::default().fg(Color::DarkGray),
            ),
        ]);

        let para = Paragraph::new(text).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" 消息提示 ")
                .title_alignment(Alignment::Left)
                .border_style(Style::default().fg(color)),
        );

        f.render_widget(para, area);
    } else {
        let para = Paragraph::new(Line::from(Span::styled(
            "准备就绪",
            Style::default().fg(Color::DarkGray),
        )))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" 消息提示 ")
                .title_alignment(Alignment::Left),
        )
        .alignment(Alignment::Center);

        f.render_widget(para, area);
    }
}
