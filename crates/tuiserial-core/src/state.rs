//! Application state management
//!
//! This module defines the main application state structure that holds all
//! runtime data, UI state, and configuration for the tuiserial application.

use ratatui::widgets::ListState;
use serde_json;
use std::collections::VecDeque;

use crate::config::SerialConfig;
use crate::error::{AppError, CoreError, ErrorSeverity};
use crate::error_log::ErrorLog;
use crate::log::MessageLog;
use crate::notification::Notification;
use crate::types::{
    AppendMode, DisplayMode, FlowControl, FocusedField, Language, MenuState, Parity,
    PluginLoadState, PluginModalMode, RegistryEntry, TxMode,
};

/// Main application state
pub struct AppState {
    // Serial configuration
    pub config: SerialConfig,
    pub message_log: MessageLog,
    pub display_mode: DisplayMode,
    pub is_connected: bool,
    pub config_locked: bool,

    // Available ports
    pub ports: Vec<String>,

    // Scroll state
    pub scroll_offset: u16,
    pub auto_scroll: bool,

    // UI State for dropdowns
    pub port_list_state: ListState,
    pub baud_rate_options: Vec<u32>,
    pub baud_rate_state: ListState,
    pub parity_options: Vec<Parity>,
    pub parity_state: ListState,
    pub flow_control_options: Vec<FlowControl>,
    pub flow_control_state: ListState,
    pub data_bits_options: Vec<u8>,
    pub data_bits_state: ListState,
    pub stop_bits_options: Vec<u8>,
    pub stop_bits_state: ListState,

    // TX Input state
    pub tx_input: String,
    pub tx_mode: TxMode,
    pub tx_append_mode: AppendMode,
    pub tx_cursor: usize,
    pub append_mode_options: Vec<AppendMode>,
    pub append_mode_state: ListState,

    // UI Focus
    pub focused_field: FocusedField,

    // Notification system
    pub notifications: VecDeque<Notification>,

    // Debug info
    pub debug_mode: bool,
    pub last_mouse_event: String,

    // Menu and Language
    pub menu_state: MenuState,
    pub language: Language,

    // Help overlay
    pub show_shortcuts_help: bool,

    // Plugin management modal
    pub show_plugin_modal: bool,
    /// Which view the modal is showing (Local or Registry)
    pub plugin_modal_mode: PluginModalMode,
    /// Per-plugin load status for display in the modal
    pub plugin_statuses: Vec<PluginLoadStatus>,
    /// Scroll offset in the plugin modal list
    pub plugin_modal_scroll: usize,

    // Plugin registry (for Registry mode)
    /// Cached registry entries fetched from the remote
    pub registry_entries: Vec<RegistryEntry>,
    /// Search query typed by the user
    pub registry_search_query: String,
    /// Scroll offset in the registry result list
    pub registry_scroll: usize,
    /// True while the registry is being fetched
    pub registry_loading: bool,

    // Plugin status bar info
    pub plugin_loaded_count: usize,
    pub plugin_error_count: usize,
    pub plugin_total_count: usize,

    // Unified error log
    pub error_log: ErrorLog,
}

/// Lightweight per-plugin status for the plugin modal UI.
#[derive(Debug, Clone)]
pub struct PluginLoadStatus {
    pub name: String,
    pub state: PluginLoadState,
    pub has_rx_hook: bool,
    pub has_tx_hook: bool,
    pub has_connect_hook: bool,
    pub has_disconnect_hook: bool,
    pub error_message: Option<String>,
    pub metadata: Option<PluginMetadataSimple>,
}

/// Simplified plugin metadata for UI display.
#[derive(Debug, Clone)]
pub struct PluginMetadataSimple {
    pub version: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
}

