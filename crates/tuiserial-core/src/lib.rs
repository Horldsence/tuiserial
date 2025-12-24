//! Core data models and types for tuiserial
//!
//! This crate provides the fundamental data structures, enums, and state management
//! types used throughout the tuiserial application.

use chrono::{DateTime, Local};
use ratatui::widgets::ListState;
use std::collections::VecDeque;
use std::time::Instant;

// Re-exports
pub use chrono;
pub use ratatui;
pub use serde;
pub use serde_json;

/// Notification level for user messages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationLevel {
    Info,
    Warning,
    Error,
    Success,
}

/// Direction of serial communication
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogDirection {
    Rx,
    Tx,
}

/// A single log entry representing a serial communication event
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: DateTime<Local>,
    pub direction: LogDirection,
    pub data: Vec<u8>,
}

impl LogEntry {
    pub fn new(direction: LogDirection, data: Vec<u8>) -> Self {
        Self {
            timestamp: Local::now(),
            direction,
            data,
        }
    }
}

/// A notification message shown to the user
#[derive(Debug, Clone)]
pub struct Notification {
    pub message: String,
    pub level: NotificationLevel,
    pub created_at: Instant,
    pub duration_ms: u64,
}

impl Notification {
    pub fn new(message: String, level: NotificationLevel) -> Self {
        Self {
            message,
            level,
            created_at: Instant::now(),
            duration_ms: 3000, // Default 3 seconds
        }
    }

    pub fn info(message: String) -> Self {
        Self::new(message, NotificationLevel::Info)
    }

    pub fn warning(message: String) -> Self {
        Self::new(message, NotificationLevel::Warning)
    }

    pub fn error(message: String) -> Self {
        Self::new(message, NotificationLevel::Error)
    }

    pub fn success(message: String) -> Self {
        Self::new(message, NotificationLevel::Success)
    }

    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed().as_millis() as u64 > self.duration_ms
    }
}

/// Display mode for serial data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayMode {
    Hex,
    Text,
}

/// Transmission mode for sending data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TxMode {
    Hex,
    Ascii,
}

/// Data append options for transmission
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppendMode {
    None, // 无追加
    LF,   // \n (0x0A)
    CR,   // \r (0x0D)
    CRLF, // \r\n (0x0D 0x0A)
    LFCR, // \n\r (0x0A 0x0D)
}

impl AppendMode {
    /// Get the bytes for this append mode
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            AppendMode::None => &[],
            AppendMode::LF => &[0x0A],
            AppendMode::CR => &[0x0D],
            AppendMode::CRLF => &[0x0D, 0x0A],
            AppendMode::LFCR => &[0x0A, 0x0D],
        }
    }

    /// Get the display name
    pub fn name(&self) -> &str {
        match self {
            AppendMode::None => "无追加",
            AppendMode::LF => "\\n",
            AppendMode::CR => "\\r",
            AppendMode::CRLF => "\\r\\n",
            AppendMode::LFCR => "\\n\\r",
        }
    }

    /// Get all available append modes
    pub fn all() -> Vec<AppendMode> {
        vec![
            AppendMode::None,
            AppendMode::LF,
            AppendMode::CR,
            AppendMode::CRLF,
            AppendMode::LFCR,
        ]
    }
}

/// Serial port parity setting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Parity {
    None,
    Even,
    Odd,
}

/// Serial port flow control setting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowControl {
    None,
    Hardware,
    Software,
}

/// UI field that currently has focus
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusedField {
    Port,
    BaudRate,
    DataBits,
    Parity,
    StopBits,
    FlowControl,
    LogArea,
    TxInput,
}

/// Serial port configuration
#[derive(Debug, Clone)]
pub struct SerialConfig {
    pub port: String,
    pub baud_rate: u32,
    pub data_bits: u8,
    pub parity: Parity,
    pub stop_bits: u8,
    pub flow_control: FlowControl,
}

impl Default for SerialConfig {
    fn default() -> Self {
        Self {
            port: String::new(),
            baud_rate: 9600,
            data_bits: 8,
            parity: Parity::None,
            stop_bits: 1,
            flow_control: FlowControl::None,
        }
    }
}

/// Maximum number of log lines to keep in memory
pub const MAX_LOG_LINES: usize = 10000;

/// Message log containing all serial communication events
#[derive(Debug, Default)]
pub struct MessageLog {
    pub entries: VecDeque<LogEntry>,
    pub rx_count: u64,
    pub tx_count: u64,
}

impl MessageLog {
    pub fn new() -> Self {
        Self {
            entries: VecDeque::with_capacity(MAX_LOG_LINES),
            rx_count: 0,
            tx_count: 0,
        }
    }

    pub fn push_rx(&mut self, data: Vec<u8>) {
        self.push_entry(LogEntry::new(LogDirection::Rx, data));
        self.rx_count += 1;
    }

    pub fn push_tx(&mut self, data: Vec<u8>) {
        self.push_entry(LogEntry::new(LogDirection::Tx, data));
        self.tx_count += 1;
    }

    fn push_entry(&mut self, entry: LogEntry) {
        if self.entries.len() >= MAX_LOG_LINES {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.rx_count = 0;
        self.tx_count = 0;
    }
}

/// Main application state
pub struct AppState {
    pub config: SerialConfig,
    pub message_log: MessageLog,
    pub display_mode: DisplayMode,
    pub is_connected: bool,
    pub config_locked: bool, // 配置是否被锁定（连接时锁定）
    pub ports: Vec<String>,
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

    // TX Input
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
        }
    }
}

impl AppState {
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

    /// Toggle display mode
    pub fn toggle_display_mode(&mut self) {
        self.display_mode = match self.display_mode {
            DisplayMode::Hex => DisplayMode::Text,
            DisplayMode::Text => DisplayMode::Hex,
        };
    }

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
}
