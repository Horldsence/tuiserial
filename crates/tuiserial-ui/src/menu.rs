//! Menu bar and dropdown menu rendering
//!
//! This module handles the rendering of the menu bar and its dropdown menus.

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use tuiserial_core::{i18n::t, AppState, MenuState};

use crate::utils::display_width;

/// Draw the menu bar at the top
pub fn draw_menu_bar(f: &mut Frame, app: &AppState, area: Rect) {
    let lang = app.language;

    // Menu structure: [File, Settings, Help]
    let menus = vec![
        t("menu.file", lang),
        t("menu.settings", lang),
        t("menu.help", lang),
    ];

    let mut spans = Vec::new();

    match app.menu_state {
        MenuState::None => {
            // Normal state - show all menus
            for (i, menu) in menus.iter().enumerate() {
                if i > 0 {
                    spans.push(Span::raw("  "));
                }
                spans.push(Span::styled(
                    format!(" {} ", menu),
                    Style::default().fg(Color::White).bg(Color::DarkGray),
                ));
            }
        }
        MenuState::MenuBar(selected) => {
            // Menu bar focused - highlight selected
            for (i, menu) in menus.iter().enumerate() {
                if i > 0 {
                    spans.push(Span::raw("  "));
                }
                if i == selected {
                    spans.push(Span::styled(
                        format!(" {} ", menu),
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ));
                } else {
                    spans.push(Span::styled(
                        format!(" {} ", menu),
                        Style::default().fg(Color::White).bg(Color::DarkGray),
                    ));
                }
            }
        }
        MenuState::Dropdown(menu_idx, _) => {
            // Dropdown open - highlight parent menu
            for (i, menu) in menus.iter().enumerate() {
                if i > 0 {
                    spans.push(Span::raw("  "));
                }
                if i == menu_idx {
                    spans.push(Span::styled(
                        format!(" {} ", menu),
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ));
                } else {
                    spans.push(Span::styled(
                        format!(" {} ", menu),
                        Style::default().fg(Color::White).bg(Color::DarkGray),
                    ));
                }
            }
        }
    }

    // Fill rest of line with background
    let menu_bar = Paragraph::new(Line::from(spans)).style(Style::default().bg(Color::DarkGray));

    f.render_widget(menu_bar, area);
}

/// Draw dropdown menu
pub fn draw_menu_dropdown(
    f: &mut Frame,
    app: &AppState,
    menu_bar_area: Rect,
    menu_idx: usize,
    selected_item: usize,
) {
    let lang = app.language;

    // Calculate x position based on menu index
    let menus = vec![
        t("menu.file", lang),
        t("menu.settings", lang),
        t("menu.help", lang),
    ];

    let mut x_offset = 0u16;
    for i in 0..menu_idx {
        if i < menus.len() {
            x_offset += display_width(menus[i]) as u16 + 4; // +4 for padding and spacing
        }
    }

    // Menu items based on menu index
    let items: Vec<&str> = match menu_idx {
        0 => vec![
            t("menu.file.save_config", lang),
            t("menu.file.load_config", lang),
            "",
            t("menu.file.exit", lang),
        ],
        1 => vec![t("menu.settings.toggle_language", lang)],
        2 => vec![t("menu.help.about", lang)],
        _ => vec![],
    };

    // Calculate dropdown dimensions
    // +6: borders(2) + padding(2) + extra space(2) for CJK characters
    let max_width = items.iter().map(|s| display_width(s)).max().unwrap_or(10) as u16 + 6;
    let height = items.len() as u16 + 2;

    // Position dropdown below menu bar
    let dropdown_area = Rect {
        x: menu_bar_area.x + x_offset,
        y: menu_bar_area.y + 1,
        width: max_width,
        height,
    };

    // Create list items
    let list_items: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            if item.is_empty() {
                ListItem::new("─".repeat(max_width.saturating_sub(2) as usize))
                    .style(Style::default().fg(Color::DarkGray))
            } else if i == selected_item {
                ListItem::new(format!(" {}", item))
                    .style(Style::default().bg(Color::Cyan).fg(Color::Black))
            } else {
                ListItem::new(format!(" {}", item)).style(Style::default().fg(Color::White))
            }
        })
        .collect();

    let list = List::new(list_items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .style(Style::default().bg(Color::Black)),
    );

    f.render_widget(list, dropdown_area);
}
