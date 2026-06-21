//! Serial port connection handler

use tuiserial_core::{AppError, AppState, ErrorContext, RecoveryStrategy, SerialErrorKind};
use tuiserial_serial::{SerialError, serialport::SerialPort};

/// Maximum number of consecutive read errors before auto-disconnect.
const MAX_CONSECUTIVE_READ_ERRORS: u32 = 5;

/// Handler for managing serial port connections
pub struct SerialHandler {
    port: Option<Box<dyn SerialPort>>,
    /// Counts consecutive read errors; reset on success.
    pub consecutive_read_errors: u32,
}

impl SerialHandler {
    /// Create a new serial handler
    pub fn new() -> Self {
        Self {
            port: None,
            consecutive_read_errors: 0,
        }
    }

    /// Connect to the serial port using the current configuration
    pub fn connect(&mut self, app: &AppState) -> Result<(), SerialError> {
        let port = tuiserial_serial::open_port(&app.config)?;
        self.port = Some(port);
        self.consecutive_read_errors = 0;
        Ok(())
    }

    /// Disconnect from the serial port
    pub fn disconnect(&mut self) {
        self.port = None;
        self.consecutive_read_errors = 0;
    }

    /// Check if the serial port is connected
    pub fn is_connected(&self) -> bool {
        self.port.is_some()
    }

    /// Send data to the serial port
    pub fn send(&mut self, data: &[u8]) -> Result<usize, SerialError> {
        match &mut self.port {
            Some(port) => tuiserial_serial::write_data(port.as_mut(), data),
            None => Err(SerialError::NotConnected),
        }
    }

    /// Read data from the serial port
    pub fn read(&mut self) -> Result<Vec<u8>, SerialError> {
        match &mut self.port {
            Some(port) => tuiserial_serial::read_data(port.as_mut()),
            None => Err(SerialError::NotConnected),
        }
    }

    /// Convert a `SerialError` into an `AppError` and track consecutive
    /// errors for auto-disconnect logic.
    ///
    /// Returns `true` if the port should be disconnected due to
    /// excessive errors.
    pub fn handle_read_error(&mut self, error: SerialError) -> (AppError, bool) {
        let kind: SerialErrorKind = error.into();
        match &kind {
            SerialErrorKind::NotConnected | SerialErrorKind::Io(_) => {
                self.consecutive_read_errors += 1;
            }
            _ => {}
        }

        let should_disconnect =
            self.consecutive_read_errors >= MAX_CONSECUTIVE_READ_ERRORS;

        let app_error = AppError::Serial {
            kind,
            ctx: ErrorContext::new(
                "serial",
                "read",
                if should_disconnect {
                    RecoveryStrategy::None
                } else {
                    RecoveryStrategy::Skip
                },
            ),
        };

        (app_error, should_disconnect)
    }

    /// Reset the consecutive read error counter (e.g., after a
    /// successful read).
    pub fn reset_read_errors(&mut self) {
        self.consecutive_read_errors = 0;
    }
}

impl Default for SerialHandler {
    fn default() -> Self {
        Self::new()
    }
}
