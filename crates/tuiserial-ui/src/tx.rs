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
use tuiserial_core::{i18n::t, AppState, FocusedField, TxMode};

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
        TxMode::Hex => t("tx.hex", app.language),
        TxMode::Ascii => t("tx.ascii", app.language),
    };

    let mode_icon = match app.tx_mode {
        TxMode::Hex => "🔢",
        TxMode::Ascii => "📝",
    };

    let title = if focused {
        format!(
            " {} {} - {} [↑↓ {} | Enter {} | Esc {}] ",
            mode_icon,
            t("label.send", app.language),
            mode_str,
            t("hint.toggle", app.language),
            t("button.send", app.language),
            t("hint.clear", app.language)
        )
    } else {
        format!(
            " {} {} - {} ",
            mode_icon,
            t("label.send", app.language),
            mode_str
        )
    };

    let style = if focused {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let prompt_text = t("label.input_prompt", app.language);
    let cursor_line = if app.tx_input.is_empty() {
        if focused {
            Line::from(vec![
                Span::styled("|", Style::default().fg(Color::Yellow)),
                Span::styled(
                    format!(" {}", prompt_text),
                    Style::default().fg(Color::DarkGray),
                ),
            ])
        } else {
            Line::from(Span::styled(
                prompt_text,
                Style::default().fg(Color::DarkGray),
            ))
        }
    } else {
        if focused {
            // Insert cursor at the correct position
            // Use char indices to ensure we split on character boundaries, not bytes
            let chars: Vec<char> = app.tx_input.chars().collect();
            let cursor_pos = app.tx_cursor.min(chars.len());

            let before_cursor: String = chars[..cursor_pos].iter().collect();
            let after_cursor: String = chars[cursor_pos..].iter().collect();

            Line::from(vec![
                Span::styled(before_cursor, Style::default().fg(Color::White)),
                Span::styled("|", Style::default().fg(Color::Yellow)),
                Span::styled(after_cursor, Style::default().fg(Color::White)),
            ])
        } else {
            Line::from(Span::styled(
                app.tx_input.clone(),
                Style::default().fg(Color::White),
            ))
        }
    };

    let help_text = match app.tx_mode {
        TxMode::Hex => {
            if app.language == tuiserial_core::Language::Chinese {
                "HEX: 按空格分隔字节 (例: 48 65 6C 6C 6F)"
            } else {
                "HEX: Space-separated bytes (e.g., 48 65 6C 6C 6F)"
            }
        }
        TxMode::Ascii => {
            if app.language == tuiserial_core::Language::Chinese {
                "ASCII: 直接输入文本内容"
            } else {
                "ASCII: Enter text directly"
            }
        }
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
            let display = mode.name(app.language);
            ListItem::new(display)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {} ", t("label.append_mode", app.language)))
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
