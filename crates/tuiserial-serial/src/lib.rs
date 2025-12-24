//! Serial port communication library for tuiserial
//!
//! This crate provides serial port operations including port enumeration,
//! connection management, and data transmission.

use serialport::SerialPort;
use std::time::Duration;
use tuiserial_core::{FlowControl, Parity, SerialConfig};

// Re-exports
pub use serialport;
pub use tokio;

/// List all available serial ports on the system
pub fn list_ports() -> Vec<String> {
    match serialport::available_ports() {
        Ok(ports) => ports.iter().map(|p| p.port_name.clone()).collect(),
        Err(_) => Vec::new(),
    }
}

/// Open a serial port with the given configuration
pub fn open_port(config: &SerialConfig) -> Result<Box<dyn SerialPort>, String> {
    serialport::new(&config.port, config.baud_rate)
        .timeout(Duration::from_millis(10))
        .data_bits(match config.data_bits {
            5 => serialport::DataBits::Five,
            6 => serialport::DataBits::Six,
            7 => serialport::DataBits::Seven,
            _ => serialport::DataBits::Eight,
        })
        .parity(match config.parity {
            Parity::Even => serialport::Parity::Even,
            Parity::Odd => serialport::Parity::Odd,
            Parity::None => serialport::Parity::None,
        })
        .stop_bits(match config.stop_bits {
            1 => serialport::StopBits::One,
            2 => serialport::StopBits::Two,
            _ => serialport::StopBits::One,
        })
        .flow_control(match config.flow_control {
            FlowControl::Hardware => serialport::FlowControl::Hardware,
            FlowControl::Software => serialport::FlowControl::Software,
            FlowControl::None => serialport::FlowControl::None,
        })
        .open_native()
        .map_err(|e| format!("Failed to open port: {}", e))
        .map(|p| Box::new(p) as Box<dyn SerialPort>)
}

/// Read data from the serial port
pub fn read_data(port: &mut dyn SerialPort) -> Result<Vec<u8>, String> {
    let mut buf = vec![0u8; 256];
    match port.read(buf.as_mut_slice()) {
        Ok(n) if n > 0 => {
            buf.truncate(n);
            Ok(buf)
        }
        Ok(_) => Ok(Vec::new()),
        Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => Ok(Vec::new()),
        Err(e) => Err(format!("Read error: {}", e)),
    }
}

/// Write data to the serial port
pub fn write_data(port: &mut dyn SerialPort, data: &[u8]) -> Result<usize, String> {
    port.write_all(data)
        .map(|_| data.len())
        .map_err(|e| format!("Write error: {}", e))
}

/// Convert hex string to bytes
///
/// # Example
/// ```
/// use tuiserial_serial::hex_to_bytes;
/// let bytes = hex_to_bytes("48656C6C6F").unwrap();
/// assert_eq!(bytes, vec![0x48, 0x65, 0x6C, 0x6C, 0x6F]);
/// ```
pub fn hex_to_bytes(hex_str: &str) -> Result<Vec<u8>, String> {
    let hex_str = hex_str.trim().replace(" ", "");
    if hex_str.len() % 2 != 0 {
        return Err("Hex string must have even length".to_string());
    }

    hex_str
        .chars()
        .collect::<Vec<_>>()
        .chunks(2)
        .map(|chunk| {
            u8::from_str_radix(&chunk.iter().collect::<String>(), 16)
                .map_err(|_| "Invalid hex character".to_string())
        })
        .collect()
}

/// Convert bytes to hex string representation
///
/// # Example
/// ```
/// use tuiserial_serial::bytes_to_hex;
/// let hex = bytes_to_hex(&[0x48, 0x65, 0x6C, 0x6C, 0x6F]);
/// assert_eq!(hex, "48 65 6C 6C 6F");
/// ```
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Convert bytes to string, escaping non-printable characters
///
/// # Example
/// ```
/// use tuiserial_serial::bytes_to_string;
/// let s = bytes_to_string(&[0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x0A]);
/// assert_eq!(s, "Hello\\x0A");
/// ```
pub fn bytes_to_string(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|&b| {
            if b >= 32 && b < 127 {
                (b as char).to_string()
            } else {
                format!("\\x{:02X}", b)
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_to_bytes() {
        assert_eq!(
            hex_to_bytes("48656C6C6F").unwrap(),
            vec![0x48, 0x65, 0x6C, 0x6C, 0x6F]
        );
        assert_eq!(
            hex_to_bytes("48 65 6C 6C 6F").unwrap(),
            vec![0x48, 0x65, 0x6C, 0x6C, 0x6F]
        );
        assert!(hex_to_bytes("4865F").is_err()); // Odd length
        assert!(hex_to_bytes("48XY").is_err()); // Invalid hex
    }

    #[test]
    fn test_bytes_to_hex() {
        assert_eq!(
            bytes_to_hex(&[0x48, 0x65, 0x6C, 0x6C, 0x6F]),
            "48 65 6C 6C 6F"
        );
        assert_eq!(bytes_to_hex(&[]), "");
    }

    #[test]
    fn test_bytes_to_string() {
        assert_eq!(bytes_to_string(&[0x48, 0x65, 0x6C, 0x6C, 0x6F]), "Hello");
        assert_eq!(
            bytes_to_string(&[0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x0A]),
            "Hello\\x0A"
        );
        assert_eq!(bytes_to_string(&[0x00, 0x1F, 0x7F]), "\\x00\\x1F\\x7F");
    }
}
