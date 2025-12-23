use serialport::SerialPort;
use std::time::Duration;

use crate::model::{FlowControl, Parity, SerialConfig};

pub fn list_ports() -> Vec<String> {
    match serialport::available_ports() {
        Ok(ports) => ports.iter().map(|p| p.port_name.clone()).collect(),
        Err(_) => Vec::new(),
    }
}

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

pub fn write_data(port: &mut dyn SerialPort, data: &[u8]) -> Result<usize, String> {
    port.write_all(data)
        .map(|_| data.len())
        .map_err(|e| format!("Write error: {}", e))
}

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

pub fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ")
}

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
