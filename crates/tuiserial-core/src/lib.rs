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
pub mod error;
pub mod error_log;
pub mod log;
pub mod menu_def;
pub mod notification;
pub mod state;
pub mod types;

// Re-exports for convenience
pub use config::SerialConfig;
pub use error::{
    AppError, ConfigErrorKind, CoreError, ErrorContext, ErrorSeverity, PluginErrorKind,
    RecoveryStrategy, SerialErrorKind,
};
pub use error_log::{ErrorLog, ErrorLogEntry};
pub use log::{LogDirection, LogEntry, MAX_LOG_LINES, MessageLog};
pub use menu_def::{MENU_BAR, MenuAction, MenuBar};
pub use notification::{Notification, NotificationLevel};
pub use state::{AppState, PluginLoadStatus, PluginMetadataSimple};
pub use types::{
    AppendMode, DisplayMode, FlowControl, FocusedField, Language, MenuState, Parity,
    PluginLoadState, PluginModalMode, RegistryEntry, TxMode, convert_tx_input,
};

// Utility functions

/// Calculate display width of a string (handles CJK characters)
///
/// ASCII characters occupy 1 terminal cell; CJK and other wide characters occupy 2.
pub fn display_width(s: &str) -> usize {
    s.chars().map(|c| if c.is_ascii() { 1 } else { 2 }).sum()
}

// Re-export commonly used dependencies
pub use chrono;
pub use ratatui;
pub use serde;
pub use serde_json;

// i18n support
use rust_i18n::i18n;
// Initialize i18n translations at compile time
i18n!("../../locales", fallback = "en");
