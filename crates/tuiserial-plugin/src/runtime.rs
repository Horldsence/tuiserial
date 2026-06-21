//! Plugin runtime — wraps a Boa JS engine per plugin.
//!
//! Each plugin gets its own `boa_engine::Context` (isolated JS realm).
//! Native Rust functions are registered for `require()` and `tuiserial.fs`
//! to support multi-file plugins.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Instant;

use boa_engine::{Context, Source};
use tuiserial_core::NotificationLevel;

use crate::native::register_native_functions;
use crate::script::{self, drain_log_queue, make_config_json};
use crate::types::{PluginContext, PluginError, PluginHooks, PluginResult};

/// Maximum number of consecutive data-hook errors before the plugin
/// is permanently disabled (has_error = true).
pub(crate) const MAX_CONSECUTIVE_ERRORS: u32 = 3;

/// Backoff duration applied after a transient data-hook error.
/// The plugin is skipped during this window but automatically
/// re-enabled once the backoff expires.
pub(crate) const ERROR_BACKOFF_SECS: u64 = 5;

/// Maximum number of error history entries kept per plugin.
const MAX_ERROR_HISTORY: usize = 10;

/// JS bootstrap that sets up the `tuiserial` global object.
///
/// This is evaluated before the plugin's own source so that
/// `tuiserial.log.*`, `tuiserial.config.get()`, `tuiserial.require()`,
/// and `tuiserial.fs.*` are available in the global scope.
pub(crate) const BOOTSTRAP: &str = r#"
var __tuiserial_log_queue__ = [];
var __tuiserial_module_cache__ = {};

var tuiserial = {
  log: {
    info:    function(msg) { __tuiserial_log_queue__.push({l:0, m:msg}); },
    warn:    function(msg) { __tuiserial_log_queue__.push({l:1, m:msg}); },
    error:   function(msg) { __tuiserial_log_queue__.push({l:2, m:msg}); },
    success: function(msg) { __tuiserial_log_queue__.push({l:3, m:msg}); },
  },
  config: {
    get: function() { return JSON.parse(__tuiserial_config__); },
  },
  require: function(path) {
    if (__tuiserial_module_cache__.hasOwnProperty(path)) {
      return __tuiserial_module_cache__[path];
    }
    var exports = __tuiserial_native_require__(path);
    __tuiserial_module_cache__[path] = exports;
    return exports;
  },
  fs: {
    read:       function(path) { return __tuiserial_native_fs_read__(path); },
    readBinary: function(path) { return __tuiserial_native_fs_read_binary__(path); },
  },
};
"#;

pub struct PluginRuntime {
    pub name: String,
    pub source_path: PathBuf,
    pub plugin_dir: PathBuf,
    pub hooks: PluginHooks,
    /// Permanently disabled (load error or too many consecutive runtime errors).
    pub has_error: bool,
    pub error_message: Option<String>,
    /// Total error count since last load.
    pub error_count: u32,
    /// Errors in a row (reset on success).
    pub consecutive_errors: u32,
    /// If set, the plugin is temporarily skipped until this instant.
    pub disabled_until: Option<Instant>,
    /// Ring buffer of recent error messages for diagnostics.
    pub error_history: VecDeque<(Instant, String)>,
    context: Option<Context>,
    plugin_ctx: Rc<RefCell<PluginContext>>,
}

impl PluginRuntime {
    pub fn new(name: &str, source_path: PathBuf, plugin_dir: PathBuf) -> Result<Self, PluginError> {
        Ok(Self {
            name: name.to_string(),
            source_path,
            plugin_dir,
            hooks: PluginHooks::default(),
            has_error: false,
            error_message: None,
            error_count: 0,
            consecutive_errors: 0,
            disabled_until: None,
            error_history: VecDeque::new(),
            context: None,
            plugin_ctx: Rc::new(RefCell::new(PluginContext::new(name.to_string()))),
        })
    }

