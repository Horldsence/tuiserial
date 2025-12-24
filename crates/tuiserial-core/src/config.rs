//! Serial port configuration types
//!
//! This module defines the serial port configuration structure and related
//! settings for establishing serial connections.

use serde::{Deserialize, Serialize};

use crate::types::{FlowControl, Parity};

/// Serial port configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl SerialConfig {
    /// Create a new serial configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a configuration with specified port and default other values
    pub fn with_port(port: impl Into<String>) -> Self {
        Self {
            port: port.into(),
            ..Default::default()
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.port.is_empty() {
            return Err("Port cannot be empty".to_string());
        }

        if self.baud_rate == 0 {
            return Err("Baud rate must be greater than 0".to_string());
        }

        if self.data_bits < 5 || self.data_bits > 8 {
            return Err("Data bits must be between 5 and 8".to_string());
        }

        if self.stop_bits < 1 || self.stop_bits > 2 {
            return Err("Stop bits must be 1 or 2".to_string());
        }

        Ok(())
    }

    /// Format configuration as a human-readable string
    pub fn format_display(&self) -> String {
        let parity_char = match self.parity {
            Parity::None => 'N',
            Parity::Even => 'E',
            Parity::Odd => 'O',
        };

        format!(
            "{} @ {} bps, {}-{}-{}",
            self.port, self.baud_rate, self.data_bits, parity_char, self.stop_bits
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SerialConfig::default();
        assert_eq!(config.port, "");
        assert_eq!(config.baud_rate, 9600);
        assert_eq!(config.data_bits, 8);
        assert_eq!(config.stop_bits, 1);
    }

    #[test]
    fn test_with_port() {
        let config = SerialConfig::with_port("/dev/ttyUSB0");
        assert_eq!(config.port, "/dev/ttyUSB0");
        assert_eq!(config.baud_rate, 9600);
    }

    #[test]
    fn test_validate() {
        let mut config = SerialConfig::default();
        assert!(config.validate().is_err()); // Empty port

        config.port = "/dev/ttyUSB0".to_string();
        assert!(config.validate().is_ok());

        config.baud_rate = 0;
        assert!(config.validate().is_err());

        config.baud_rate = 9600;
        config.data_bits = 9;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_format_display() {
        let config = SerialConfig {
            port: "/dev/ttyUSB0".to_string(),
            baud_rate: 115200,
            data_bits: 8,
            parity: Parity::None,
            stop_bits: 1,
            flow_control: FlowControl::None,
        };

        let display = config.format_display();
        assert_eq!(display, "/dev/ttyUSB0 @ 115200 bps, 8-N-1");
    }
}
