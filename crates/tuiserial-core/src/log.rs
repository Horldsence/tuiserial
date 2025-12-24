//! Log entry and message log functionality
//!
//! This module defines log entries for serial communication events and
//! the message log that stores communication history.

use chrono::{DateTime, Local};
use std::collections::VecDeque;

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
    /// Create a new log entry with the current timestamp
    pub fn new(direction: LogDirection, data: Vec<u8>) -> Self {
        Self {
            timestamp: Local::now(),
            direction,
            data,
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
    /// Create a new empty message log
    pub fn new() -> Self {
        Self {
            entries: VecDeque::with_capacity(MAX_LOG_LINES),
            rx_count: 0,
            tx_count: 0,
        }
    }

    /// Add a received data entry to the log
    pub fn push_rx(&mut self, data: Vec<u8>) {
        self.push_entry(LogEntry::new(LogDirection::Rx, data));
        self.rx_count += 1;
    }

    /// Add a transmitted data entry to the log
    pub fn push_tx(&mut self, data: Vec<u8>) {
        self.push_entry(LogEntry::new(LogDirection::Tx, data));
        self.tx_count += 1;
    }

    /// Internal method to add an entry, maintaining size limit
    fn push_entry(&mut self, entry: LogEntry) {
        if self.entries.len() >= MAX_LOG_LINES {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
    }

    /// Clear all log entries and reset counters
    pub fn clear(&mut self) {
        self.entries.clear();
        self.rx_count = 0;
        self.tx_count = 0;
    }
}
