//! Menu bar and dropdown menu rendering
//!
//! This module handles the rendering of the menu bar and its dropdown menus.
//! All menu structure is now centralized in tuiserial_core::menu_def.

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use tuiserial_core::{i18n::t, menu_def::MENU_BAR, AppState, MenuState};

use crate::utils::display_width;

/// Draw the menu bar at the top
pub fn draw_menu_bar(f: &mut Frame, app: &AppState, area: Rect) {
    let lang = app.language;
    let mut spans = Vec::new();

    match app.menu_state {
        MenuState::None => {
            // Normal state - show all menus
            for i in 0..MENU_BAR.menu_count() {
                if i > 0 {
                    spans.push(Span::raw("  "));
                }
                if let Some(label_key) = MENU_BAR.get_menu_label_key(i) {
                    let label = t(label_key, lang);
                    spans.push(Span::styled(
                        format!(" {} ", label),
                        Style::default().fg(Color::White).bg(Color::DarkGray),
                    ));
                }
            }
        }
        MenuState::MenuBar(selected) => {
            // Menu bar focused - highlight selected
            for i in 0..MENU_BAR.menu_count() {
                if i > 0 {
                    spans.push(Span::raw("  "));
                }
                if let Some(label_key) = MENU_BAR.get_menu_label_key(i) {
                    let label = t(label_key, lang);
                    let style = if i == selected {
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::Cyan)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White).bg(Color::DarkGray)
                    };
                    spans.push(Span::styled(format!(" {} ", label), style));
                }
            }
        }
        MenuState::Dropdown(menu_idx, _) => {
            // Dropdown open - highlight parent menu
            for i in 0..MENU_BAR.menu_count() {
                if i > 0 {
                    spans.push(Span::raw("  "));
                }
                if let Some(label_key) = MENU_BAR.get_menu_label_key(i) {
                    let label = t(label_key, lang);
                    let style = if i == menu_idx {
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::Cyan)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White).bg(Color::DarkGray)
                    };
                    spans.push(Span::styled(format!(" {} ", label), style));
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

    // Get menu
    let menu = match MENU_BAR.get_menu(menu_idx) {
        Some(m) => m,
        None => return,
    };

    // Calculate x position based on menu index
    let x_offset = tuiserial_core::menu_def::calculate_menu_x_offset(menu_idx, lang);

    // Build menu items with translations
    let items: Vec<(&str, bool)> = menu
        .items
        .iter()
        .map(|action| {
            let label = if action.is_separator() {
                ""
            } else {
                t(action.label_key(), lang)
            };
            (label, action.is_separator())
        })
        .collect();

    // Calculate dropdown dimensions
    let max_width = items
        .iter()
        .map(|(label, _)| display_width(label))
        .max()
        .unwrap_or(10) as u16
        + 6; // +6 for borders and padding
    let height = items.len() as u16 + 2; // +2 for borders

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
        .map(|(i, (label, is_sep))| {
            if *is_sep {
                // Separator line
                ListItem::new("─".repeat(max_width.saturating_sub(2) as usize))
                    .style(Style::default().fg(Color::DarkGray))
            } else if i == selected_item {
                // Selected item
                ListItem::new(format!(" {}", label))
                    .style(Style::default().bg(Color::Cyan).fg(Color::Black))
            } else {
                // Normal item
                ListItem::new(format!(" {}", label)).style(Style::default().fg(Color::White))
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

/// Find which menu was clicked based on mouse position
pub fn find_clicked_menu(
    x: u16,
    y: u16,
    menu_bar_area: Rect,
    lang: tuiserial_core::Language,
) -> Option<usize> {
    // Check if click is within menu bar
    if y != menu_bar_area.y || x < menu_bar_area.x || x >= menu_bar_area.x + menu_bar_area.width {
        return None;
    }

    // Calculate relative x position
    let relative_x = x.saturating_sub(menu_bar_area.x);

    // Use centralized menu position calculation
    tuiserial_core::menu_def::find_clicked_menu(relative_x, lang)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tuiserial_core::Language;

    #[test]
    fn test_menu_click_detection() {
        let area = Rect {
            x: 0,
            y: 0,
            width: 80,
            height: 1,
        };

        // Click on first menu (File)
        let result = find_clicked_menu(2, 0, area, Language::English);
        assert_eq!(result, Some(0));

        // Click outside menu bar
        let result = find_clicked_menu(2, 1, area, Language::English);
        assert_eq!(result, None);
    }
}
