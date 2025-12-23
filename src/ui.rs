use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::model::{AppState, DisplayMode, FlowControl, FocusedField, Parity, TxMode, NotificationLevel, LogDirection};

pub fn draw(f: &mut Frame, app: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(15),
            Constraint::Length(3), // Notification area
        ])
        .split(f.area());

    draw_main_content(f, app, chunks[0]);
    draw_notification_bar(f, app, chunks[1]);
}

fn draw_main_content(f: &mut Frame, app: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(40), Constraint::Min(50)])
        .split(area);

    draw_config_panel(f, app, chunks[0]);
    draw_main_area(f, app, chunks[1]);
}

fn draw_config_panel(f: &mut Frame, app: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Port
            Constraint::Length(4),  // Baud
            Constraint::Length(3),  // Data bits
            Constraint::Length(3),  // Parity
            Constraint::Length(3),  // Stop bits
            Constraint::Length(3),  // Flow control
            Constraint::Min(5),     // Status
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

fn draw_main_area(f: &mut Frame, app: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(8),     // Log area
            Constraint::Length(6),  // TX area
            Constraint::Length(3),  // Status bar
        ])
        .split(area);

    draw_log_area(f, app, chunks[0]);
    draw_tx_area(f, app, chunks[1]);
    draw_control_area(f, app, chunks[2]);
}

fn draw_port_dropdown(f: &mut Frame, app: &AppState, area: Rect) {
    let focused = app.focused_field == FocusedField::Port;
    let title = if focused { " 串口 (↑↓ 选择) " } else { " 串口 " };
    let style = if focused { Style::default().fg(Color::Yellow) } else { Style::default() };
    
    let items: Vec<ListItem> = app.ports.iter().map(|p| ListItem::new(p.as_str())).collect();
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title).border_style(style))
        .highlight_style(Style::default().bg(Color::DarkGray));

    f.render_stateful_widget(list, area, &mut app.port_list_state.clone());
}

fn draw_baud_rate_dropdown(f: &mut Frame, app: &AppState, area: Rect) {
    let focused = app.focused_field == FocusedField::BaudRate;
    let title = if focused { " 波特率 (←→ 变更) " } else { " 波特率 " };
    let style = if focused { Style::default().fg(Color::Yellow) } else { Style::default() };

    let items: Vec<ListItem> = app
        .baud_rate_options
        .iter()
        .map(|b| ListItem::new(b.to_string()))
        .collect();
    
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title).border_style(style))
        .highlight_style(Style::default().bg(Color::DarkGray));

    f.render_stateful_widget(list, area, &mut app.baud_rate_state.clone());
}

fn draw_data_bits_dropdown(f: &mut Frame, app: &AppState, area: Rect) {
    let focused = app.focused_field == FocusedField::DataBits;
    let title = if focused { " 数据位 (↑↓ 选择) " } else { " 数据位 " };
    let style = if focused { Style::default().fg(Color::Yellow) } else { Style::default() };

    let items: Vec<ListItem> = app
        .data_bits_options
        .iter()
        .map(|b| ListItem::new(b.to_string()))
        .collect();
    
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title).border_style(style))
        .highlight_style(Style::default().bg(Color::DarkGray));

    f.render_stateful_widget(list, area, &mut app.data_bits_state.clone());
}

fn draw_parity_dropdown(f: &mut Frame, app: &AppState, area: Rect) {
    let focused = app.focused_field == FocusedField::Parity;
    let title = if focused { " 校验位 (p: 切换) " } else { " 校验位 " };
    let style = if focused { Style::default().fg(Color::Yellow) } else { Style::default() };

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
        .block(Block::default().borders(Borders::ALL).title(title).border_style(style))
        .highlight_style(Style::default().bg(Color::DarkGray));

    f.render_stateful_widget(list, area, &mut app.parity_state.clone());
}

fn draw_stop_bits_dropdown(f: &mut Frame, app: &AppState, area: Rect) {
    let focused = app.focused_field == FocusedField::StopBits;
    let title = if focused { " 停止位 (↑↓ 选择) " } else { " 停止位 " };
    let style = if focused { Style::default().fg(Color::Yellow) } else { Style::default() };

    let items: Vec<ListItem> = app
        .stop_bits_options
        .iter()
        .map(|b| ListItem::new(b.to_string()))
        .collect();
    
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title).border_style(style))
        .highlight_style(Style::default().bg(Color::DarkGray));

    f.render_stateful_widget(list, area, &mut app.stop_bits_state.clone());
}

fn draw_flow_control_dropdown(f: &mut Frame, app: &AppState, area: Rect) {
    let focused = app.focused_field == FocusedField::FlowControl;
    let title = if focused { " 流控制 (f: 切换) " } else { " 流控制 " };
    let style = if focused { Style::default().fg(Color::Yellow) } else { Style::default() };

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
        .block(Block::default().borders(Borders::ALL).title(title).border_style(style))
        .highlight_style(Style::default().bg(Color::DarkGray));

    f.render_stateful_widget(list, area, &mut app.flow_control_state.clone());
}

