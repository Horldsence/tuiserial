//! Notification bar rendering - displays user messages and alerts
//!
//! This module handles rendering of the notification bar at the bottom of the screen
//! showing temporary messages with different severity levels.

use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use rust_i18n::t;
use tuiserial_core::{AppState, NotificationLevel};

use crate::areas::{UiAreaField, update_area};

/// Draw the notification bar at the bottom
pub fn draw_notification_bar(f: &mut Frame, app: &AppState, area: Rect) {
    // Store area for mouse interaction
    update_area(UiAreaField::NotificationArea, area);

    if let Some(notification) = app.notifications.front() {
        draw_active_notification(f, notification, area);
    } else {
        draw_empty_notification(f, app, area);
    }
}

/// Draw an active notification message
fn draw_active_notification(
    f: &mut Frame,
    notification: &tuiserial_core::Notification,
    area: Rect,
) {
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
            .title(format!(" {} ", t!("label.message")))
            .title_alignment(Alignment::Left)
            .border_style(Style::default().fg(color)),
    );

    f.render_widget(para, area);
}

/// Draw empty notification bar — shows the most recent persistent error
/// if one exists, otherwise shows "Ready".
fn draw_empty_notification(f: &mut Frame, app: &AppState, area: Rect) {
    // If there's a recent error in the log, show it persistently.
    if let Some(entry) = app.error_log.most_recent_error() {
        let count_hint = if entry.count > 1 {
            format!(" (×{})", entry.count)
        } else {
            String::new()
        };
        let text = Line::from(vec![
            Span::raw(" "),
            Span::styled("❌", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" "),
            Span::styled(
                format!("{}{}", entry.error.to_user_message(), count_hint),
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
        ]);

        let para = Paragraph::new(text).block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {} ", t!("label.message")))
                .title_alignment(Alignment::Left)
                .border_style(Style::default().fg(Color::Red)),
        );

        f.render_widget(para, area);
        return;
    }

    let ready_text = if app.language == tuiserial_core::Language::Chinese {
        "准备就绪"
    } else {
        "Ready"
    };

    let para = Paragraph::new(Line::from(Span::styled(
        ready_text,
        Style::default().fg(Color::DarkGray),
    )))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", t!("label.message")))
            .title_alignment(Alignment::Left),
    )
    .alignment(Alignment::Center);

    f.render_widget(para, area);
}
