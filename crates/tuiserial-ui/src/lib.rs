//! Terminal user interface components for tuiserial
//!
//! This crate provides the UI rendering logic using ratatui for displaying
//! serial port configuration, logs, and user interactions with full mouse support.
//!
//! ## Architecture
//!
//! The UI is organized into modular components:
//! - `areas`: UI area definitions and mouse interaction handling
//! - `menu`: Menu bar and dropdown rendering
//! - `config`: Configuration panel with dropdowns for serial settings
//! - `status`: Status panel and statistics display
//! - `log`: Log area showing serial communication data
//! - `tx`: Transmission input area
//! - `notification`: Notification bar for user messages
//! - `utils`: Utility functions for UI rendering

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};
use tuiserial_core::{AppState, MenuState};

// Module declarations
mod areas;
mod config;
mod log;
mod menu;
mod notification;
mod status;
mod tx;
mod utils;

// Re-exports for external use
pub use areas::{get_clicked_field, get_ui_areas, is_inside, UiAreas};
pub use crossterm;
pub use ratatui;

/// Main draw function - renders the entire application UI
///
/// This is the entry point for rendering the UI. It orchestrates the layout
/// and delegates rendering to specialized modules.
pub fn draw(f: &mut Frame, app: &AppState) {
    // Main layout: menu bar, content area, notification bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Menu bar
            Constraint::Min(15),   // Main content
            Constraint::Length(3), // Notification area
        ])
        .split(f.area());

    // Render main content first
    draw_main_content(f, app, chunks[1]);

    // Render notification bar
    notification::draw_notification_bar(f, app, chunks[2]);

    // Render menu bar (without dropdown)
    menu::draw_menu_bar(f, app, chunks[0]);

    // Render dropdown last to ensure it's on top
    if let MenuState::Dropdown(menu_idx, item_idx) = app.menu_state {
        menu::draw_menu_dropdown(f, app, chunks[0], menu_idx, item_idx);
    }

    // Store menu bar and notification area for mouse interaction
    areas::update_area(areas::UiAreaField::MenuBar, chunks[0]);
    areas::update_area(areas::UiAreaField::NotificationArea, chunks[2]);
}

/// Draw the main content area (config panel + log/tx areas)
fn draw_main_content(f: &mut Frame, app: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(42), Constraint::Min(50)])
        .split(area);

    draw_config_panel(f, app, chunks[0]);
    draw_main_area(f, app, chunks[1]);
}

/// Draw the configuration panel on the left
fn draw_config_panel(f: &mut Frame, app: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Port
            Constraint::Length(5), // Baud rate
            Constraint::Length(3), // Data bits
            Constraint::Length(3), // Parity
            Constraint::Length(3), // Stop bits
            Constraint::Length(3), // Flow control
            Constraint::Min(10),   // Status panel
        ])
        .split(area);

    config::draw_port_dropdown(f, app, chunks[0]);
    config::draw_baud_rate_dropdown(f, app, chunks[1]);
    config::draw_data_bits_dropdown(f, app, chunks[2]);
    config::draw_parity_dropdown(f, app, chunks[3]);
    config::draw_stop_bits_dropdown(f, app, chunks[4]);
    config::draw_flow_control_dropdown(f, app, chunks[5]);
    status::draw_status_panel(f, app, chunks[6]);
}

/// Draw the main area on the right (log + tx + control)
fn draw_main_area(f: &mut Frame, app: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),   // Log area
            Constraint::Length(7), // TX area
            Constraint::Length(3), // Control/stats area
        ])
        .split(area);

    log::draw_log_area(f, app, chunks[0]);
    tx::draw_tx_area(f, app, chunks[1]);
    status::draw_control_area(f, app, chunks[2]);
}