fn draw_status_panel(f: &mut Frame, app: &AppState, area: Rect) {
    let status = if app.is_connected {
        Span::styled("✓ 已连接", Style::default().fg(Color::Green))
    } else {
        Span::styled("✗ 未连接", Style::default().fg(Color::Red))
    };

    let text = vec![
        Line::from(vec![Span::raw("状态: "), status]),
        Line::raw(""),
        Line::from(vec![
            Span::raw("按 "),
            Span::styled("Tab", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" 切换字段"),
        ]),
        Line::from(vec![
            Span::raw("按 "),
            Span::styled("o", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" 打开/关闭"),
        ]),
    ];

    let para = Paragraph::new(text).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" 连接 ")
            .title_alignment(Alignment::Left),
    );

    f.render_widget(para, area);
}

fn draw_log_area(f: &mut Frame, app: &AppState, area: Rect) {
    let focused = app.focused_field == FocusedField::LogArea;
    if app.message_log.entries.is_empty() {
        let para = Paragraph::new("(等待数据...)")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(if focused { Style::default().fg(Color::Yellow) } else { Style::default() })
                    .title(" LOG ")
                    .title_alignment(Alignment::Left),
            )
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(para, area);
        return;
    }

    let mut lines: Vec<Line> = Vec::new();
    for entry in app.message_log.entries.iter() {
        let time_color = match entry.direction {
            LogDirection::Rx => Color::Cyan,
            LogDirection::Tx => Color::Green,
        };

        let time_str = entry.timestamp.format("%H:%M:%S%.3f").to_string();
        let dir_str = match entry.direction {
            LogDirection::Rx => "RX",
            LogDirection::Tx => "TX",
        };

        let data_str = match app.display_mode {
            DisplayMode::Hex => crate::serial::bytes_to_hex(&entry.data),
            DisplayMode::Text => crate::serial::bytes_to_string(&entry.data),
        };

        let mut spans: Vec<Span> = Vec::new();
        spans.push(Span::styled(format!("[{} {}] ", time_str, dir_str), Style::default().fg(time_color)));
        spans.push(Span::styled(data_str, Style::default().fg(Color::White)));
        lines.push(Line::from(spans));
    }

    let title = match app.display_mode {
        DisplayMode::Hex => " HEX (↑↓ 切换模式) ",
        DisplayMode::Text => " TEXT (↑↓ 切换模式) ",
    };

    let total_lines = lines.len() as u16;
    let viewport_lines = area.height.saturating_sub(2).max(1); // minus borders
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
                .border_style(if focused { Style::default().fg(Color::Yellow) } else { Style::default() })
                .title(title)
                .title_alignment(Alignment::Left),
        )
        .scroll((scroll_top, 0));

    f.render_widget(para, area);
}

fn draw_tx_area(f: &mut Frame, app: &AppState, area: Rect) {
    let focused = app.focused_field == FocusedField::TxInput;
    let mode_str = match app.tx_mode {
        TxMode::Hex => "HEX",
        TxMode::Ascii => "ASCII",
    };
    let title = if focused { 
        format!(" 发送数据 [{} 模式] (↑↓ 切换模式, Enter: 发送) ", mode_str)
    } else {
        format!(" 发送数据 [{} 模式] ", mode_str)
    };
    
    let style = if focused { 
        Style::default().fg(Color::Yellow) 
    } else { 
        Style::default() 
    };

    let cursor_line = if app.tx_input.is_empty() {
        Span::raw("输入要发送的数据...")
    } else {
        let display_text = format!("{}▮", app.tx_input);
        Span::raw(display_text)
    };

    let text = vec![
        Line::from(cursor_line),
        Line::raw(""),
        Line::from("HEX 格式: 按空格分隔 (如: 48 65 6C 6C 6F)"),
        Line::from("ASCII 格式: 直接输入文本"),
    ];

    let para = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title(title).border_style(style))
        .wrap(Wrap { trim: true });

    f.render_widget(para, area);
}

fn draw_control_area(f: &mut Frame, app: &AppState, area: Rect) {
    // Status bar
    let stats = format!(
        " Tx: {} │ Rx: {} │ RX Mode: {} │ Auto: {} ",
        app.message_log.tx_count,
        app.message_log.rx_count,
        if app.display_mode == DisplayMode::Hex {
            "HEX"
        } else {
            "TEXT"
        },
        if app.auto_scroll { "ON" } else { "OFF" }
    );

    let para = Paragraph::new(stats)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" 状态 ")
                .title_alignment(Alignment::Left),
        )
        .style(Style::default().fg(Color::Gray));

    f.render_widget(para, area);
}

fn draw_notification_bar(f: &mut Frame, app: &AppState, area: Rect) {
    if let Some(notification) = app.notifications.front() {
        let (color, emoji) = match notification.level {
            NotificationLevel::Error => (Color::Red, "✗"),
            NotificationLevel::Warning => (Color::Yellow, "⚠"),
            NotificationLevel::Success => (Color::Green, "✓"),
            NotificationLevel::Info => (Color::Cyan, "ℹ"),
        };

        let text = format!("{} {}", emoji, notification.message);
        let para = Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" 提示 ")
                    .title_alignment(Alignment::Left),
            )
            .style(Style::default().fg(color));

        f.render_widget(para, area);
    } else {
        let para = Paragraph::new("")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" 提示 ")
                    .title_alignment(Alignment::Left),
            )
            .style(Style::default().fg(Color::Gray));

        f.render_widget(para, area);
    }
}
