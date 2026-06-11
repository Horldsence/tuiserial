//! Plugin system for tuiserial.
//!
//! This crate provides a JavaScript-based plugin engine using the
//! [boa_engine](https://crates.io/crates/boa_engine) pure-Rust JS runtime.
//!
//! ## Plugin format
//!
//! Plugins are directories under `~/.config/tuiserial/plugins/<name>/`
//! containing a `plugin.ts` or `plugin.js` file as the entry point.
//!
//! ## Multi-file plugins
//!
//! Plugins can split code across multiple files:
//! - `tuiserial.require('./lib.js')` — load and evaluate another JS/TS file
//!   from the plugin directory. Returns the module's `exports` object.
//!   Modules are cached: requiring the same path twice returns the cached exports.
//! - `tuiserial.fs.read('config.json')` — read a file as a UTF-8 string
//! - `tuiserial.fs.readBinary('data.bin')` — read a file as a `number[]`
//!
//! Path traversal outside the plugin directory is blocked for security.
//!
//! ## API
//!
//! Plugins define global functions (all optional):
//! - `onLoad()` — called when plugin is loaded
//! - `onUnload()` — called when plugin is unloaded
//! - `onConnect()` — called when serial port connects
//! - `onDisconnect()` — called when serial port disconnects
//! - `onRx(data)` — receive hook, `data` is `number[]`, return `number[] | null`
//! - `onTx(data)` — transmit hook, same signature as onRx
//!
//! The global `tuiserial` object provides:
//! - `tuiserial.log.info(msg)` / `.warn(msg)` / `.error(msg)` / `.success(msg)`
//! - `tuiserial.config.get()` → `{port, baudRate, dataBits, parity, stopBits, flowControl}`
//! - `tuiserial.require(path)` → module exports
//! - `tuiserial.fs.read(path)` → string
//! - `tuiserial.fs.readBinary(path)` → number[]

pub mod git;
pub mod manager;
pub mod registry;
pub mod runtime;
pub mod types;

pub use manager::PluginManager;
pub use types::{
    PluginError, PluginHooks, PluginInfo, PluginMetadata, PluginResult, PluginUpdateStatus,
    RegistryEntry,
};