impl Default for AppState {
    fn default() -> Self {
        let baud_rate_options = vec![
            300, 1200, 2400, 4800, 9600, 19200, 38400, 57600, 115200, 230400,
        ];
        let parity_options = vec![Parity::None, Parity::Even, Parity::Odd];
        let flow_control_options = vec![
            FlowControl::None,
            FlowControl::Hardware,
            FlowControl::Software,
        ];
        let data_bits_options = vec![5, 6, 7, 8];
        let stop_bits_options = vec![1, 2];
        let append_mode_options = AppendMode::all();

        Self {
            config: SerialConfig::default(),
            message_log: MessageLog::new(),
            display_mode: DisplayMode::Hex,
            is_connected: false,
            config_locked: false,
            ports: Vec::new(),
            scroll_offset: 0,
            auto_scroll: true,
            port_list_state: ListState::default().with_selected(Some(0)),
            baud_rate_state: ListState::default().with_selected(Some(4)), // 9600
            parity_state: ListState::default().with_selected(Some(0)),    // None
            flow_control_state: ListState::default().with_selected(Some(0)), // None
            data_bits_state: ListState::default().with_selected(Some(3)), // 8
            stop_bits_state: ListState::default().with_selected(Some(0)), // 1
            baud_rate_options,
            parity_options,
            flow_control_options,
            data_bits_options,
            stop_bits_options,
            tx_input: String::new(),
            tx_mode: TxMode::Ascii,
            tx_append_mode: AppendMode::None,
            tx_cursor: 0,
            append_mode_options,
            append_mode_state: ListState::default().with_selected(Some(0)),
            focused_field: FocusedField::Port,
            notifications: VecDeque::new(),
            debug_mode: false,
            last_mouse_event: String::new(),
            menu_state: MenuState::None,
            language: Language::English,
            show_shortcuts_help: false,
            show_plugin_modal: false,
            plugin_modal_mode: PluginModalMode::Local,
            plugin_statuses: Vec::new(),
            plugin_modal_scroll: 0,
            registry_entries: Vec::new(),
            registry_search_query: String::new(),
            registry_scroll: 0,
            registry_loading: false,
            plugin_loaded_count: 0,
            plugin_error_count: 0,
            plugin_total_count: 0,
            error_log: ErrorLog::new(),
        }
    }
}

impl AppState {
    /// Create a new application state with default values
    pub fn new() -> Self {
        Self::default()
    }

    // Configuration management

    /// Lock configuration (called when connecting)
    pub fn lock_config(&mut self) {
        self.config_locked = true;
    }

    /// Unlock configuration (called when disconnecting)
    pub fn unlock_config(&mut self) {
        self.config_locked = false;
    }

    /// Check if configuration can be modified
    pub fn can_modify_config(&self) -> bool {
        !self.config_locked
    }

    // Notification management

    /// Add a notification to the queue
    pub fn add_notification(&mut self, notification: Notification) {
        self.notifications.push_back(notification);
    }

    /// Add an info notification
    pub fn add_info(&mut self, msg: impl Into<String>) {
        self.add_notification(Notification::info(msg.into()));
    }

    /// Add a warning notification
    pub fn add_warning(&mut self, msg: impl Into<String>) {
        self.add_notification(Notification::warning(msg.into()));
    }

    /// Add an error notification
    pub fn add_error(&mut self, msg: impl Into<String>) {
        self.add_notification(Notification::error(msg.into()));
    }

    /// Add a success notification
    pub fn add_success(&mut self, msg: impl Into<String>) {
        self.add_notification(Notification::success(msg.into()));
    }

    /// Remove expired notifications
    pub fn update_notifications(&mut self) {
        while let Some(front) = self.notifications.front() {
            if front.is_expired() {
                self.notifications.pop_front();
            } else {
                break;
            }
        }
    }

    /// Record an `AppError`: push it into the persistent error log and
    /// also add a user-facing notification at the appropriate level.
    pub fn record_error(&mut self, error: AppError) {
        let severity = error.severity();
        let msg = error.to_user_message();
        self.error_log.push(error);

        match severity {
            ErrorSeverity::Info => {
                log::info!("{msg}");
                self.add_info(msg);
            }
            ErrorSeverity::Warning => {
                log::warn!("{msg}");
                self.add_warning(msg);
            }
            ErrorSeverity::Error | ErrorSeverity::Critical => {
                log::error!("{msg}");
                self.add_error(msg);
            }
            ErrorSeverity::Fatal => {
                log::error!("FATAL: {msg}");
                self.add_error(format!("FATAL: {msg}"));
            }
        }
    }

