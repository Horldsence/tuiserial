//! UI area rectangles and mouse interaction handling
//!
//! This module manages UI area definitions and mouse click detection for interactive elements.

use ratatui::layout::Rect;
use tuiserial_core::FocusedField;

/// UI area rectangles for mouse interaction
#[derive(Debug, Clone, Copy)]
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
}

impl Default for UiAreas {
    fn default() -> Self {
        Self {
            menu_bar: Rect::default(),
            port: Rect::default(),
            baud_rate: Rect::default(),
            data_bits: Rect::default(),
            parity: Rect::default(),
            stop_bits: Rect::default(),
            flow_control: Rect::default(),
            status_panel: Rect::default(),
            log_area: Rect::default(),
            tx_area: Rect::default(),
            control_area: Rect::default(),
            notification_area: Rect::default(),
        }
    }
}

// Global static for UI areas (thread-local would be better in production)
static mut UI_AREAS: UiAreas = UiAreas {
    menu_bar: Rect {
        x: 0,
        y: 0,
        width: 0,
        height: 0,
    },
    port: Rect {
        x: 0,
        y: 0,
        width: 0,
        height: 0,
    },
    baud_rate: Rect {
        x: 0,
        y: 0,
        width: 0,
        height: 0,
    },
    data_bits: Rect {
        x: 0,
        y: 0,
        width: 0,
        height: 0,
    },
    parity: Rect {
        x: 0,
        y: 0,
        width: 0,
        height: 0,
    },
    stop_bits: Rect {
        x: 0,
        y: 0,
        width: 0,
        height: 0,
    },
    flow_control: Rect {
        x: 0,
        y: 0,
        width: 0,
        height: 0,
    },
    status_panel: Rect {
        x: 0,
        y: 0,
        width: 0,
        height: 0,
    },
    log_area: Rect {
        x: 0,
        y: 0,
        width: 0,
        height: 0,
    },
    tx_area: Rect {
        x: 0,
        y: 0,
        width: 0,
        height: 0,
    },
    control_area: Rect {
        x: 0,
        y: 0,
        width: 0,
        height: 0,
    },
    notification_area: Rect {
        x: 0,
        y: 0,
        width: 0,
        height: 0,
    },
};

/// Get the UI areas for mouse interaction
pub fn get_ui_areas() -> UiAreas {
    unsafe { UI_AREAS }
}

/// Update UI areas (called during rendering)
pub fn update_ui_areas(areas: UiAreas) {
    unsafe {
        UI_AREAS = areas;
    }
}

/// Update specific UI area fields
pub fn update_area(field: UiAreaField, rect: Rect) {
    unsafe {
        match field {
            UiAreaField::MenuBar => UI_AREAS.menu_bar = rect,
            UiAreaField::Port => UI_AREAS.port = rect,
            UiAreaField::BaudRate => UI_AREAS.baud_rate = rect,
            UiAreaField::DataBits => UI_AREAS.data_bits = rect,
            UiAreaField::Parity => UI_AREAS.parity = rect,
            UiAreaField::StopBits => UI_AREAS.stop_bits = rect,
            UiAreaField::FlowControl => UI_AREAS.flow_control = rect,
            UiAreaField::StatusPanel => UI_AREAS.status_panel = rect,
            UiAreaField::LogArea => UI_AREAS.log_area = rect,
            UiAreaField::TxArea => UI_AREAS.tx_area = rect,
            UiAreaField::ControlArea => UI_AREAS.control_area = rect,
            UiAreaField::NotificationArea => UI_AREAS.notification_area = rect,
        }
    }
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
