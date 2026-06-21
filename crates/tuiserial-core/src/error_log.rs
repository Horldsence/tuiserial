//! Persistent error log with deduplication.
//!
//! The error log stores up to `MAX_ERROR_LOG_ENTRIES` entries in a
//! ring buffer.  Consecutive identical errors are folded together by
//! incrementing a counter rather than creating duplicate entries,
//! which prevents log spam from high-frequency errors (e.g. serial
//! read failures in a tight loop).

use std::collections::VecDeque;

use crate::error::{AppError, ErrorSeverity};

/// Maximum number of distinct error entries kept in the log.
const MAX_ERROR_LOG_ENTRIES: usize = 200;

/// A single entry in the error log.
///
/// Consecutive occurrences of the same error message are collapsed
/// into one entry with `count > 1`.
#[derive(Debug)]
pub struct ErrorLogEntry {
    /// The error that occurred.
    pub error: AppError,
    /// Maximum severity observed across deduplicated occurrences.
    pub severity: ErrorSeverity,
    /// How many times this error occurred consecutively.
    pub count: usize,
}

/// Ring-buffer of recent application errors.
///
/// # Deduplication
///
/// When `push()` is called with an error whose `to_string()` matches
/// the most recent entry, the count is incremented and the severity
/// is raised to the maximum of the two, rather than adding a new
/// entry.  This keeps the log readable even when transient errors
/// fire at high frequency.
#[derive(Debug)]
pub struct ErrorLog {
    pub entries: VecDeque<ErrorLogEntry>,
}

impl Default for ErrorLog {
    fn default() -> Self {
        Self::new()
    }
}

impl ErrorLog {
    /// Create an empty error log.
    pub fn new() -> Self {
        Self {
            entries: VecDeque::new(),
        }
    }

    /// Record an error.
    ///
    /// If the error's display representation matches the most recent
    /// entry, that entry's `count` is incremented.  Otherwise a new
    /// entry is pushed (evicting the oldest entry if the log is full).
    pub fn push(&mut self, error: AppError) {
        let severity = error.severity();

        // Dedup: fold into the last entry if the message matches.
        if let Some(last) = self.entries.back_mut() {
            if last.error.to_string() == error.to_string() {
                last.count += 1;
                last.severity = last.severity.max(severity);
                return;
            }
        }

        // Evict oldest if at capacity.
        if self.entries.len() >= MAX_ERROR_LOG_ENTRIES {
            self.entries.pop_front();
        }

        self.entries.push_back(ErrorLogEntry {
            error,
            severity,
            count: 1,
        });
    }

    /// Remove all entries.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Number of stored entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the log is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Count entries with severity >= `Error`.
    pub fn error_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|e| e.severity >= ErrorSeverity::Error)
            .count()
    }

    /// Count entries with severity >= `Warning`.
    pub fn warning_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|e| e.severity >= ErrorSeverity::Warning)
            .count()
    }

    /// Count entries with severity >= `Critical`.
    pub fn critical_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|e| e.severity >= ErrorSeverity::Critical)
            .count()
    }

    /// Return a compact summary string for the status bar,
    /// e.g. `"E:3 W:1"`.
    pub fn summary(&self) -> String {
        let errs = self.error_count();
        let warns = self.warning_count();
        let crits = self.critical_count();
        if crits > 0 {
            format!("C:{crits} E:{errs}")
        } else if errs > 0 {
            format!("E:{errs}")
        } else if warns > 0 {
            format!("W:{warns}")
        } else {
            String::new()
        }
    }

    /// Return the most recent entry with severity >= `ErrorSeverity::Error`,
    /// if any.
    pub fn most_recent_error(&self) -> Option<&ErrorLogEntry> {
        self.entries
            .iter()
            .rfind(|e| e.severity >= ErrorSeverity::Error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{ErrorContext, RecoveryStrategy, SerialErrorKind};

    fn make_error(msg: &str) -> AppError {
        AppError::Serial {
            kind: SerialErrorKind::Io(msg.into()),
            ctx: ErrorContext::new("test", msg, RecoveryStrategy::Skip),
        }
    }

    #[test]
    fn test_basic_push() {
        let mut log = ErrorLog::new();
        log.push(make_error("first"));
        assert_eq!(log.len(), 1);
        assert_eq!(log.entries[0].count, 1);
    }

    #[test]
    fn test_dedup() {
        let mut log = ErrorLog::new();
        log.push(make_error("e1"));
        log.push(make_error("e1"));
        log.push(make_error("e1"));
        assert_eq!(log.len(), 1);
        assert_eq!(log.entries[0].count, 3);
    }

    #[test]
    fn test_no_dedup_different() {
        let mut log = ErrorLog::new();
        log.push(make_error("e1"));
        log.push(make_error("e2"));
        assert_eq!(log.len(), 2);
    }

    #[test]
    fn test_dedup_interleaved() {
        let mut log = ErrorLog::new();
        log.push(make_error("e1"));
        log.push(make_error("e2"));
        log.push(make_error("e1")); // different from last → new entry
        assert_eq!(log.len(), 3);
    }

    #[test]
    fn test_clear() {
        let mut log = ErrorLog::new();
        log.push(make_error("e1"));
        log.clear();
        assert!(log.is_empty());
    }

    #[test]
    fn test_summary() {
        let mut log = ErrorLog::new();
        assert_eq!(log.summary(), "");

        use crate::error::PluginErrorKind;
        log.push(AppError::Plugin {
            plugin: "p1".into(),
            kind: PluginErrorKind::Panic {
                hook: "onRx".into(),
                message: "boom".into(),
            },
            ctx: ErrorContext::new("p", "rx", RecoveryStrategy::DisableComponent),
        });
        // Panic → Error severity
        assert_eq!(log.summary(), "E:1");
    }

    #[test]
    fn test_most_recent_error() {
        let mut log = ErrorLog::new();
        assert!(log.most_recent_error().is_none());

        log.push(make_error("warn")); // SerialErrorKind::Io → Warning
        assert!(log.most_recent_error().is_none()); // Warning < Error

        log.push(AppError::Internal {
            message: "bad".into(),
            ctx: ErrorContext::new("t", "test", RecoveryStrategy::None),
        });
        assert!(log.most_recent_error().is_some());
    }
}
