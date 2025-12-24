//! Serial port session management
//!
//! This module provides session management for multiple serial port connections,
//! allowing users to monitor and interact with multiple serial ports simultaneously.

use ratatui::widgets::ListState;
use std::collections::VecDeque;

use crate::config::SerialConfig;
use crate::log::MessageLog;
use crate::notification::Notification;
use crate::types::{AppendMode, DisplayMode, FlowControl, FocusedField, Parity, TxMode};

/// A single serial port session
#[derive(Clone)]
pub struct SerialSession {
    /// Unique session ID
    pub id: usize,

    /// Session name (user-customizable)
    pub name: String,

    /// Serial configuration for this session
    pub config: SerialConfig,

    /// Message log for this session
    pub message_log: MessageLog,

    /// Display mode (Hex/Text)
    pub display_mode: DisplayMode,

    /// Connection status
    pub is_connected: bool,

    /// Configuration lock status
    pub config_locked: bool,

    /// Scroll state
    pub scroll_offset: u16,
    pub auto_scroll: bool,

    /// TX Input state
    pub tx_input: String,
    pub tx_mode: TxMode,
    pub tx_append_mode: AppendMode,
    pub tx_cursor: usize,

    /// UI State for this session
    pub port_list_state: ListState,
    pub baud_rate_state: ListState,
    pub parity_state: ListState,
    pub flow_control_state: ListState,
    pub data_bits_state: ListState,
    pub stop_bits_state: ListState,
    pub append_mode_state: ListState,

    /// Focused field for this session
    pub focused_field: FocusedField,

    /// Session-specific notifications
    pub notifications: VecDeque<Notification>,
}

impl SerialSession {
    /// Create a new session with default configuration
    pub fn new(id: usize, name: String) -> Self {
        Self {
            id,
            name,
            config: SerialConfig::default(),
            message_log: MessageLog::new(),
            display_mode: DisplayMode::Hex,
            is_connected: false,
            config_locked: false,
            scroll_offset: 0,
            auto_scroll: true,
            port_list_state: ListState::default().with_selected(Some(0)),
            baud_rate_state: ListState::default().with_selected(Some(4)), // 9600
            parity_state: ListState::default().with_selected(Some(0)),    // None
            flow_control_state: ListState::default().with_selected(Some(0)), // None
            data_bits_state: ListState::default().with_selected(Some(3)), // 8
            stop_bits_state: ListState::default().with_selected(Some(0)), // 1
            tx_input: String::new(),
            tx_mode: TxMode::Ascii,
            tx_append_mode: AppendMode::None,
            tx_cursor: 0,
            append_mode_state: ListState::default().with_selected(Some(0)),
            focused_field: FocusedField::Port,
            notifications: VecDeque::new(),
        }
    }

    /// Create a new session with specific port
    pub fn with_port(id: usize, name: String, port: String) -> Self {
        let mut session = Self::new(id, name);
        session.config.port = port;
        session
    }

    /// Lock configuration (when connecting)
    pub fn lock_config(&mut self) {
        self.config_locked = true;
    }

    /// Unlock configuration (when disconnecting)
    pub fn unlock_config(&mut self) {
        self.config_locked = false;
    }

    /// Check if configuration can be modified
    pub fn can_modify_config(&self) -> bool {
        !self.config_locked
    }

    /// Add a notification to this session
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

    /// Update UI states from configuration
    pub fn update_ui_states(
        &mut self,
        baud_rate_options: &[u32],
        parity_options: &[Parity],
        flow_control_options: &[FlowControl],
        data_bits_options: &[u8],
        stop_bits_options: &[u8],
    ) {
        if let Some(idx) = baud_rate_options
            .iter()
            .position(|&b| b == self.config.baud_rate)
        {
            self.baud_rate_state.select(Some(idx));
        }
        if let Some(idx) = parity_options.iter().position(|&p| p == self.config.parity) {
            self.parity_state.select(Some(idx));
        }
        if let Some(idx) = flow_control_options
            .iter()
            .position(|&f| f == self.config.flow_control)
        {
            self.flow_control_state.select(Some(idx));
        }
        if let Some(idx) = data_bits_options
            .iter()
            .position(|&d| d == self.config.data_bits)
        {
            self.data_bits_state.select(Some(idx));
        }
        if let Some(idx) = stop_bits_options
            .iter()
            .position(|&s| s == self.config.stop_bits)
        {
            self.stop_bits_state.select(Some(idx));
        }
    }

    /// Toggle display mode
    pub fn toggle_display_mode(&mut self) {
        self.display_mode = match self.display_mode {
            DisplayMode::Hex => DisplayMode::Text,
            DisplayMode::Text => DisplayMode::Hex,
        };
    }

