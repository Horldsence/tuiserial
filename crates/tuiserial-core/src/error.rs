//! Unified error types for tuiserial.
//!
//! Defines a layered error architecture:
//! - `AppError`: the top-level unified error enum used across all crates
//! - `ErrorSeverity`: classifies how severe an error is
//! - `RecoveryStrategy`: hints at what the caller should do
//! - `ErrorContext`: metadata about where/when/why an error occurred
//! - Sub-kind enums (`SerialErrorKind`, `PluginErrorKind`, `ConfigErrorKind`):
//!   pure data enums that describe error details without implementing `Error`

use std::fmt;
use std::time::Instant;

use thiserror::Error;

// ── Severity ──────────────────────────────────────────────────────

/// Classifies the severity of an application error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    /// Informational, no action needed.
    Info = 0,
    /// Non-critical, operation continues with degraded state.
    Warning = 1,
    /// Operation failed but the app continues normally.
    Error = 2,
    /// App can continue but a feature is unavailable.
    Critical = 3,
    /// App must exit.
    Fatal = 4,
}

// ── Recovery strategy ─────────────────────────────────────────────

/// Suggests how the caller should recover from an error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryStrategy {
    /// No recovery possible — propagate to user.
    None,
    /// Operation can be retried immediately.
    Retry,
    /// Retry with backoff (max duration).
    RetryWithBackoff(std::time::Duration),
    /// Skip this operation, continue with the next.
    Skip,
    /// Disable the affected component/plugin but keep the app running.
    DisableComponent,
    /// Use a fallback or default value.
    UseFallback,
}

// ── Error context ─────────────────────────────────────────────────

/// Metadata attached to every `AppError` variant.
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// Crate or module where the error originated.
    pub source_module: &'static str,
    /// Human-readable description of what was being attempted.
    pub operation: String,
    /// When the error occurred.
    pub timestamp: Instant,
    /// Suggested recovery strategy.
    pub recovery: RecoveryStrategy,
}

impl ErrorContext {
    pub fn new(
        module: &'static str,
        operation: impl Into<String>,
        recovery: RecoveryStrategy,
    ) -> Self {
        Self {
            source_module: module,
            operation: operation.into(),
            timestamp: Instant::now(),
            recovery,
        }
    }
}

// ── Serial error kind ─────────────────────────────────────────────

/// Data-only description of a serial-port error.
///
/// This is intentionally **not** an `Error` type — it is a payload
/// carried inside `AppError::Serial`.  The `From` conversions live in
/// `tuiserial-serial`.
#[derive(Debug)]
pub enum SerialErrorKind {
    /// Failed to open the serial port.
    PortOpen(String),
    /// I/O error during read/write.
    Io(String),
    /// Hex string has an odd number of characters.
    InvalidHexLength,
    /// Invalid hex character encountered.
    ParseHex(String),
    /// Port is not connected.
    NotConnected,
}

impl fmt::Display for SerialErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PortOpen(e) => write!(f, "Failed to open port: {e}"),
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::InvalidHexLength => write!(f, "Hex string must have an even length"),
            Self::ParseHex(e) => write!(f, "Invalid hex character: {e}"),
            Self::NotConnected => write!(f, "Port is not connected"),
        }
    }
}

// ── Plugin error kind ─────────────────────────────────────────────

/// Data-only description of a plugin error.
#[derive(Debug)]
pub enum PluginErrorKind {
    /// I/O error (reading plugin file, etc.).
    Io(String),
    /// Error in the plugin JavaScript/TypeScript source.
    Script(String),
    /// Error in the Boa JS runtime.
    Runtime(String),
    /// Git operation failed.
    Git(String),
    /// Plugin panicked (caught by `catch_unwind`).
    Panic {
        /// The hook or operation that panicked.
        hook: String,
        /// Panic message extracted from the unwind payload.
        message: String,
    },
    /// Plugin execution timed out (future).
    Timeout,
    /// Maximum retries exceeded for a transient error.
    MaxRetriesExceeded,
}

impl fmt::Display for PluginErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O: {e}"),
            Self::Script(m) => write!(f, "Script: {m}"),
            Self::Runtime(m) => write!(f, "Runtime: {m}"),
            Self::Git(m) => write!(f, "Git: {m}"),
            Self::Panic { hook, message } => write!(f, "Panic in {hook}: {message}"),
            Self::Timeout => write!(f, "Plugin execution timed out"),
            Self::MaxRetriesExceeded => write!(f, "Maximum retries exceeded"),
        }
    }
}

