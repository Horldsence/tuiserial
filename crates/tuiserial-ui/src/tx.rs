//! TX (transmit) area rendering - input box and append mode selector
//!
//! This module handles rendering of the transmission area including
//! the input box for data entry and the append mode selector.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};
use rust_i18n::t;
use tuiserial_core::{AppState, FocusedField, TxMode, display_width};

use crate::areas::{UiAreaField, update_area, update_cursor_state};

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
        TxMode::Hex => t!("tx.hex"),
        TxMode::Ascii => t!("tx.ascii"),
    };

    let mode_icon = match app.tx_mode {
        TxMode::Hex => "🔢",
        TxMode::Ascii => "📝",
    };

    let title = if focused {
        format!(
            " {} {} - {} [↑↓ {} | Enter {} | Esc {}] ",
            mode_icon,
            t!("label.send"),
            mode_str,
            t!("hint.toggle"),
            t!("button.send"),
            t!("hint.clear")
        )
    } else {
        format!(" {} {} - {} ", mode_icon, t!("label.send"), mode_str)
    };

    let style = if focused {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let prompt_text = t!("label.input_prompt");

    let inner_width = area.width.saturating_sub(2) as usize; // minus borders

    // Compute visible text and cursor visual position, with horizontal scrolling
    let (visible_text, cursor_visual_x) = if app.tx_input.is_empty() {
        (prompt_text.to_string(), 0)
    } else {
        let chars: Vec<char> = app.tx_input.chars().collect();
        let cursor_pos = app.tx_cursor.min(chars.len());
        let text_before_cursor: String = chars[..cursor_pos].iter().collect();
        let cursor_x = display_width(&text_before_cursor);
        let text_width = display_width(&app.tx_input);

        if text_width <= inner_width {
            (app.tx_input.clone(), cursor_x)
        } else {
            // Stateless scroll: keep cursor visible within the input box
            let max_scroll = text_width.saturating_sub(inner_width);
            let scroll = if cursor_x < inner_width {
                0
            } else {
                cursor_x
                    .saturating_sub(inner_width.saturating_sub(1))
                    .min(max_scroll)
            };

            // Find start char index from display scroll position
            let mut acc = 0;
            let mut start = 0;
            for (i, c) in chars.iter().enumerate() {
                let w = if c.is_ascii() { 1 } else { 2 };
                if acc + w > scroll {
                    start = i;
                    break;
                }
                acc += w;
            }

            // Find end char index within visible area
            let mut acc = 0;
            let mut end = chars.len();
            for (i, c) in chars.iter().enumerate().skip(start) {
                let w = if c.is_ascii() { 1 } else { 2 };
                if acc + w > inner_width {
                    end = i;
                    break;
                }
                acc += w;
            }

            let visible: String = chars[start..end].iter().collect();
            let visual_x = cursor_x.saturating_sub(scroll);
            (visible, visual_x)
        }
    };

    let cursor_line = if app.tx_input.is_empty() {
        Line::from(Span::styled(
            visible_text,
            Style::default().fg(Color::DarkGray),
        ))
    } else {
        Line::from(Span::styled(
            visible_text,
            Style::default().fg(Color::White),
        ))
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

    // Position native terminal cursor
    if focused {
        let cursor_x = area.x + 1 + cursor_visual_x.min(inner_width) as u16;
        let cursor_y = area.y + 2; // border top + empty first line
        f.set_cursor_position((cursor_x, cursor_y));
        update_cursor_state(cursor_x, cursor_y, true);
    } else {
        update_cursor_state(0, 0, false);
    }
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
                .title(format!(" {} ", t!("label.append_mode")))
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
