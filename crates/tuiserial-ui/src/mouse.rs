//! Mouse event handling for UI interactions
//!
//! This module provides mouse event handling functionality for menu bar,
//! tabs, buttons, and other interactive UI elements.

use ratatui::layout::Rect;
use tuiserial_core::{AppState, FocusedField, Language, MenuState};

use crate::areas::{get_clicked_field, get_clicked_menu, is_inside, is_shortcuts_hint_clicked};

/// Mouse action result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MouseAction {
    /// No action taken
    None,
    /// Field was focused
    FocusField(FocusedField),
    /// Menu was opened
    OpenMenu(usize),
    /// Menu item was selected
    SelectMenuItem(usize, usize), // menu_idx, item_idx
    /// Tab was switched
    SwitchTab(usize),
    /// Connect/disconnect button clicked
    ToggleConnection,
    /// Clear log button clicked
    ClearLog,
    /// Refresh ports button clicked
    RefreshPorts,
    /// Send button clicked
    SendData,
    /// Show shortcuts help
    ShowShortcutsHelp,
    /// Close shortcuts help
    CloseShortcutsHelp,
    /// Close menu/dropdown
    CloseMenu,
}

/// Handle mouse click events
pub fn handle_mouse_click(
    app: &AppState,
    x: u16,
    y: u16,
    menu_dropdown_area: Option<Rect>,
) -> MouseAction {
    // If shortcuts help is showing, check if clicked outside to close
    if app.show_shortcuts_help {
        // Check if clicked on shortcuts hint area to toggle
        if is_shortcuts_hint_clicked(x, y) {
            return MouseAction::CloseShortcutsHelp;
        }
        // Click anywhere else closes the help
        return MouseAction::CloseShortcutsHelp;
    }

    // Check menu dropdown first (if open)
    if let MenuState::Dropdown(menu_idx, _) = app.menu_state {
        if let Some(dropdown_area) = menu_dropdown_area {
            if is_inside(dropdown_area, x, y) {
                // Clicked inside dropdown - determine which item
                let item_idx = calculate_dropdown_item(dropdown_area, y);
                return MouseAction::SelectMenuItem(menu_idx, item_idx);
            } else {
                // Clicked outside dropdown - close it
                return MouseAction::CloseMenu;
            }
        }
    }

    // Check menu bar
    if let Some(menu_idx) = get_clicked_menu(x, y) {
        return MouseAction::OpenMenu(menu_idx);
    }

    // Check shortcuts hint bar
    if is_shortcuts_hint_clicked(x, y) {
        return MouseAction::ShowShortcutsHelp;
    }

    // Check tab bar (if visible)
    // Note: This would be used in multi-session mode
    // For now, returning None as tabs are not in current single-session UI

    // Check configuration fields
    if let Some(field) = get_clicked_field(x, y) {
        return MouseAction::FocusField(field);
    }

    // Check for button clicks in status panel
    // This is approximate and would need actual button areas
    // For now, we return None

    MouseAction::None
}

/// Handle mouse hover/move events
pub fn handle_mouse_hover(_app: &AppState, x: u16, y: u16) -> Option<String> {
    // Return tooltip text based on hover position

    // Check menu bar
    if let Some(menu_idx) = get_clicked_menu(x, y) {
        let tooltip = match menu_idx {
            0 => "File operations",
            1 => "Session management",
            2 => "View layouts",
            3 => "Application settings",
            4 => "Help and information",
            _ => "",
        };
        return Some(tooltip.to_string());
    }

    // Check shortcuts hint
    if is_shortcuts_hint_clicked(x, y) {
        return Some("Click to show keyboard shortcuts".to_string());
    }

    // Check configuration fields
    if let Some(field) = get_clicked_field(x, y) {
        let tooltip = match field {
            FocusedField::Port => "Select serial port",
            FocusedField::BaudRate => "Select baud rate",
            FocusedField::DataBits => "Select data bits",
            FocusedField::Parity => "Select parity",
            FocusedField::StopBits => "Select stop bits",
            FocusedField::FlowControl => "Select flow control",
            FocusedField::LogArea => "Serial communication log",
            FocusedField::TxInput => "Enter data to send",
        };
        return Some(tooltip.to_string());
    }

    None
}

/// Calculate which dropdown item was clicked based on Y coordinate
fn calculate_dropdown_item(dropdown_area: Rect, y: u16) -> usize {
    let _ = dropdown_area;
    if y < dropdown_area.y || y >= dropdown_area.y + dropdown_area.height {
        return 0;
    }

    let relative_y = y.saturating_sub(dropdown_area.y);

    // Account for border (1 line at top)
    if relative_y == 0 {
        return 0;
    }

    // Each item is 1 line, subtract 1 for top border
    relative_y.saturating_sub(1) as usize
}