// ── Config error kind ─────────────────────────────────────────────

/// Data-only description of a configuration error.
///
/// `CoreError` (below) is kept for backward compatibility and
/// converts into this type.
#[derive(Debug)]
pub enum ConfigErrorKind {
    /// Config directory not found.
    ConfigDirNotFound,
    /// JSON serialization / deserialization error.
    Serialization(serde_json::Error),
    /// Configuration value failed validation.
    Validation(String),
}

impl fmt::Display for ConfigErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConfigDirNotFound => write!(f, "Config directory not found"),
            Self::Serialization(e) => write!(f, "Serialization: {e}"),
            Self::Validation(m) => write!(f, "{m}"),
        }
    }
}

// ── CoreError (backward-compatible) ───────────────────────────────

/// Legacy core error type — kept for backward compatibility.
///
/// New code should prefer `AppError`.  `CoreError` converts into
/// `AppError` automatically.
#[derive(Error, Debug)]
pub enum CoreError {
    #[error("{0}")]
    Validation(String),

    #[error("Config directory not found")]
    ConfigDirNotFound,

    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization: {0}")]
    Serde(#[from] serde_json::Error),
}

impl From<CoreError> for ConfigErrorKind {
    fn from(e: CoreError) -> Self {
        match e {
            CoreError::Validation(s) => Self::Validation(s),
            CoreError::ConfigDirNotFound => Self::ConfigDirNotFound,
            CoreError::Io(io) => {
                // IO errors from config are treated as serialization failures
                // because they only happen during save/load.
                Self::Serialization(serde_json::Error::io(io))
            }
            CoreError::Serde(e) => Self::Serialization(e),
        }
    }
}

impl From<CoreError> for AppError {
    fn from(e: CoreError) -> Self {
        let config_kind: ConfigErrorKind = e.into();
        AppError::Config {
            kind: config_kind,
            ctx: ErrorContext::new("core", "config operation", RecoveryStrategy::UseFallback),
        }
    }
}

// ── Unified application error ─────────────────────────────────────

/// Top-level error type for the entire tuiserial application.
///
/// Every library crate converts its native error type into an
/// `AppError` at the crate boundary.  The application layer
/// (`tuiserial-cli`) records `AppError`s through
/// `AppState::record_error()`, which both logs them persistently
/// and shows a user-facing notification.
///
/// # Naming convention
///
/// The field carrying the sub-kind is named `kind` (not `source`)
/// because thiserror v2 reserves `source` for the `Error::source()`
/// chain and requires those types to implement `std::error::Error`.
/// Only `AppError::Io` has `kind: std::io::Error` which *does*
/// implement `Error` and is annotated with `#[source]`.
#[derive(Error, Debug)]
pub enum AppError {
    /// Serial-port related error.
    #[error("Serial: {kind}")]
    Serial {
        kind: SerialErrorKind,
        ctx: ErrorContext,
    },

    /// Plugin-related error (load, runtime, panic, git, …).
    #[error("Plugin '{plugin}': {kind}")]
    Plugin {
        plugin: String,
        kind: PluginErrorKind,
        ctx: ErrorContext,
    },

    /// Configuration error.
    #[error("Config: {kind}")]
    Config {
        kind: ConfigErrorKind,
        ctx: ErrorContext,
    },

    /// General I/O error.
    #[error("I/O")]
    Io {
        #[source]
        kind: std::io::Error,
        ctx: ErrorContext,
    },

