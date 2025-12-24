//! Basic types and enums for tuiserial
//!
//! This module contains fundamental type definitions and enums used throughout
//! the application, including display modes, transmission modes, parity settings, etc.

use serde::{Deserialize, Serialize};

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Parity {
    None,
    Even,
    Odd,
}

/// Serial port flow control setting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

/// Language selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    English,
    Chinese,
}

impl Language {
    pub fn all() -> Vec<Language> {
        vec![Language::English, Language::Chinese]
    }

    pub fn name(&self) -> &str {
        match self {
            Language::English => "English",
            Language::Chinese => "中文",
        }
    }
}

/// Menu state for UI navigation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuState {
    None,
    MenuBar(usize),         // Selected menu index
    Dropdown(usize, usize), // Menu index, selected item index
}
