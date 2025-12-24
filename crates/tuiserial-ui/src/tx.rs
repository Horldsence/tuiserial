//! TX (transmit) area rendering - input box and append mode selector
//!
//! This module handles rendering of the transmission area including
//! the input box for data entry and the append mode selector.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};
use tuiserial_core::{AppState, FocusedField, TxMode};

use crate::areas::{update_area, UiAreaField};

/// Draw the transmit input area
pub fn draw_tx_area(f: &mut Frame, app: &AppState, area: Rect) {
    // Store area for mouse interaction
    update_area(UiAreaField::TxArea, area);

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
        .highlight_symbol("> ");

    f.render_stateful_widget(list, area, &mut app.append_mode_state.clone());
}