    /// Return a compact error summary string for the status bar,
    /// e.g. `"E:3"` or `"C:1 E:2"`.
    pub fn error_summary(&self) -> String {
        self.error_log.summary()
    }

    // Baud rate management

    /// Select next baud rate
    pub fn next_baud_rate(&mut self) -> bool {
        if !self.can_modify_config() {
            return false;
        }
        if let Some(selected) = self.baud_rate_state.selected() {
            let next = (selected + 1) % self.baud_rate_options.len();
            self.baud_rate_state.select(Some(next));
            self.config.baud_rate = self.baud_rate_options[next];
            true
        } else {
            false
        }
    }

    /// Select previous baud rate
    pub fn prev_baud_rate(&mut self) -> bool {
        if !self.can_modify_config() {
            return false;
        }
        if let Some(selected) = self.baud_rate_state.selected() {
            let next = if selected == 0 {
                self.baud_rate_options.len() - 1
            } else {
                selected - 1
            };
            self.baud_rate_state.select(Some(next));
            self.config.baud_rate = self.baud_rate_options[next];
            true
        } else {
            false
        }
    }

    // Parity management

    /// Toggle parity setting
    pub fn toggle_parity(&mut self) -> bool {
        if !self.can_modify_config() {
            return false;
        }
        if let Some(selected) = self.parity_state.selected() {
            let next = (selected + 1) % self.parity_options.len();
            self.parity_state.select(Some(next));
            self.config.parity = self.parity_options[next];
            true
        } else {
            false
        }
    }

    // Flow control management

    /// Toggle flow control setting
    pub fn toggle_flow_control(&mut self) -> bool {
        if !self.can_modify_config() {
            return false;
        }
        if let Some(selected) = self.flow_control_state.selected() {
            let next = (selected + 1) % self.flow_control_options.len();
            self.flow_control_state.select(Some(next));
            self.config.flow_control = self.flow_control_options[next];
            true
        } else {
            false
        }
    }

    // Data bits management

    /// Select next data bits setting
    pub fn next_data_bits(&mut self) -> bool {
        if !self.can_modify_config() {
            return false;
        }
        if let Some(selected) = self.data_bits_state.selected() {
            let next = (selected + 1) % self.data_bits_options.len();
            self.data_bits_state.select(Some(next));
            self.config.data_bits = self.data_bits_options[next];
            true
        } else {
            false
        }
    }

    // Stop bits management

    /// Select next stop bits setting
    pub fn next_stop_bits(&mut self) -> bool {
        if !self.can_modify_config() {
            return false;
        }
        if let Some(selected) = self.stop_bits_state.selected() {
            let next = (selected + 1) % self.stop_bits_options.len();
            self.stop_bits_state.select(Some(next));
            self.config.stop_bits = self.stop_bits_options[next];
            true
        } else {
            false
        }
    }

    // Port management

    /// Select port (with validation)
    pub fn select_port(&mut self, index: usize) -> bool {
        if !self.can_modify_config() {
            return false;
        }
        if index < self.ports.len() {
            self.port_list_state.select(Some(index));
            self.config.port = self.ports[index].clone();
            true
        } else {
            false
        }
    }

    // TX mode management

    /// Toggle transmission mode, converting existing input when switching
    pub fn toggle_tx_mode(&mut self) {
        self.tx_input = crate::types::convert_tx_input(&self.tx_input, self.tx_mode);
        self.tx_mode = match self.tx_mode {
            TxMode::Hex => TxMode::Ascii,
            TxMode::Ascii => TxMode::Hex,
        };
        self.tx_cursor = self.tx_input.chars().count();
    }

    /// Cycle to next append mode
    pub fn next_append_mode(&mut self) {
        if let Some(selected) = self.append_mode_state.selected() {
            let next = (selected + 1) % self.append_mode_options.len();
            self.append_mode_state.select(Some(next));
            self.tx_append_mode = self.append_mode_options[next];
        }
    }

    /// Cycle to previous append mode
    pub fn prev_append_mode(&mut self) {
        if let Some(selected) = self.append_mode_state.selected() {
            let next = if selected == 0 {
                self.append_mode_options.len() - 1
            } else {
                selected - 1
            };
            self.append_mode_state.select(Some(next));
            self.tx_append_mode = self.append_mode_options[next];
        }
    }