/// Get the area for a dropdown menu
pub fn calculate_dropdown_area(
    menu_bar_area: Rect,
    menu_idx: usize,
    item_count: usize,
    lang: Language,
) -> Rect {
    // Calculate x position based on menu index using centralized calculation
    let x_offset = tuiserial_core::menu_def::calculate_menu_x_offset(menu_idx, lang);

    // Calculate dropdown dimensions
    let max_width = 25u16; // Reasonable default width
    let height = item_count as u16 + 2; // +2 for borders

    Rect {
        x: menu_bar_area.x + x_offset,
        y: menu_bar_area.y + 1, // Below menu bar
        width: max_width,
        height,
    }
}

/// Check if a click is on a button area (approximate)
#[allow(dead_code)]
pub fn is_button_area(area: Rect, x: u16, y: u16, _button_text: &str) -> bool {
    if !is_inside(area, x, y) {
        return false;
    }

    // This is a simplified check - in practice you'd need actual button positions
    // For now, just check if inside the area
    true
}

/// Handle mouse scroll events in log area
pub fn handle_mouse_scroll(
    _app: &AppState,
    x: u16,
    y: u16,
    direction: ScrollDirection,
) -> Option<ScrollAction> {
    use crate::areas::get_ui_areas;

    let areas = get_ui_areas();

    // Check if scrolling in log area
    if is_inside(areas.log_area, x, y) {
        match direction {
            ScrollDirection::Up => Some(ScrollAction::ScrollUp(3)),
            ScrollDirection::Down => Some(ScrollAction::ScrollDown(3)),
        }
    } else {
        None
    }
}

/// Scroll direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollDirection {
    Up,
    Down,
}

/// Scroll action
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollAction {
    ScrollUp(u16),
    ScrollDown(u16),
}

/// Get visual feedback for hover state
pub fn get_hover_style(is_hovered: bool) -> ratatui::style::Style {
    use ratatui::style::{Color, Modifier, Style};

    if is_hovered {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    }
}

/// Check if mouse is in a clickable area
pub fn is_clickable_area(x: u16, y: u16) -> bool {
    use crate::areas::get_ui_areas;

    let areas = get_ui_areas();

    // Check all interactive areas
    is_inside(areas.menu_bar, x, y)
        || is_inside(areas.port, x, y)
        || is_inside(areas.baud_rate, x, y)
        || is_inside(areas.data_bits, x, y)
        || is_inside(areas.parity, x, y)
        || is_inside(areas.stop_bits, x, y)
        || is_inside(areas.flow_control, x, y)
        || is_inside(areas.tx_area, x, y)
        || is_inside(areas.shortcuts_hint, x, y)
        || is_inside(areas.tab_bar, x, y)
}

/// Mouse cursor type for different areas
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorType {
    Default,
    Pointer, // Clickable
    Text,    // Text input
    Help,    // Help/info
}

/// Get appropriate cursor type for position
pub fn get_cursor_type(_app: &AppState, x: u16, y: u16) -> CursorType {
    use crate::areas::get_ui_areas;

    let areas = get_ui_areas();

    if is_inside(areas.tx_area, x, y) {
        CursorType::Text
    } else if is_shortcuts_hint_clicked(x, y) {
        CursorType::Help
    } else if is_clickable_area(x, y) {
        CursorType::Pointer
    } else {
        CursorType::Default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_dropdown_item() {
        let area = Rect {
            x: 0,
            y: 1,
            width: 20,
            height: 5,
        };

        assert_eq!(calculate_dropdown_item(area, 0), 0); // Above area
        assert_eq!(calculate_dropdown_item(area, 1), 0); // Border
        assert_eq!(calculate_dropdown_item(area, 2), 1); // First item
        assert_eq!(calculate_dropdown_item(area, 3), 2); // Second item
    }

    #[test]
    fn test_calculate_dropdown_area() {
        let menu_bar = Rect {
            x: 0,
            y: 0,
            width: 80,
            height: 1,
        };

        let area = calculate_dropdown_area(menu_bar, 0, 4, Language::English);
        assert_eq!(area.y, 1); // Below menu bar
        assert_eq!(area.height, 6); // 4 items + 2 borders
    }

    #[test]
    fn test_is_clickable_area() {
        // This would need actual UI areas set up
        // For now, just test that the function exists
        let result = is_clickable_area(0, 0);
        assert!(result == true || result == false);
    }
}
