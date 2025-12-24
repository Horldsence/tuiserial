//! Tab bar UI rendering for multiple sessions
//!
//! This module provides UI rendering functions for displaying session tabs,
//! allowing users to see and switch between multiple serial port sessions.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};

use crate::session::SessionManager;

/// Render the tab bar showing all sessions
pub fn draw_tab_bar(
    f: &mut Frame,
    area: Rect,
    session_manager: &SessionManager,
    show_border: bool,
) {
    let active_idx = session_manager.active_index();
    let sessions = session_manager.sessions();

    // Build tab titles
    let titles: Vec<Line> = sessions
        .iter()
        .enumerate()
        .map(|(idx, session)| {
            let mut spans = vec![];

            // Add connection indicator
            if session.is_connected {
                spans.push(Span::styled("● ", Style::default().fg(Color::Green)));
            } else {
                spans.push(Span::styled("○ ", Style::default().fg(Color::DarkGray)));
            }

            // Add session name
            spans.push(Span::raw(&session.name));

            // Add close button hint for active tab
            if idx == active_idx {
                spans.push(Span::styled(" [×]", Style::default().fg(Color::Red)));
            }

            Line::from(spans)
        })
        .collect();

    let tabs = Tabs::new(titles)
        .block(if show_border {
            Block::default().borders(Borders::ALL).title(" Sessions ")
        } else {
            Block::default()
        })
        .select(active_idx)
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
                .bg(Color::DarkGray),
        )
        .divider(Span::raw(" | "));

    f.render_widget(tabs, area);
}

/// Render a compact tab bar (single line without border)
pub fn draw_compact_tab_bar(f: &mut Frame, area: Rect, session_manager: &SessionManager) {
    draw_tab_bar(f, area, session_manager, false);
}

/// Render tab bar with additional controls
pub fn draw_tab_bar_with_controls(
    f: &mut Frame,
    area: Rect,
    session_manager: &SessionManager,
    show_help: bool,
) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(10), Constraint::Length(40)])
        .split(area);

    // Draw main tab bar
    draw_tab_bar(f, chunks[0], session_manager, false);

    // Draw help text
    if show_help {
        let help_text = " Ctrl+T: New | Ctrl+W: Close | Ctrl+←/→: Switch ";
        let help = Paragraph::new(help_text)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Right);
        f.render_widget(help, chunks[1]);
    }
}

/// Render session info overlay (for renaming, etc.)
pub fn draw_session_info_overlay(
    f: &mut Frame,
    session_name: &str,
    is_editing: bool,
    cursor_pos: usize,
) {
    let area = f.area();

    // Calculate overlay position (centered)
    let overlay_width = 50;
    let overlay_height = 5;
    let x = (area.width.saturating_sub(overlay_width)) / 2;
    let y = (area.height.saturating_sub(overlay_height)) / 2;

    let overlay_area = Rect {
        x,
        y,
        width: overlay_width,
        height: overlay_height,
    };

    // Clear the area
    let clear_block = Block::default()
        .borders(Borders::ALL)
        .title(" Rename Session ")
        .style(Style::default().bg(Color::Black));
    f.render_widget(clear_block, overlay_area);

    // Calculate inner area
    let inner = Rect {
        x: x + 2,
        y: y + 2,
        width: overlay_width.saturating_sub(4),
        height: overlay_height.saturating_sub(4),
    };

    if is_editing {
        // Show editable text with cursor
        let mut text = session_name.to_string();
        if cursor_pos <= text.len() {
            text.insert(cursor_pos, '█');
        }

        let paragraph = Paragraph::new(text).style(Style::default().fg(Color::Yellow));
        f.render_widget(paragraph, inner);

        // Show hint
        let hint_area = Rect {
            x: inner.x,
            y: inner.y + 1,
            width: inner.width,
            height: 1,
        };
        let hint = Paragraph::new("Press Enter to confirm, Esc to cancel")
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(hint, hint_area);
    } else {
        let paragraph = Paragraph::new(session_name).style(Style::default().fg(Color::White));
        f.render_widget(paragraph, inner);
    }
}

