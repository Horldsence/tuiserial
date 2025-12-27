//! Log area rendering - displays received and transmitted serial data
//!
//! This module handles rendering of the log area showing serial communication history.

use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use tuiserial_core::{i18n::t, AppState, DisplayMode, FocusedField, LogDirection};
use tuiserial_serial::{bytes_to_hex, bytes_to_string};

use crate::areas::{update_area, UiAreaField};

/// Draw the log area showing received and transmitted data
pub fn draw_log_area(f: &mut Frame, app: &AppState, area: Rect) {
    // Store area for mouse interaction
    update_area(UiAreaField::LogArea, area);

    let focused = app.focused_field == FocusedField::LogArea;

    if app.message_log.entries.is_empty() {
        draw_empty_log(f, app, area, focused);
        return;
    }

    draw_log_entries(f, app, area, focused);
}

/// Draw empty log area with help text
fn draw_empty_log(f: &mut Frame, app: &AppState, area: Rect, focused: bool) {
    let status_msg = if app.is_connected {
        t("empty.connect_hint", app.language)
    } else {
        t("status.not_connected", app.language)
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
            t("shortcuts.title", app.language),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(format!("  {}", t("empty.shortcuts", app.language))),
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
                    " {} - {} ",
                    t("label.message", app.language),
                    match app.display_mode {
                        DisplayMode::Hex => t("display.hex", app.language),
                        DisplayMode::Text => t("display.text", app.language),
                    }
                ))
                .title_alignment(Alignment::Left),
        )
        .alignment(Alignment::Center);

    f.render_widget(para, area);
}

/// Draw log entries
fn draw_log_entries(f: &mut Frame, app: &AppState, area: Rect, focused: bool) {
    let mut lines: Vec<Line> = Vec::new();

    for entry in app.message_log.entries.iter() {
        let (time_color, dir_str, dir_icon) = match entry.direction {
            LogDirection::Rx => (Color::Cyan, t("label.rx_count", app.language), "<"),
            LogDirection::Tx => (Color::Green, t("label.tx_count", app.language), ">"),
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
        DisplayMode::Hex => t("display.hex", app.language),
        DisplayMode::Text => t("display.text", app.language),
    };

    let title = format!(
        " {} - {} | {} 条 [x {} | c {}] ",
        t("label.message", app.language),
        display_mode_str,
        app.message_log.entries.len(),
        t("hint.toggle", app.language),
        t("hint.clear", app.language)
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
