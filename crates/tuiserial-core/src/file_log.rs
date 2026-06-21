//! Persistent file-log infrastructure.
//!
//! Provides helpers for discovering the log directory (XDG-compatible)
//! and rotating old log files.  The actual logging backend (`fern`)
//! lives in `tuiserial-cli` because it depends on the final binary.
//!
//! `dirs::config_dir()` is used so the path follows platform conventions:
//! - Linux:   `~/.config/tuiserial/log/`
//! - macOS:   `~/Library/Application Support/tuiserial/log/`

use std::fs;
use std::path::{Path, PathBuf};

/// Maximum size of `tuiserial.log` before it is rotated, in bytes.
const MAX_LOG_SIZE: u64 = 1024 * 1024; // 1 MiB

/// Return the log directory path, if the platform config directory is available.
///
/// The directory is **not** created by this function — callers must
/// `fs::create_dir_all` before writing.
pub fn log_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("tuiserial").join("log"))
}

/// Rotate the main log file if it exceeds [`MAX_LOG_SIZE`].
///
/// The old file is renamed to `tuiserial.{timestamp}.log` where
/// `timestamp` is the current Unix epoch in seconds.  Returns
/// `Ok(())` if no rotation was needed or if rotation succeeded.
pub fn rotate_log(log_dir: &Path) -> std::io::Result<()> {
    let log_path = log_dir.join("tuiserial.log");

    match fs::metadata(&log_path) {
        Ok(meta) if meta.len() > MAX_LOG_SIZE => {
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let rotated = log_dir.join(format!("tuiserial.{}.log", ts));
            fs::rename(&log_path, &rotated)?;
        }
        _ => { /* file doesn't exist yet or is within limit */ }
    }

    Ok(())
}