    /// Internal / unexpected error.
    #[error("Internal: {message}")]
    Internal {
        message: String,
        ctx: ErrorContext,
    },
}

impl AppError {
    /// Severity classification for this error.
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            AppError::Serial { kind, .. } => match kind {
                SerialErrorKind::PortOpen(_) => ErrorSeverity::Error,
                SerialErrorKind::NotConnected => ErrorSeverity::Warning,
                SerialErrorKind::Io(_) => ErrorSeverity::Warning,
                SerialErrorKind::InvalidHexLength | SerialErrorKind::ParseHex(_) => {
                    ErrorSeverity::Error
                }
            },
            AppError::Plugin { kind, .. } => match kind {
                PluginErrorKind::Panic { .. } => ErrorSeverity::Error,
                PluginErrorKind::Script(_) | PluginErrorKind::Runtime(_) => ErrorSeverity::Warning,
                PluginErrorKind::Io(_) => ErrorSeverity::Warning,
                PluginErrorKind::Git(_) => ErrorSeverity::Warning,
                PluginErrorKind::Timeout => ErrorSeverity::Error,
                PluginErrorKind::MaxRetriesExceeded => ErrorSeverity::Error,
            },
            AppError::Config { .. } => ErrorSeverity::Warning,
            AppError::Io { .. } => ErrorSeverity::Error,
            AppError::Internal { .. } => ErrorSeverity::Critical,
        }
    }

    /// Suggested recovery strategy.
    pub fn recovery(&self) -> RecoveryStrategy {
        self.ctx().recovery
    }

    /// Borrow the error context.
    pub fn ctx(&self) -> &ErrorContext {
        match self {
            AppError::Serial { ctx, .. }
            | AppError::Plugin { ctx, .. }
            | AppError::Config { ctx, .. }
            | AppError::Io { ctx, .. }
            | AppError::Internal { ctx, .. } => ctx,
        }
    }

    /// If this is a `Plugin` error, return the plugin name.
    pub fn plugin_name(&self) -> Option<&str> {
        match self {
            AppError::Plugin { plugin, .. } => Some(plugin),
            _ => None,
        }
    }

    /// Produce a concise, user-facing message suitable for the
    /// notification bar.
    pub fn to_user_message(&self) -> String {
        match self {
            AppError::Serial {
                kind: SerialErrorKind::NotConnected,
                ..
            } => "Serial port not connected".into(),
            AppError::Plugin {
                plugin,
                kind: PluginErrorKind::Panic { message, .. },
                ..
            } => format!("Plugin '{plugin}' crashed: {message}"),
            _ => self.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_ordering() {
        assert!(ErrorSeverity::Error > ErrorSeverity::Warning);
        assert!(ErrorSeverity::Critical > ErrorSeverity::Error);
        assert!(ErrorSeverity::Fatal > ErrorSeverity::Critical);
        assert!(ErrorSeverity::Info < ErrorSeverity::Warning);
    }

    #[test]
    fn test_core_error_backward_compat() {
        let ce = CoreError::ConfigDirNotFound;
        let ae: AppError = ce.into();
        assert!(matches!(ae, AppError::Config { .. }));
        assert_eq!(ae.severity(), ErrorSeverity::Warning);
    }

    #[test]
    fn test_app_error_severity_serial() {
        let err = AppError::Serial {
            kind: SerialErrorKind::NotConnected,
            ctx: ErrorContext::new("test", "read", RecoveryStrategy::Skip),
        };
        assert_eq!(err.severity(), ErrorSeverity::Warning);

        let err = AppError::Serial {
            kind: SerialErrorKind::PortOpen("denied".into()),
            ctx: ErrorContext::new("test", "open", RecoveryStrategy::Retry),
        };
        assert_eq!(err.severity(), ErrorSeverity::Error);
    }

    #[test]
    fn test_app_error_severity_plugin() {
        let err = AppError::Plugin {
            plugin: "test".into(),
            kind: PluginErrorKind::Panic {
                hook: "onRx".into(),
                message: "boom".into(),
            },
            ctx: ErrorContext::new("plugin", "rx pipeline", RecoveryStrategy::DisableComponent),
        };
        assert_eq!(err.severity(), ErrorSeverity::Error);

        let err = AppError::Plugin {
            plugin: "test".into(),
            kind: PluginErrorKind::Script("syntax error".into()),
            ctx: ErrorContext::new("plugin", "load", RecoveryStrategy::DisableComponent),
        };
        assert_eq!(err.severity(), ErrorSeverity::Warning);
    }

    #[test]
    fn test_plugin_name_extraction() {
        let err = AppError::Plugin {
            plugin: "my-plugin".into(),
            kind: PluginErrorKind::Script("oops".into()),
            ctx: ErrorContext::new("plugin", "load", RecoveryStrategy::DisableComponent),
        };
        assert_eq!(err.plugin_name(), Some("my-plugin"));

        let err = AppError::Serial {
            kind: SerialErrorKind::NotConnected,
            ctx: ErrorContext::new("test", "read", RecoveryStrategy::Skip),
        };
        assert_eq!(err.plugin_name(), None);
    }

    #[test]
    fn test_user_message_panic() {
        let err = AppError::Plugin {
            plugin: "test".into(),
            kind: PluginErrorKind::Panic {
                hook: "onRx".into(),
                message: "null pointer".into(),
            },
            ctx: ErrorContext::new("p", "rx", RecoveryStrategy::DisableComponent),
        };
        let msg = err.to_user_message();
        assert!(msg.contains("test"));
        assert!(msg.contains("null pointer"));
    }
}