    /// Toggle TX mode
    pub fn toggle_tx_mode(&mut self) {
        self.tx_mode = match self.tx_mode {
            TxMode::Hex => TxMode::Ascii,
            TxMode::Ascii => TxMode::Hex,
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

/// Session manager for handling multiple serial port sessions
pub struct SessionManager {
    /// All sessions
    sessions: Vec<SerialSession>,

    /// Currently active session index
    active_session: usize,

    /// Next session ID
    next_id: usize,
}

impl SessionManager {
    /// Create a new session manager with one default session
    pub fn new() -> Self {
        let mut sessions = Vec::new();
        sessions.push(SerialSession::new(0, "Session 1".to_string()));

        Self {
            sessions,
            active_session: 0,
            next_id: 1,
        }
    }

    /// Get the number of sessions
    pub fn len(&self) -> usize {
        self.sessions.len()
    }

    /// Check if there are no sessions
    pub fn is_empty(&self) -> bool {
        self.sessions.is_empty()
    }

    /// Get the active session
    pub fn active_session(&self) -> &SerialSession {
        &self.sessions[self.active_session]
    }

    /// Get the active session mutably
    pub fn active_session_mut(&mut self) -> &mut SerialSession {
        &mut self.sessions[self.active_session]
    }

    /// Get the active session index
    pub fn active_index(&self) -> usize {
        self.active_session
    }

    /// Get a session by index
    pub fn get_session(&self, index: usize) -> Option<&SerialSession> {
        self.sessions.get(index)
    }

    /// Get a session mutably by index
    pub fn get_session_mut(&mut self, index: usize) -> Option<&mut SerialSession> {
        self.sessions.get_mut(index)
    }

    /// Get all sessions
    pub fn sessions(&self) -> &[SerialSession] {
        &self.sessions
    }

    /// Get all sessions mutably
    pub fn sessions_mut(&mut self) -> &mut [SerialSession] {
        &mut self.sessions
    }

    /// Add a new session
    pub fn add_session(&mut self, name: Option<String>) -> usize {
        let id = self.next_id;
        self.next_id += 1;

        let name = name.unwrap_or_else(|| format!("Session {}", id + 1));
        let session = SerialSession::new(id, name);

        self.sessions.push(session);
        self.sessions.len() - 1
    }

    /// Add a new session with specific port
    pub fn add_session_with_port(&mut self, port: String, name: Option<String>) -> usize {
        let id = self.next_id;
        self.next_id += 1;

        let name = name.unwrap_or_else(|| format!("Session {} - {}", id + 1, port));
        let session = SerialSession::with_port(id, name, port);

        self.sessions.push(session);
        self.sessions.len() - 1
    }

    /// Remove a session by index
    pub fn remove_session(&mut self, index: usize) -> Option<SerialSession> {
        if self.sessions.len() <= 1 {
            // Keep at least one session
            return None;
        }

        if index >= self.sessions.len() {
            return None;
        }

        let session = self.sessions.remove(index);

        // Adjust active session if needed
        if self.active_session >= self.sessions.len() {
            self.active_session = self.sessions.len().saturating_sub(1);
        } else if self.active_session > index {
            self.active_session -= 1;
        }

        Some(session)
    }

    /// Switch to a specific session
    pub fn switch_to(&mut self, index: usize) -> bool {
        if index < self.sessions.len() {
            self.active_session = index;
            true
        } else {
            false
        }
    }

    /// Switch to next session
    pub fn next_session(&mut self) {
        if !self.sessions.is_empty() {
            self.active_session = (self.active_session + 1) % self.sessions.len();
        }
    }

    /// Switch to previous session
    pub fn prev_session(&mut self) {
        if !self.sessions.is_empty() {
            if self.active_session == 0 {
                self.active_session = self.sessions.len() - 1;
            } else {
                self.active_session -= 1;
            }
        }
    }

    /// Rename a session
    pub fn rename_session(&mut self, index: usize, name: String) -> bool {
        if let Some(session) = self.sessions.get_mut(index) {
            session.name = name;
            true
        } else {
            false
        }
    }

    /// Duplicate current session
    pub fn duplicate_active_session(&mut self) -> usize {
        let mut new_session = self.active_session().clone();
        new_session.id = self.next_id;
        self.next_id += 1;
        new_session.name = format!("{} (Copy)", new_session.name);
        new_session.is_connected = false;
        new_session.config_locked = false;
        new_session.message_log = MessageLog::new();

        self.sessions.push(new_session);
        self.sessions.len() - 1
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}
