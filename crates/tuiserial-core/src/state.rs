//! Application state management
//!
//! This module defines the main application state structure that holds all
//! runtime data, UI state, and configuration for the tuiserial application.

use ratatui::widgets::ListState;
use serde_json;
use std::collections::VecDeque;

use crate::config::SerialConfig;
use crate::log::MessageLog;
use crate::notification::Notification;
use crate::types::{
    AppendMode, DisplayMode, FlowControl, FocusedField, Language, MenuState, Parity, TxMode,
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

    /// Toggle transmission mode
    pub fn toggle_tx_mode(&mut self) {
        self.tx_mode = match self.tx_mode {
            TxMode::Hex => TxMode::Ascii,
            TxMode::Ascii => TxMode::Hex,
        };
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
    pub fn save_config(&self) -> Result<(), String> {
        let config_dir =
            dirs::config_dir().ok_or_else(|| "Could not determine config directory".to_string())?;
        let app_config_dir = config_dir.join("tuiserial");
        std::fs::create_dir_all(&app_config_dir)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;

        let config_path = app_config_dir.join("config.json");
        let json = serde_json::to_string_pretty(&self.config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        std::fs::write(&config_path, json)
            .map_err(|e| format!("Failed to write config file: {}", e))?;

        Ok(())
    }

    /// Load configuration from file, return default if not found or error
    pub fn load_config(&mut self) {
        if let Some(config_dir) = dirs::config_dir() {
            let config_path = config_dir.join("tuiserial").join("config.json");
            if let Ok(json) = std::fs::read_to_string(&config_path) {
                if let Ok(config) = serde_json::from_str::<SerialConfig>(&json) {
                    // Update UI states to match loaded config
                    if let Some(idx) = self
                        .baud_rate_options
                        .iter()
                        .position(|&b| b == config.baud_rate)
                    {
                        self.baud_rate_state.select(Some(idx));
                    }
                    if let Some(idx) = self.parity_options.iter().position(|&p| p == config.parity)
                    {
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
    }

    // Language management

    /// Toggle language
    pub fn toggle_language(&mut self) {
        self.language = match self.language {
            Language::English => Language::Chinese,
            Language::Chinese => Language::English,
        };
    }
}