/// Render pane borders with labels
pub fn draw_pane_border(
    f: &mut Frame,
    area: Rect,
    session_name: &str,
    is_focused: bool,
    is_connected: bool,
) -> Rect {
    let border_style = if is_focused {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let connection_indicator = if is_connected {
        Span::styled("● ", Style::default().fg(Color::Green))
    } else {
        Span::styled("○ ", Style::default().fg(Color::DarkGray))
    };

    let title = Line::from(vec![
        Span::raw(" "),
        connection_indicator,
        Span::raw(session_name),
        Span::raw(" "),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(title);

    let inner = block.inner(area);
    f.render_widget(block, area);

    inner
}

/// Render layout mode indicator
pub fn draw_layout_indicator(f: &mut Frame, area: Rect, layout_name: &str) {
    let text = format!(" Layout: {} ", layout_name);
    let widget = Paragraph::new(text)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(widget, area);
}

/// Render a small notification badge on tabs
pub fn draw_tab_notification_badge(f: &mut Frame, area: Rect, count: usize) {
    if count == 0 {
        return;
    }

    let badge_text = if count > 99 {
        "99+".to_string()
    } else {
        count.to_string()
    };

    let badge = Paragraph::new(badge_text)
        .style(
            Style::default()
                .fg(Color::White)
                .bg(Color::Red)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);

    f.render_widget(badge, area);
}

/// Render session list (for session picker)
pub fn draw_session_list(
    f: &mut Frame,
    area: Rect,
    sessions: &[(usize, String, bool)], // (index, name, is_connected)
    _selected: Option<usize>,
) {
    use ratatui::widgets::{List, ListItem};

    let items: Vec<ListItem> = sessions
        .iter()
        .map(|(idx, name, is_connected)| {
            let connection = if *is_connected { "●" } else { "○" };
            let color = if *is_connected {
                Color::Green
            } else {
                Color::DarkGray
            };

            let line = Line::from(vec![
                Span::styled(format!("{} ", connection), Style::default().fg(color)),
                Span::raw(format!("[{}] ", idx + 1)),
                Span::raw(name),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Select Session "),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
                .bg(Color::DarkGray),
        )
        .highlight_symbol(">> ");

    f.render_widget(list, area);
}

/// Calculate tab bar height based on number of sessions
pub fn calculate_tab_bar_height(session_count: usize, with_border: bool) -> u16 {
    if session_count <= 1 {
        0 // Hide tab bar if only one session
    } else if with_border {
        3 // Tab content + border
    } else {
        1 // Just the tab bar
    }
}

/// Helper to get tab area at cursor position (for mouse clicks)
pub fn get_tab_at_position(
    area: Rect,
    x: u16,
    y: u16,
    session_count: usize,
    _active_index: usize,
) -> Option<usize> {
    // Check if click is within tab bar area
    if y < area.y || y >= area.y + 1 || x < area.x || x >= area.x + area.width {
        return None;
    }

    // Estimate tab width (simplified calculation)
    // Real implementation would need actual tab width calculations
    let avg_tab_width = area.width / session_count.max(1) as u16;
    let relative_x = x.saturating_sub(area.x);
    let tab_index = (relative_x / avg_tab_width.max(1)) as usize;

    if tab_index < session_count {
        Some(tab_index)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_tab_bar_height() {
        assert_eq!(calculate_tab_bar_height(0, false), 0);
        assert_eq!(calculate_tab_bar_height(1, false), 0);
        assert_eq!(calculate_tab_bar_height(2, false), 1);
        assert_eq!(calculate_tab_bar_height(2, true), 3);
    }

    #[test]
    fn test_get_tab_at_position() {
        let area = Rect {
            x: 0,
            y: 0,
            width: 80,
            height: 1,
        };

        assert_eq!(get_tab_at_position(area, 10, 0, 4, 0), Some(0));
        assert_eq!(get_tab_at_position(area, 30, 0, 4, 0), Some(1));
        assert_eq!(get_tab_at_position(area, 10, 5, 4, 0), None); // Outside area
    }
}
