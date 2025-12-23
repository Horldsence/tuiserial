use std::collections::VecDeque;
use std::time::Instant;
use ratatui::widgets::ListState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationLevel {
    Info,
    Warning,
    Error,
    Success,
}

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
            duration_ms: 3000, // 默认显示3秒
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayMode {
    Hex,
    Text,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TxMode {
    Hex,
    Ascii,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Parity {
    None,
    Even,
    Odd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowControl {
    None,
    Hardware,
    Software,
}

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

pub const MAX_LOG_LINES: usize = 10000;

pub struct RxBuffer {
    buf: VecDeque<u8>,
    rx_count: u64,
}

impl RxBuffer {
    pub fn new() -> Self {
        Self {
            buf: VecDeque::with_capacity(MAX_LOG_LINES),
            rx_count: 0,
        }
    }

    pub fn push(&mut self, byte: u8) {
        if self.buf.len() >= MAX_LOG_LINES {
            self.buf.pop_front();
        }
        self.buf.push_back(byte);
        self.rx_count += 1;
    }

    pub fn extend(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.push(b);
        }
    }

    pub fn clear(&mut self) {
        self.buf.clear();
    }

    pub fn as_slice(&self) -> &[u8] {
        self.buf.as_slices().0
    }

    pub fn rx_count(&self) -> u64 {
        self.rx_count
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }
}

pub struct AppState {
    pub config: SerialConfig,
    pub rx_buffer: RxBuffer,
    pub tx_count: u64,
    pub display_mode: DisplayMode,
    pub is_connected: bool,
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
    pub tx_cursor: usize,
    
    // UI Focus
    pub focused_field: FocusedField,
    
    // Notification system
    pub notifications: VecDeque<Notification>,
}

impl Default for AppState {
    fn default() -> Self {
        let baud_rate_options = vec![300, 1200, 2400, 4800, 9600, 19200, 38400, 57600, 115200, 230400];
        let parity_options = vec![Parity::None, Parity::Even, Parity::Odd];
        let flow_control_options = vec![FlowControl::None, FlowControl::Hardware, FlowControl::Software];
        let data_bits_options = vec![5, 6, 7, 8];
        let stop_bits_options = vec![1, 2];

        Self {
            config: SerialConfig::default(),
            rx_buffer: RxBuffer::new(),
            tx_count: 0,
            display_mode: DisplayMode::Hex,
            is_connected: false,
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
            tx_cursor: 0,
            focused_field: FocusedField::Port,
            notifications: VecDeque::new(),
        }
    }
}

impl AppState {
    pub fn add_notification(&mut self, notification: Notification) {
        self.notifications.push_back(notification);
    }

    pub fn add_info(&mut self, msg: impl Into<String>) {
        self.add_notification(Notification::info(msg.into()));
    }

    pub fn add_warning(&mut self, msg: impl Into<String>) {
        self.add_notification(Notification::warning(msg.into()));
    }

    pub fn add_error(&mut self, msg: impl Into<String>) {
        self.add_notification(Notification::error(msg.into()));
    }

    pub fn add_success(&mut self, msg: impl Into<String>) {
        self.add_notification(Notification::success(msg.into()));
    }

    pub fn update_notifications(&mut self) {
        while let Some(front) = self.notifications.front() {
            if front.is_expired() {
                self.notifications.pop_front();
            } else {
                break;
            }
        }
    }
}

impl AppState {
    pub fn next_baud_rate(&mut self) {
        if let Some(selected) = self.baud_rate_state.selected() {
            let next = (selected + 1) % self.baud_rate_options.len();
            self.baud_rate_state.select(Some(next));
            self.config.baud_rate = self.baud_rate_options[next];
        }
    }

    pub fn prev_baud_rate(&mut self) {
        if let Some(selected) = self.baud_rate_state.selected() {
            let next = if selected == 0 { self.baud_rate_options.len() - 1 } else { selected - 1 };
            self.baud_rate_state.select(Some(next));
            self.config.baud_rate = self.baud_rate_options[next];
        }
    }

    pub fn toggle_parity(&mut self) {
        if let Some(selected) = self.parity_state.selected() {
            let next = (selected + 1) % self.parity_options.len();
            self.parity_state.select(Some(next));
            self.config.parity = self.parity_options[next];
        }
    }

    pub fn toggle_flow_control(&mut self) {
        if let Some(selected) = self.flow_control_state.selected() {
            let next = (selected + 1) % self.flow_control_options.len();
            self.flow_control_state.select(Some(next));
            self.config.flow_control = self.flow_control_options[next];
        }
    }

    pub fn next_data_bits(&mut self) {
        if let Some(selected) = self.data_bits_state.selected() {
            let next = (selected + 1) % self.data_bits_options.len();
            self.data_bits_state.select(Some(next));
            self.config.data_bits = self.data_bits_options[next];
        }
    }

    pub fn next_stop_bits(&mut self) {
        if let Some(selected) = self.stop_bits_state.selected() {
            let next = (selected + 1) % self.stop_bits_options.len();
            self.stop_bits_state.select(Some(next));
            self.config.stop_bits = self.stop_bits_options[next];
        }
    }

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
