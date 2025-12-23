use serialport::SerialPort;

use crate::model::AppState;

pub struct SerialHandler {
    port: Option<Box<dyn SerialPort>>,
}

impl SerialHandler {
    pub fn new() -> Self {
        Self {
            port: None,
        }
    }

    pub fn connect(&mut self, app: &AppState) -> Result<(), String> {
        // Try to open the serial port
        match crate::serial::open_port(&app.config) {
            Ok(port) => {
                self.port = Some(port);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    pub fn disconnect(&mut self) {
        self.port = None;
    }

    pub fn is_connected(&self) -> bool {
        self.port.is_some()
    }

    pub fn send(&mut self, data: &[u8]) -> Result<usize, String> {
        match &mut self.port {
            Some(port) => crate::serial::write_data(port.as_mut(), data),
            None => Err("Port not connected".to_string()),
        }
    }

    pub fn read(&mut self) -> Result<Vec<u8>, String> {
        match &mut self.port {
            Some(port) => crate::serial::read_data(port.as_mut()),
            None => Err("Port not connected".to_string()),
        }
    }
}
