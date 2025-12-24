//! Notification types and functionality
//!
//! This module defines notification messages shown to users with different
//! severity levels and automatic expiration.

use std::time::Instant;

/// Notification level for user messages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationLevel {
    Info,
    Warning,
    Error,
    Success,
}

/// A notification message shown to the user
#[derive(Debug, Clone)]
pub struct Notification {
    pub message: String,
    pub level: NotificationLevel,
    pub created_at: Instant,
    pub duration_ms: u64,
}

impl Notification {
    /// Create a new notification with the specified level
    pub fn new(message: String, level: NotificationLevel) -> Self {
        Self {
            message,
            level,
            created_at: Instant::now(),
            duration_ms: 3000, // Default 3 seconds
        }
    }

    /// Create an info notification
    pub fn info(message: String) -> Self {
        Self::new(message, NotificationLevel::Info)
    }

    /// Create a warning notification
    pub fn warning(message: String) -> Self {
        Self::new(message, NotificationLevel::Warning)
    }

    /// Create an error notification
    pub fn error(message: String) -> Self {
        Self::new(message, NotificationLevel::Error)
    }

    /// Create a success notification
    pub fn success(message: String) -> Self {
        Self::new(message, NotificationLevel::Success)
    }

    /// Check if the notification has expired
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed().as_millis() as u64 > self.duration_ms
    }
}
