//! Core types for the tuiserial plugin system.
//!
//! Defines the data structures for plugin state, hook detection,
//! and return values from plugin function calls.

use serde::{Deserialize, Serialize};
use tuiserial_core::{NotificationLevel, SerialConfig};

/// Per-plugin mutable state shared between Rust and JS.
///
/// A single `PluginContext` is created for each loaded plugin and
/// mutated during hook calls. After a hook returns, the context
/// is inspected to determine the result.
#[derive(Debug, Clone)]
pub struct PluginContext {
    /// Human-readable plugin name (directory name)
    pub plugin_name: String,
    /// Snapshot of serial config at time of last hook call
    pub config: SerialConfig,
    /// Buffered log messages waiting to be flushed to AppState
    pub log_messages: Vec<(NotificationLevel, String)>,
}

impl PluginContext {
    pub fn new(plugin_name: String) -> Self {
        Self {
            plugin_name,
            config: SerialConfig::default(),
            log_messages: Vec::new(),
        }
    }

    pub fn update_config(&mut self, config: &SerialConfig) {
        self.config = config.clone();
    }
}

/// Bitmask-style detection of which hooks a plugin exports.
///
/// After executing the plugin script, the bootstrap checks
/// `typeof onLoad === 'function'` etc. for each hook.
#[derive(Debug, Clone, Default)]
pub struct PluginHooks {
    pub on_load: bool,
    pub on_unload: bool,
    pub on_connect: bool,
    pub on_disconnect: bool,
    pub on_rx: bool,
    pub on_tx: bool,
}

impl PluginHooks {
    /// True when the plugin has no hooks at all
    pub fn is_empty(&self) -> bool {
        !self.on_load
            && !self.on_unload
            && !self.on_connect
            && !self.on_disconnect
            && !self.on_rx
            && !self.on_tx
    }
}

/// Result of calling a plugin data hook (onRx / onTx).
#[derive(Debug, Clone)]
pub enum PluginResult {
    /// Hook returned null/undefined — pass data through unchanged
    PassThrough,
    /// Hook returned a modified byte array
    Modified(Vec<u8>),
    /// Hook returned an empty array or signalled suppression
    Suppressed,
    /// Hook threw an exception
    Error(String),
}

/// Lightweight plugin info for UI display.
#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub name: String,
    pub enabled: bool,
    pub hooks: PluginHooks,
    pub has_error: bool,
    pub error_message: Option<String>,
}

/// Errors that can occur during plugin operations.
#[derive(Debug)]
pub enum PluginError {
    Io(std::io::Error),
    Script(String),
    Runtime(String),
    Git(String),
}

impl std::fmt::Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginError::Io(e) => write!(f, "IO error: {}", e),
            PluginError::Script(e) => write!(f, "Script error: {}", e),
            PluginError::Runtime(e) => write!(f, "Runtime error: {}", e),
            PluginError::Git(e) => write!(f, "Git error: {}", e),
        }
    }
}

impl std::error::Error for PluginError {}

impl From<std::io::Error> for PluginError {
    fn from(e: std::io::Error) -> Self {
        PluginError::Io(e)
    }
}

// ── Plugin metadata & registry ──────────────────────────────────

/// Metadata stored in a plugin's `plugin.json` file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub name: String,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub repo: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
}
// RegistryEntry is defined in tuiserial-core so it can be used in AppState.
pub use tuiserial_core::RegistryEntry;

/// Status of a plugin relative to its git remote.
#[derive(Debug, Clone)]
pub struct PluginUpdateStatus {
    pub name: String,
    pub repo: String,
    pub current_commit: String,
    pub latest_commit: String,
    pub has_update: bool,
}