    pub fn load(&mut self) -> Result<(), PluginError> {
        self.has_error = false;
        self.error_message = None;

        let raw_source = std::fs::read_to_string(&self.source_path)?;
        let js_source = script::strip_ts_annotations(&raw_source);

        let mut context = Context::default();

        // Evaluate bootstrap (defines tuiserial global and queue)
        context
            .eval(Source::from_bytes(BOOTSTRAP.as_bytes()))
            .map_err(|e| PluginError::Runtime(format!("Bootstrap: {}", e)))?;

        // Store plugin dir in a JS variable so native functions can read it
        let dir_escaped = script::escape_js_string(&self.plugin_dir.to_string_lossy());
        let dir_init = format!("var __tuiserial_plugin_dir__ = '{}';", dir_escaped);
        context
            .eval(Source::from_bytes(dir_init.as_bytes()))
            .map_err(|e| PluginError::Runtime(format!("Dir init: {}", e)))?;

        // Init empty config
        context
            .eval(Source::from_bytes(b"var __tuiserial_config__ = '{}';"))
            .map_err(|e| PluginError::Runtime(format!("Config init: {}", e)))?;

        // Register native functions
        register_native_functions(&mut context);

        // Evaluate plugin source
        if let Err(e) = context.eval(Source::from_bytes(js_source.as_bytes())) {
            self.has_error = true;
            let msg = format!("{}", e);
            self.error_message = Some(msg.clone());
            return Err(PluginError::Script(msg));
        }

        self.hooks = script::detect_hooks(&mut context)?;
        self.context = Some(context);

        if self.hooks.on_load {
            let _ = self.call_lifecycle_hook("onLoad");
        }

        Ok(())
    }

    pub fn call_lifecycle_hook(&mut self, hook_name: &str) -> Result<(), PluginError> {
        let ctx = match &mut self.context {
            Some(c) => c,
            None => return Err(PluginError::Runtime("Plugin not loaded".into())),
        };

        let pctx = self.plugin_ctx.clone();
        let config_json = make_config_json(&pctx.borrow());
        let _ = ctx.eval(Source::from_bytes(config_json.as_bytes()));

        let code = format!(
            "if (typeof {} === 'function') {{ {}(); }}",
            hook_name, hook_name
        );
        ctx.eval(Source::from_bytes(code.as_bytes()))
            .map_err(|e| PluginError::Script(format!("{}", e)))?;

        drain_log_queue(ctx, &pctx);
        Ok(())
    }

    pub fn call_data_hook(&mut self, hook_name: &str, data: &[u8]) -> PluginResult {
        let ctx = match &mut self.context {
            Some(c) => c,
            None => return PluginResult::Error("Plugin not loaded".into()),
        };

        let pctx = self.plugin_ctx.clone();
        let config_json = make_config_json(&pctx.borrow());
        let _ = ctx.eval(Source::from_bytes(config_json.as_bytes()));

        let arr = script::build_js_array_literal(data);
        let code = format!(
            "var __d = {}; __tuiserial_result__ = (typeof {} === 'function') ? {}(__d) : null;",
            arr, hook_name, hook_name
        );

        if let Err(e) = ctx.eval(Source::from_bytes(code.as_bytes())) {
            drain_log_queue(ctx, &pctx);
            return PluginResult::Error(format!("{}", e));
        }

        let result = match ctx.eval(Source::from_bytes(b"JSON.stringify(__tuiserial_result__)")) {
            Ok(v) => v
                .as_string()
                .map(|s| s.to_std_string_escaped())
                .unwrap_or_default(),
            Err(e) => {
                drain_log_queue(ctx, &pctx);
                return PluginResult::Error(format!("{}", e));
            }
        };

        drain_log_queue(ctx, &pctx);
        script::parse_hook_result(&result)
    }

    pub fn unload(&mut self) {
        if self.hooks.on_unload {
            let _ = self.call_lifecycle_hook("onUnload");
        }
        self.context = None;
    }

    /// Reset all error counters and the backoff timer.
    ///
    /// Called when a plugin is manually reloaded or when it succeeds
    /// after a transient error.
    pub fn clear_errors(&mut self) {
        self.has_error = false;
        self.error_message = None;
        self.error_count = 0;
        self.consecutive_errors = 0;
        self.disabled_until = None;
        self.error_history.clear();
    }

    /// Record an error in the plugin's history ring buffer.
    pub(crate) fn record_error(&mut self, msg: String) {
        if self.error_history.len() >= MAX_ERROR_HISTORY {
            self.error_history.pop_front();
        }
        self.error_history.push_back((Instant::now(), msg));
    }

    pub fn drain_log_messages(&self) -> Vec<(NotificationLevel, String)> {
        self.plugin_ctx
            .borrow_mut()
            .log_messages
            .drain(..)
            .collect()
    }

    pub fn append_log(&self, level: NotificationLevel, msg: String) {
        self.plugin_ctx.borrow_mut().log_messages.push((level, msg));
    }

