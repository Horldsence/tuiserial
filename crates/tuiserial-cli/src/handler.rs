//! Serial port connection handler

use tuiserial_core::AppState;
use tuiserial_serial::serialport::SerialPort;

/// Handler for managing serial port connections
pub struct SerialHandler {
    port: Option<Box<dyn SerialPort>>,
}

impl SerialHandler {
    /// Create a new serial handler
    pub fn new() -> Self {
        Self { port: None }
    }

    /// Connect to the serial port using the current configuration
    pub fn connect(&mut self, app: &AppState) -> Result<(), String> {
        match tuiserial_serial::open_port(&app.config) {
            Ok(port) => {
                self.port = Some(port);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    /// Disconnect from the serial port
    pub fn disconnect(&mut self) {
        self.port = None;
    }

    /// Check if the serial port is connected
    pub fn is_connected(&self) -> bool {
        self.port.is_some()
    }

    /// Send data to the serial port
    pub fn send(&mut self, data: &[u8]) -> Result<usize, String> {
        match &mut self.port {
            Some(port) => tuiserial_serial::write_data(port.as_mut(), data),
            None => Err("Port not connected".to_string()),
        }
    }

    /// Read data from the serial port
    pub fn read(&mut self) -> Result<Vec<u8>, String> {
        match &mut self.port {
            Some(port) => tuiserial_serial::read_data(port.as_mut()),
            None => Err("Port not connected".to_string()),
        }
    }
}

impl Default for SerialHandler {
    fn default() -> Self {
        Self::new()
    }
}
