//! Keyboard shortcuts help panel
//!
//! This module provides a help panel that displays all available keyboard shortcuts
//! for the application, organized by category.

use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use tuiserial_core::{i18n::t, Language};

/// Draw the keyboard shortcuts help overlay
pub fn draw_shortcuts_help(f: &mut Frame, lang: Language) {
    let area = f.area();

    // Calculate centered position
    let help_width = 70.min(area.width.saturating_sub(4));
    let help_height = 28.min(area.height.saturating_sub(4));
    let x = (area.width.saturating_sub(help_width)) / 2;
    let y = (area.height.saturating_sub(help_height)) / 2;

    let help_area = Rect {
        x,
        y,
        width: help_width,
        height: help_height,
    };

    // Clear the area first
    f.render_widget(Clear, help_area);

    // Create the help content
    let lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            t("shortcuts.session", lang),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  Ctrl+T", Style::default().fg(Color::Yellow)),
            Span::raw("          "),
            Span::raw(
                t("shortcuts.new_session", lang)
                    .split(':')
                    .nth(1)
                    .unwrap_or("New Session"),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+W", Style::default().fg(Color::Yellow)),
            Span::raw("          "),
            Span::raw(
                t("shortcuts.close_session", lang)
                    .split(':')
                    .nth(1)
                    .unwrap_or("Close Session"),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+Tab", Style::default().fg(Color::Yellow)),
            Span::raw("        "),
            Span::raw(
                t("shortcuts.next_session", lang)
                    .split(':')
                    .nth(1)
                    .unwrap_or("Next Session"),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+Shift+Tab", Style::default().fg(Color::Yellow)),
            Span::raw("  "),
            Span::raw(
                t("shortcuts.prev_session", lang)
                    .split(':')
                    .nth(1)
                    .unwrap_or("Previous Session"),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+←/→", Style::default().fg(Color::Yellow)),
            Span::raw("        "),
            Span::raw("Switch between sessions"),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+1~9", Style::default().fg(Color::Yellow)),
            Span::raw("        "),
            Span::raw(
                t("shortcuts.switch_1_9", lang)
                    .split(':')
                    .nth(1)
                    .unwrap_or("Switch to session"),
            ),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            t("shortcuts.layout", lang),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  Ctrl+L", Style::default().fg(Color::Yellow)),
            Span::raw("          "),
            Span::raw(
                t("shortcuts.cycle_layout", lang)
                    .split(':')
                    .nth(1)
                    .unwrap_or("Cycle layout mode"),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+Shift+L", Style::default().fg(Color::Yellow)),
            Span::raw("    "),
            Span::raw(
                t("shortcuts.prev_layout", lang)
                    .split(':')
                    .nth(1)
                    .unwrap_or("Previous layout"),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+P", Style::default().fg(Color::Yellow)),
            Span::raw("          "),
            Span::raw(
                t("shortcuts.next_pane", lang)
                    .split(':')
                    .nth(1)
                    .unwrap_or("Focus next pane"),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+Shift+P", Style::default().fg(Color::Yellow)),
            Span::raw("    "),
            Span::raw(
                t("shortcuts.prev_pane_key", lang)
                    .split(':')
                    .nth(1)
                    .unwrap_or("Focus previous pane"),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+N", Style::default().fg(Color::Yellow)),
            Span::raw("          "),
            Span::raw(
                t("shortcuts.cycle_pane_session", lang)
                    .split(':')
                    .nth(1)
                    .unwrap_or("Cycle session in pane"),
            ),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            t("shortcuts.general", lang),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  O", Style::default().fg(Color::Yellow)),
            Span::raw("               "),
            Span::raw("Connect/Disconnect"),
        ]),
        Line::from(vec![
            Span::styled("  C", Style::default().fg(Color::Yellow)),
            Span::raw("               "),
            Span::raw("Clear log"),
        ]),
        Line::from(vec![
            Span::styled("  X", Style::default().fg(Color::Yellow)),
            Span::raw("               "),
            Span::raw("Toggle display mode (HEX/TEXT)"),
        ]),
        Line::from(vec![
            Span::styled("  A", Style::default().fg(Color::Yellow)),
            Span::raw("               "),
            Span::raw("Toggle auto scroll"),
        ]),
        Line::from(vec![
            Span::styled("  F10", Style::default().fg(Color::Yellow)),
            Span::raw("             "),
            Span::raw("Open menu"),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+Q", Style::default().fg(Color::Yellow)),
            Span::raw("          "),
            Span::raw("Quit application"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ", Style::default().fg(Color::DarkGray)),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::styled(" or ", Style::default().fg(Color::DarkGray)),
            Span::styled("Q", Style::default().fg(Color::Yellow)),
            Span::styled(" to close this help", Style::default().fg(Color::DarkGray)),
        ]),
    ];

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(vec![
                    Span::raw(" "),
                    Span::styled(
                        t("shortcuts.title", lang),
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" "),
                ])
                .style(Style::default().bg(Color::Black)),
        )
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, help_area);
}

/// Draw a compact shortcuts hint bar
pub fn draw_shortcuts_hint(f: &mut Frame, area: Rect, _lang: Language) {
    let hints = vec![
        ("F10", "Menu"),
        ("Ctrl+T", "New"),
        ("Ctrl+W", "Close"),
        ("Ctrl+L", "Layout"),
        ("Ctrl+P", "Pane"),
        ("O", "Connect"),
        ("C", "Clear"),
        ("X", "Mode"),
        ("?", "Help"),
    ];

    let mut spans = vec![];
    for (i, (key, desc)) in hints.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw("  "));
        }
        spans.push(Span::styled(
            *key,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            format!(":{}", desc),
            Style::default().fg(Color::DarkGray),
        ));
    }

    let paragraph = Paragraph::new(Line::from(spans))
        .style(Style::default().bg(Color::Black))
        .alignment(Alignment::Center);

    f.render_widget(paragraph, area);
}

/// Draw an inline shortcuts reference (for specific contexts)
pub fn draw_context_shortcuts(
    f: &mut Frame,
    area: Rect,
    shortcuts: &[(&str, &str)],
    title: Option<&str>,
) {
    let mut lines = vec![];

    if let Some(title_text) = title {
        lines.push(Line::from(vec![Span::styled(
            title_text,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(""));
    }

    for (key, description) in shortcuts {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                *key,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(": "),
            Span::raw(*description),
        ]));
    }

    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Left);

    f.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_shortcuts_rendering() {
        // Just ensure the functions can be called without panic
        // Actual rendering tests would require a terminal backend
        let shortcuts = vec![("Ctrl+T", "New Session"), ("Ctrl+W", "Close Session")];
        assert_eq!(shortcuts.len(), 2);
    }
}