    pub fn update_config(&self, config: &tuiserial_core::SerialConfig) {
        self.plugin_ctx.borrow_mut().update_config(config);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn make_plugin_dir() -> (TempDir, PathBuf) {
        let tmp = TempDir::new().unwrap();
        let plugin_dir = tmp.path().join("my-plugin");
        std::fs::create_dir_all(&plugin_dir).unwrap();
        (tmp, plugin_dir)
    }

    fn write_file(dir: &std::path::Path, name: &str, content: &str) {
        let mut f = std::fs::File::create(dir.join(name)).unwrap();
        f.write_all(content.as_bytes()).unwrap();
    }

    #[test]
    fn test_require_sub_module() {
        let (_tmp, plugin_dir) = make_plugin_dir();
        write_file(
            &plugin_dir,
            "utils.js",
            "function add(a, b) { return a + b; }\nexports.add = add;",
        );
        write_file(
            &plugin_dir,
            "plugin.js",
            r#"
            var utils = tuiserial.require('utils.js');
            var result = utils.add(1, 2);
            "#,
        );

        let source_path = plugin_dir.join("plugin.js");
        let mut runtime = PluginRuntime::new("test", source_path, plugin_dir.clone()).unwrap();
        runtime.load().unwrap();

        let ctx = runtime.context.as_mut().unwrap();
        let val = ctx.eval(Source::from_bytes(b"utils.add(10, 20)")).unwrap();
        assert_eq!(val.as_number().unwrap(), 30.0);
    }

    #[test]
    fn test_require_caching() {
        let (_tmp, plugin_dir) = make_plugin_dir();
        write_file(
            &plugin_dir,
            "counter.js",
            "var count = (exports.count || 0) + 1;\nexports.count = count;",
        );
        write_file(
            &plugin_dir,
            "plugin.js",
            r#"
            var a = tuiserial.require('counter.js');
            var b = tuiserial.require('counter.js');
            "#,
        );

        let mut runtime =
            PluginRuntime::new("test", plugin_dir.join("plugin.js"), plugin_dir).unwrap();
        runtime.load().unwrap();

        let ctx = runtime.context.as_mut().unwrap();
        let val = ctx
            .eval(Source::from_bytes(b"a.count === b.count"))
            .unwrap();
        assert!(val.as_boolean().unwrap());
    }

    #[test]
    fn test_fs_read_text_file() {
        let (_tmp, plugin_dir) = make_plugin_dir();
        write_file(&plugin_dir, "config.json", r#"{"baud": 9600}"#);
        write_file(
            &plugin_dir,
            "plugin.js",
            "var cfg = tuiserial.fs.read('config.json');",
        );

        let mut runtime =
            PluginRuntime::new("test", plugin_dir.join("plugin.js"), plugin_dir).unwrap();
        runtime.load().unwrap();

        let ctx = runtime.context.as_mut().unwrap();
        let val = ctx.eval(Source::from_bytes(b"cfg")).unwrap();
        let s = val.as_string().unwrap().to_std_string_escaped();
        assert!(s.contains(r#""baud": 9600"#));
    }

    #[test]
    fn test_fs_read_binary() {
        let (_tmp, plugin_dir) = make_plugin_dir();
        std::fs::write(plugin_dir.join("data.bin"), [0x00, 0xFF, 0x42]).unwrap();
        write_file(
            &plugin_dir,
            "plugin.js",
            "var bytes = tuiserial.fs.readBinary('data.bin');",
        );

        let mut runtime =
            PluginRuntime::new("test", plugin_dir.join("plugin.js"), plugin_dir).unwrap();
        runtime.load().unwrap();

        let ctx = runtime.context.as_mut().unwrap();
        let val = ctx
            .eval(Source::from_bytes(b"JSON.stringify(bytes)"))
            .unwrap();
        assert_eq!(
            val.as_string().unwrap().to_std_string_escaped(),
            "[0,255,66]"
        );
    }

    #[test]
    fn test_require_traversal_blocked() {
        let (_tmp, plugin_dir) = make_plugin_dir();
        write_file(
            &plugin_dir,
            "plugin.js",
            "var bad = tuiserial.require('../../../etc/passwd');",
        );

        let mut runtime =
            PluginRuntime::new("test", plugin_dir.join("plugin.js"), plugin_dir).unwrap();
        let result = runtime.load();
        assert!(result.is_err());
    }
}
