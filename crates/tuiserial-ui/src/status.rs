//! Status panel and control area rendering
//!
//! This module handles rendering of the status panel showing connection info
//! and the control area showing statistics.

use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use tuiserial_core::{i18n::t, AppState, Parity};

use crate::areas::{update_area, UiAreaField};

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
        t("status.connected", app.language)
    } else {
        t("status.disconnected", app.language)
    };

    let config_status = if app.config_locked {
        ("🔒", t("status.locked", app.language), Color::Yellow)
    } else {
        ("🔓", t("status.modifiable", app.language), Color::Green)
    };

    // Format parity display
    let parity_str = match app.config.parity {
        Parity::None => t("parity.none", app.language).chars().next().unwrap_or('N'),
        Parity::Even => t("parity.even", app.language).chars().next().unwrap_or('E'),
        Parity::Odd => t("parity.odd", app.language).chars().next().unwrap_or('O'),
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
                format!("{}: ", t("label.port", app.language)),
                Style::default().fg(Color::Cyan),
            ),
            Span::raw(if app.config.port.is_empty() {
                t("status.not_connected", app.language)
                    .split('-')
                    .next()
                    .unwrap_or("Not selected")
                    .trim()
            } else {
                &app.config.port
            }),
        ]),
        Line::from(vec![
            Span::styled(
                format!("{}: ", t("label.baud_rate", app.language)),
                Style::default().fg(Color::Cyan),
            ),
            Span::raw(format!("{}", app.config.baud_rate)),
        ]),
        Line::from(vec![
            Span::styled(
                format!("{}: ", t("label.data_bits", app.language)),
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
            Span::raw(format!(" {}  ", t("hint.select", app.language))),
            Span::styled(
                "r",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!(" {}", t("hint.refresh", app.language))),
        ]),
        Line::from(vec![
            Span::styled(
                "q",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!(" {}  ", t("hint.quit", app.language))),
            Span::styled(
                "Tab",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!(" {}", t("hint.switch", app.language))),
        ]),
    ];

    let para = Paragraph::new(text).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", t("label.status", app.language)))
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
            format!("{}: ", t("label.tx_count", app.language)),
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
            format!("{}: ", t("label.rx_count", app.language)),
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
                t("hint.auto_scroll", app.language)
            ),
            if app.auto_scroll {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Yellow)
            },
        ),
        Span::raw(" │ "),
        Span::styled(
            format!("{}: ", t("plugin.bar.loaded", app.language)),
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

    // Add error count if any
    let mut final_stats: Vec<Span> = stats;
    if app.plugin_error_count > 0 {
        final_stats.push(Span::raw(" "));
        final_stats.push(Span::styled(
            format!("✗{}", app.plugin_error_count),
            Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD),
        ));
    }

    let para = Paragraph::new(Line::from(final_stats))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {} ", t("label.statistics", app.language)))
                .title_alignment(Alignment::Left),
        )
        .alignment(Alignment::Left);

    f.render_widget(para, area);
}
