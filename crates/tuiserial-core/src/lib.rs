//! Core data models and types for tuiserial
//!
//! This crate provides the fundamental data structures, enums, and state management
//! types used throughout the tuiserial application.
//!
//! ## Architecture
//!
//! The core is organized into modular components:
//! - `types`: Basic type definitions and enums (DisplayMode, TxMode, Parity, etc.)
//! - `notification`: Notification system for user messages
//! - `log`: Log entries and message log for serial communication
//! - `config`: Serial port configuration
//! - `state`: Main application state management
//! - `i18n`: Internationalization support

// Module declarations
pub mod config;
pub mod i18n;
pub mod log;
pub mod menu_def;
pub mod notification;
pub mod state;
pub mod types;

// Re-exports for convenience
pub use config::SerialConfig;
pub use log::{LogDirection, LogEntry, MessageLog, MAX_LOG_LINES};
pub use menu_def::{MenuAction, MenuBar, MENU_BAR};
pub use notification::{Notification, NotificationLevel};
pub use state::AppState;
pub use types::{
    AppendMode, DisplayMode, FlowControl, FocusedField, Language, MenuState, Parity, TxMode,
};

// Re-export commonly used dependencies
pub use chrono;
pub use ratatui;
pub use serde;
pub use serde_json;
