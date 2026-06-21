//! Status panel and control area rendering
//!
//! This module handles rendering of the status panel showing connection info
//! and the control area showing statistics.

use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use rust_i18n::t;
use tuiserial_core::{AppState, Parity};

use crate::areas::{UiAreaField, update_area};

/// Draw the connection status panel
pub fn draw_status_panel(f: &mut Frame, app: &AppState, area: Rect) {
    // Store area for mouse interaction
    update_area(UiAreaField::StatusPanel, area);

    let status_color = if app.is_connected {
        Color::Green
    } else {
        Color::Red
    };

    let status_icon = if app.is_connected { "✓" } else { "✗" };
    let status_text = if app.is_connected {
        t!("status.connected")
    } else {
        t!("status.disconnected")
    };

    let config_status = if app.config_locked {
        ("🔒", t!("status.locked"), Color::Yellow)
    } else {
        ("🔓", t!("status.modifiable"), Color::Green)
    };

    // Format parity display
    let parity_str = match app.config.parity {
        Parity::None => t!("parity.none").chars().next().unwrap_or('N'),
        Parity::Even => t!("parity.even").chars().next().unwrap_or('E'),
        Parity::Odd => t!("parity.odd").chars().next().unwrap_or('O'),
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
            Span::styled(
                format!("{}: ", t!("label.port")),
                Style::default().fg(Color::Cyan),
            ),
            Span::raw(if app.config.port.is_empty() {
                let msg = t!("status.not_connected");
                msg.split('-')
                    .next()
                    .unwrap_or("Not selected")
                    .trim()
                    .to_string()
            } else {
                app.config.port.clone()
            }),
        ]),
        Line::from(vec![
            Span::styled(
                format!("{}: ", t!("label.baud_rate")),
                Style::default().fg(Color::Cyan),
            ),
            Span::raw(format!("{}", app.config.baud_rate)),
        ]),
        Line::from(vec![
            Span::styled(
                format!("{}: ", t!("label.data_bits")),
                Style::default().fg(Color::Cyan),
            ),
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
            Span::raw(format!(" {}  ", t!("hint.select"))),
            Span::styled(
                "r",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!(" {}", t!("hint.refresh"))),
        ]),
        Line::from(vec![
            Span::styled(
                "q",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!(" {}  ", t!("hint.quit"))),
            Span::styled(
                "Tab",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!(" {}", t!("hint.switch"))),
        ]),
    ];

    let para = Paragraph::new(text).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", t!("label.status")))
            .title_alignment(Alignment::Left),
    );

    f.render_widget(para, area);
}

/// Draw the control/status bar showing statistics
pub fn draw_control_area(f: &mut Frame, app: &AppState, area: Rect) {
    // Store area for mouse interaction
    update_area(UiAreaField::ControlArea, area);

    let auto_scroll_icon = if app.auto_scroll { "🔄" } else { "⏸" };

    let stats = vec![
        Span::styled(
            format!("{}: ", t!("label.tx_count")),
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
            format!("{}: ", t!("label.rx_count")),
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
            format!("{} {}", auto_scroll_icon, t!("hint.auto_scroll")),
            if app.auto_scroll {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Yellow)
            },
        ),
        Span::raw(" │ "),
        Span::styled(
            format!("{}: ", t!("plugin.bar.loaded")),
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{}", app.plugin_loaded_count),
            Style::default().fg(Color::Green),
        ),
        Span::raw("/"),
        Span::styled(
            format!("{}", app.plugin_total_count),
            Style::default().fg(Color::White),
        ),
    ];

    // Add plugin error count if any
    let mut final_stats: Vec<Span> = stats;
    if app.plugin_error_count > 0 {
        final_stats.push(Span::raw(" "));
        final_stats.push(Span::styled(
            format!("✗{}", app.plugin_error_count),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ));
    }

    // Add global error summary badge if there are errors
    let err_summary = app.error_summary();
    if !err_summary.is_empty() {
        final_stats.push(Span::raw(" │ "));
        final_stats.push(Span::styled(
            format!("[{}]", err_summary),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ));
    }

    let para = Paragraph::new(Line::from(final_stats))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {} ", t!("label.statistics")))
                .title_alignment(Alignment::Left),
        )
        .alignment(Alignment::Left);

    f.render_widget(para, area);
}
