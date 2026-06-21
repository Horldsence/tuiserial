//! UI area rectangles and mouse interaction handling
//!
//! This module manages UI area definitions and mouse click detection for interactive elements.

use std::cell::RefCell;

use ratatui::layout::Rect;
use tuiserial_core::FocusedField;

/// UI area rectangles for mouse interaction
#[derive(Debug, Clone, Copy, Default)]
pub struct UiAreas {
    pub menu_bar: Rect,
    pub port: Rect,
    pub baud_rate: Rect,
    pub data_bits: Rect,
    pub parity: Rect,
    pub stop_bits: Rect,
    pub flow_control: Rect,
    pub status_panel: Rect,
    pub log_area: Rect,
    pub tx_area: Rect,
    pub control_area: Rect,
    pub notification_area: Rect,
    pub shortcuts_hint: Rect,
    pub tab_bar: Rect,
    pub plugin_modal: Rect,
    /// Native terminal cursor position (set during rendering, used after draw)
    pub cursor_x: u16,
    pub cursor_y: u16,
    pub show_cursor: bool,
}

// Thread-local storage for UI areas (single-threaded terminal application)
thread_local! {
    static UI_AREAS: RefCell<UiAreas> = RefCell::new(UiAreas::default());
}

/// Get a copy of the current UI areas
pub fn get_ui_areas() -> UiAreas {
    UI_AREAS.with(|a| *a.borrow())
}

/// Update UI areas (called during rendering)
#[allow(dead_code)]
pub fn update_ui_areas(areas: UiAreas) {
    UI_AREAS.with(|a| *a.borrow_mut() = areas);
}

/// Update specific UI area field
pub fn update_area(field: UiAreaField, rect: Rect) {
    UI_AREAS.with(|a| {
        let mut areas = a.borrow_mut();
        match field {
            UiAreaField::MenuBar => areas.menu_bar = rect,
            UiAreaField::Port => areas.port = rect,
            UiAreaField::BaudRate => areas.baud_rate = rect,
            UiAreaField::DataBits => areas.data_bits = rect,
            UiAreaField::Parity => areas.parity = rect,
            UiAreaField::StopBits => areas.stop_bits = rect,
            UiAreaField::FlowControl => areas.flow_control = rect,
            UiAreaField::StatusPanel => areas.status_panel = rect,
            UiAreaField::LogArea => areas.log_area = rect,
            UiAreaField::TxArea => areas.tx_area = rect,
            UiAreaField::ControlArea => areas.control_area = rect,
            UiAreaField::NotificationArea => areas.notification_area = rect,
            UiAreaField::ShortcutsHint => areas.shortcuts_hint = rect,
            UiAreaField::TabBar => areas.tab_bar = rect,
            UiAreaField::PluginModal => areas.plugin_modal = rect,
        }
    });
}

/// UI area field identifiers
pub enum UiAreaField {
    MenuBar,
    Port,
    BaudRate,
    DataBits,
    Parity,
    StopBits,
    FlowControl,
    StatusPanel,
    LogArea,
    TxArea,
    ControlArea,
    NotificationArea,
    ShortcutsHint,
    #[allow(dead_code)]
    TabBar,
    PluginModal,
}

/// Update terminal cursor position and visibility (called during rendering)
pub fn update_cursor_state(x: u16, y: u16, show: bool) {
    UI_AREAS.with(|a| {
        let mut areas = a.borrow_mut();
        areas.cursor_x = x;
        areas.cursor_y = y;
        areas.show_cursor = show;
    });
}

/// Check if a point is inside a rectangle
pub fn is_inside(rect: Rect, x: u16, y: u16) -> bool {
    x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height
}

/// Determine which field was clicked based on coordinates
pub fn get_clicked_field(x: u16, y: u16) -> Option<FocusedField> {
    let areas = get_ui_areas();

    if is_inside(areas.port, x, y) {
        Some(FocusedField::Port)
    } else if is_inside(areas.baud_rate, x, y) {
        Some(FocusedField::BaudRate)
    } else if is_inside(areas.data_bits, x, y) {
        Some(FocusedField::DataBits)
    } else if is_inside(areas.parity, x, y) {
        Some(FocusedField::Parity)
    } else if is_inside(areas.stop_bits, x, y) {
        Some(FocusedField::StopBits)
    } else if is_inside(areas.flow_control, x, y) {
        Some(FocusedField::FlowControl)
    } else if is_inside(areas.log_area, x, y) {
        Some(FocusedField::LogArea)
    } else if is_inside(areas.tx_area, x, y) {
        Some(FocusedField::TxInput)
    } else {
        None
    }
}

/// Get which menu was clicked in the menu bar
pub fn get_clicked_menu(x: u16, y: u16) -> Option<usize> {
    let areas = get_ui_areas();

    if !is_inside(areas.menu_bar, x, y) {
        return None;
    }

    let relative_x = x.saturating_sub(areas.menu_bar.x);

    if relative_x < 7 {
        Some(0) // File
    } else if (8..17).contains(&relative_x) {
        Some(1) // Session
    } else if (18..25).contains(&relative_x) {
        Some(2) // View
    } else if (26..36).contains(&relative_x) {
        Some(3) // Settings
    } else if (37..43).contains(&relative_x) {
        Some(4) // Help
    } else {
        None
    }
}

/// Check if shortcuts hint area was clicked
pub fn is_shortcuts_hint_clicked(x: u16, y: u16) -> bool {
    let areas = get_ui_areas();
    is_inside(areas.shortcuts_hint, x, y)
}

/// Check if tab bar was clicked and return tab index
pub fn get_clicked_tab(x: u16, y: u16, tab_count: usize) -> Option<usize> {
    let areas = get_ui_areas();

    if !is_inside(areas.tab_bar, x, y) || tab_count == 0 {
        return None;
    }

    let width = areas.tab_bar.width;
    if width == 0 {
        return None;
    }

    let relative_x = x.saturating_sub(areas.tab_bar.x);
    let mut tab_index = (relative_x as usize * tab_count) / width as usize;

    if tab_index >= tab_count {
        tab_index = tab_count - 1;
    }

    Some(tab_index)
}