    // Display mode management

    /// Toggle display mode
    pub fn toggle_display_mode(&mut self) {
        self.display_mode = match self.display_mode {
            DisplayMode::Hex => DisplayMode::Text,
            DisplayMode::Text => DisplayMode::Hex,
        };
    }

    // Focus management

    /// Focus next field
    pub fn focus_next_field(&mut self) {
        self.focused_field = match self.focused_field {
            FocusedField::Port => FocusedField::BaudRate,
            FocusedField::BaudRate => FocusedField::DataBits,
            FocusedField::DataBits => FocusedField::Parity,
            FocusedField::Parity => FocusedField::StopBits,
            FocusedField::StopBits => FocusedField::FlowControl,
            FocusedField::FlowControl => FocusedField::LogArea,
            FocusedField::LogArea => FocusedField::TxInput,
            FocusedField::TxInput => FocusedField::Port,
        };
    }

    /// Focus previous field
    pub fn focus_prev_field(&mut self) {
        self.focused_field = match self.focused_field {
            FocusedField::Port => FocusedField::TxInput,
            FocusedField::BaudRate => FocusedField::Port,
            FocusedField::DataBits => FocusedField::BaudRate,
            FocusedField::Parity => FocusedField::DataBits,
            FocusedField::StopBits => FocusedField::Parity,
            FocusedField::FlowControl => FocusedField::StopBits,
            FocusedField::LogArea => FocusedField::FlowControl,
            FocusedField::TxInput => FocusedField::LogArea,
        };
    }

    // Configuration persistence

    /// Save configuration to file
    pub fn save_config(&self) -> Result<(), CoreError> {
        let config_dir = dirs::config_dir().ok_or(CoreError::ConfigDirNotFound)?;
        let app_config_dir = config_dir.join("tuiserial");
        std::fs::create_dir_all(&app_config_dir)?;

        let config_path = app_config_dir.join("config.json");
        let json = serde_json::to_string_pretty(&self.config)?;

        std::fs::write(&config_path, json)?;

        Ok(())
    }

    /// Load configuration from file, return default if not found or error
    pub fn load_config(&mut self) {
        if let Some(config_dir) = dirs::config_dir() {
            let config_path = config_dir.join("tuiserial").join("config.json");
            if let Ok(json) = std::fs::read_to_string(&config_path)
                && let Ok(config) = serde_json::from_str::<SerialConfig>(&json)
            {
                // Update UI states to match loaded config
                if let Some(idx) = self
                    .baud_rate_options
                    .iter()
                    .position(|&b| b == config.baud_rate)
                {
                    self.baud_rate_state.select(Some(idx));
                }
                if let Some(idx) = self.parity_options.iter().position(|&p| p == config.parity) {
                    self.parity_state.select(Some(idx));
                }
                if let Some(idx) = self
                    .flow_control_options
                    .iter()
                    .position(|&f| f == config.flow_control)
                {
                    self.flow_control_state.select(Some(idx));
                }
                if let Some(idx) = self
                    .data_bits_options
                    .iter()
                    .position(|&d| d == config.data_bits)
                {
                    self.data_bits_state.select(Some(idx));
                }
                if let Some(idx) = self
                    .stop_bits_options
                    .iter()
                    .position(|&s| s == config.stop_bits)
                {
                    self.stop_bits_state.select(Some(idx));
                }
                // Move config assignment to end after all borrows
                self.config = config;
            }
        }
    }

    // Language management

    /// Toggle language
    pub fn toggle_language(&mut self) {
        self.language = match self.language {
            Language::English => Language::Chinese,
            Language::Chinese => Language::English,
        };
        rust_i18n::set_locale(self.language.code());
    }

    /// Toggle shortcuts help overlay
    pub fn toggle_shortcuts_help(&mut self) {
        self.show_shortcuts_help = !self.show_shortcuts_help;
    }

    /// Show shortcuts help
    pub fn show_shortcuts_help(&mut self) {
        self.show_shortcuts_help = true;
    }

    /// Hide shortcuts help
    pub fn hide_shortcuts_help(&mut self) {
        self.show_shortcuts_help = false;
    }
}
