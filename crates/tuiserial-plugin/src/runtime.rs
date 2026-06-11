//! Plugin runtime — wraps a Boa JS engine per plugin.
//!
//! Each plugin gets its own `boa_engine::Context` (isolated JS realm).
//! Native Rust functions are registered for `require()` and `tuiserial.fs`
//! to support multi-file plugins.

use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::cell::RefCell;

use boa_engine::{
    native_function::NativeFunction,
    object::builtins::JsArray,
    property::PropertyKey,
    string::JsString,
    Context, JsNativeError, JsResult, JsValue, Source,
};
use tuiserial_core::NotificationLevel;

use crate::types::{PluginContext, PluginError, PluginHooks, PluginResult};

const BOOTSTRAP: &str = r#"
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
    pub has_error: bool,
    pub error_message: Option<String>,
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
            context: None,
            plugin_ctx: Rc::new(RefCell::new(PluginContext::new(name.to_string()))),
        })
    }

    pub fn load(&mut self) -> Result<(), PluginError> {
        self.has_error = false;
        self.error_message = None;

        let raw_source = std::fs::read_to_string(&self.source_path)?;
        let js_source = strip_ts_annotations(&raw_source);

        let mut context = Context::default();

        // Evaluate bootstrap (defines tuiserial global and queue)
        context
            .eval(Source::from_bytes(BOOTSTRAP.as_bytes()))
            .map_err(|e| PluginError::Runtime(format!("Bootstrap: {}", e)))?;

        // Store plugin dir in a JS variable so native functions can read it
        let dir_escaped = escape_js_string(&self.plugin_dir.to_string_lossy());
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

        self.hooks = detect_hooks(&mut context)?;
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

        let arr = build_js_array_literal(data);
        let code = format!(
            "var __d = {}; __tuiserial_result__ = (typeof {} === 'function') ? {}(__d) : null;",
            arr, hook_name, hook_name
        );

        if let Err(e) = ctx.eval(Source::from_bytes(code.as_bytes())) {
            drain_log_queue(ctx, &pctx);
            return PluginResult::Error(format!("{}", e));
        }

        let result = match ctx.eval(Source::from_bytes(b"JSON.stringify(__tuiserial_result__)")) {
            Ok(v) => v.as_string().map(|s| s.to_std_string_escaped()).unwrap_or_default(),
            Err(e) => {
                drain_log_queue(ctx, &pctx);
                return PluginResult::Error(format!("{}", e));
            }
        };

        drain_log_queue(ctx, &pctx);
        parse_hook_result(&result)
    }

    pub fn unload(&mut self) {
        if self.hooks.on_unload {
            let _ = self.call_lifecycle_hook("onUnload");
        }
        self.context = None;
    }

    pub fn drain_log_messages(&self) -> Vec<(NotificationLevel, String)> {
        self.plugin_ctx.borrow_mut().log_messages.drain(..).collect()
    }

    pub fn append_log(&self, level: NotificationLevel, msg: String) {
        self.plugin_ctx.borrow_mut().log_messages.push((level, msg));
    }

    pub fn update_config(&self, config: &tuiserial_core::SerialConfig) {
        self.plugin_ctx.borrow_mut().update_config(config);
    }
}

// ── Native functions (function pointers — no captures) ──────────

fn register_native_functions(context: &mut Context) {
    let realm = context.realm().clone();

    // __tuiserial_native_require__(path) → exports object
    let key = make_interned_key(context, "__tuiserial_native_require__");
    context
        .register_global_property(
            key,
            NativeFunction::from_fn_ptr(native_require).to_js_function(&realm),
            Default::default(),
        )
        .expect("register __tuiserial_native_require__");

    // __tuiserial_native_fs_read__(path) → string
    let key = make_interned_key(context, "__tuiserial_native_fs_read__");
    context
        .register_global_property(
            key,
            NativeFunction::from_fn_ptr(native_fs_read).to_js_function(&realm),
            Default::default(),
        )
        .expect("register __tuiserial_native_fs_read__");

    // __tuiserial_native_fs_read_binary__(path) → number[]
    let key = make_interned_key(context, "__tuiserial_native_fs_read_binary__");
    context
        .register_global_property(
            key,
            NativeFunction::from_fn_ptr(native_fs_read_binary).to_js_function(&realm),
            Default::default(),
        )
        .expect("register __tuiserial_native_fs_read_binary__");
}

/// Create a PropertyKey from a string, ensuring it is interned in the engine's string table.
///
/// `JsString::from(&str)` creates a raw string outside the interner, which causes
/// property lookups to miss. We eval a JS string literal instead, which guarantees
/// the resulting string is interned.
fn make_interned_key(context: &mut Context, name: &str) -> PropertyKey {
    let script = format!("'{}'", name.replace('\\', "\\\\").replace('\'', "\\'"));
    let val = context
        .eval(Source::from_bytes(script.as_bytes()))
        .expect("intern key string");
    PropertyKey::from(val.as_string().expect("key should be a string"))
}

fn get_arg_string(args: &[JsValue], context: &mut Context) -> JsResult<String> {
    match args.first() {
        Some(v) => v.to_string(context).map(|s| s.to_std_string_escaped()),
        None => Ok(String::new()),
    }
}

fn native_require(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let path = get_arg_string(args, context)?;
    let plugin_dir = get_plugin_dir(context)?;

    let full_path = resolve_plugin_path(&plugin_dir, &path).ok_or_else(|| {
        JsNativeError::typ().with_message(format!(
            "Path traversal blocked: '{}' is outside plugin directory",
            path
        ))
    })?;

    let source = std::fs::read_to_string(&full_path).map_err(|e| {
        JsNativeError::typ().with_message(format!("Cannot find module '{}': {}", path, e))
    })?;

    let js_source = strip_ts_annotations(&source);

    // Wrap in IIFE for module isolation — each required file gets its own scope.
    // The IIFE returns an exports object that require() caches and returns.
    let wrapped = format!(
        "(function() {{\nvar exports = {{}};\n{}\nreturn exports;\n}})()",
        js_source
    );

    context.eval(Source::from_bytes(wrapped.as_bytes()))
}

fn native_fs_read(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let path = get_arg_string(args, context)?;
    let plugin_dir = get_plugin_dir(context)?;

    let full_path = resolve_plugin_path(&plugin_dir, &path).ok_or_else(|| {
        JsNativeError::typ().with_message(format!(
            "Path traversal blocked: '{}' is outside plugin directory",
            path
        ))
    })?;

    let content = std::fs::read_to_string(&full_path).map_err(|e| {
        JsNativeError::typ().with_message(format!("Cannot read file '{}': {}", path, e))
    })?;

    Ok(JsValue::new(JsString::from(content.as_str())))
}

fn native_fs_read_binary(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let path = get_arg_string(args, context)?;
    let plugin_dir = get_plugin_dir(context)?;

    let full_path = resolve_plugin_path(&plugin_dir, &path).ok_or_else(|| {
        JsNativeError::typ().with_message(format!(
            "Path traversal blocked: '{}' is outside plugin directory",
            path
        ))
    })?;

    let bytes = std::fs::read(&full_path).map_err(|e| {
        JsNativeError::typ().with_message(format!("Cannot read file '{}': {}", path, e))
    })?;

    let arr = JsArray::from_iter(bytes.iter().map(|b| JsValue::new(*b as f64)), context);
    Ok(arr.into())
}

/// Read `__tuiserial_plugin_dir__` from the JS global scope.
fn get_plugin_dir(context: &mut Context) -> JsResult<PathBuf> {
    let val = context.eval(Source::from_bytes(b"__tuiserial_plugin_dir__"))?;
    let dir = val
        .as_string()
        .map(|s| s.to_std_string_escaped())
        .unwrap_or_default();
    Ok(PathBuf::from(dir))
}

/// Resolve a requested path against the plugin directory, blocking traversal escapes.
///
/// Supports normalised relative paths like `./utils.js`, `sub/dep.js`,
/// and `../shared/utils.js`. Access is allowed within the plugins root
/// (parent of `plugin_dir`) so that sibling directories can be reached,
/// but traversal above the plugins root is blocked.
fn resolve_plugin_path(plugin_dir: &Path, requested: &str) -> Option<PathBuf> {
    let mut resolved = plugin_dir.to_path_buf();

    // If path is absolute-ish or empty, reject
    if requested.is_empty() || requested.starts_with('/') {
        return None;
    }

    for component in requested.split('/') {
        match component {
            "" | "." => continue,
            ".." => {
                if !resolved.pop() {
                    return None; // popped past root
                }
            }
            _ => resolved.push(component),
        }
    }

    // Allow access up to the plugins root (parent of plugin_dir),
    // but block traversal above it.
    let boundary = plugin_dir.parent().unwrap_or(plugin_dir);
    if resolved.starts_with(boundary) {
        Some(resolved)
    } else {
        None
    }
}

// ── Free functions (no self borrow) ──────────────────────────

fn escape_js_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

fn make_config_json(ctx: &PluginContext) -> String {
    let json = serde_json::json!({
        "port": ctx.config.port,
        "baudRate": ctx.config.baud_rate,
        "dataBits": ctx.config.data_bits,
        "parity": format!("{:?}", ctx.config.parity),
        "stopBits": ctx.config.stop_bits,
        "flowControl": format!("{:?}", ctx.config.flow_control),
    });
    let escaped = json.to_string().replace('\\', "\\\\").replace('\'', "\\'");
    format!("__tuiserial_config__ = '{}';", escaped)
}

fn drain_log_queue(ctx: &mut Context, pctx: &Rc<RefCell<PluginContext>>) {
    let code = "var q = __tuiserial_log_queue__; __tuiserial_log_queue__ = []; JSON.stringify(q);";
    if let Ok(v) = ctx.eval(Source::from_bytes(code.as_bytes())) {
        let json = v.as_string().map(|s| s.to_std_string_escaped()).unwrap_or_default();
        if json == "[]" || json.is_empty() {
            return;
        }
        if let Ok(entries) = serde_json::from_str::<Vec<serde_json::Value>>(&json) {
            let mut ctx = pctx.borrow_mut();
            for entry in entries {
                let level = match entry["l"].as_u64().unwrap_or(0) {
                    0 => NotificationLevel::Info,
                    1 => NotificationLevel::Warning,
                    2 => NotificationLevel::Error,
                    _ => NotificationLevel::Success,
                };
                let msg = entry["m"].as_str().unwrap_or("").to_string();
                ctx.log_messages.push((level, msg));
            }
        }
    }
}

fn detect_hooks(context: &mut Context) -> Result<PluginHooks, PluginError> {
    let hook_names = ["onLoad", "onUnload", "onConnect", "onDisconnect", "onRx", "onTx"];
    let mut hooks = PluginHooks::default();

    for name in &hook_names {
        let code = format!("typeof {} === 'function'", name);
        let source = Source::from_bytes(code.as_bytes());
        let is_fn = match context.eval(source) {
            Ok(val) => val.as_boolean().unwrap_or(false),
            Err(_) => false,
        };

        match *name {
            "onLoad" => hooks.on_load = is_fn,
            "onUnload" => hooks.on_unload = is_fn,
            "onConnect" => hooks.on_connect = is_fn,
            "onDisconnect" => hooks.on_disconnect = is_fn,
            "onRx" => hooks.on_rx = is_fn,
            "onTx" => hooks.on_tx = is_fn,
            _ => {}
        }
    }

    Ok(hooks)
}

fn build_js_array_literal(data: &[u8]) -> String {
    if data.is_empty() {
        return "[]".to_string();
    }
    format!(
        "[{}]",
        data.iter()
            .map(|b| b.to_string())
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn parse_hook_result(json: &str) -> PluginResult {
    if json == "null" || json.is_empty() {
        return PluginResult::PassThrough;
    }

    let parsed: serde_json::Value = match serde_json::from_str(json) {
        Ok(v) => v,
        Err(_) => return PluginResult::PassThrough,
    };

    match parsed {
        serde_json::Value::Null => PluginResult::PassThrough,
        serde_json::Value::Array(arr) => {
            if arr.is_empty() {
                PluginResult::Suppressed
            } else {
                let bytes: Vec<u8> =
                    arr.iter().filter_map(|v| v.as_u64().map(|n| n as u8)).collect();
                if bytes.len() != arr.len() {
                    PluginResult::Error("Array contains non-numeric values".into())
                } else {
                    PluginResult::Modified(bytes)
                }
            }
        }
        _ => PluginResult::PassThrough,
    }
}

fn strip_ts_annotations(source: &str) -> String {
    let mut result = String::with_capacity(source.len());
    let chars: Vec<char> = source.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let c = chars[i];

        // Skip string literals (single-quoted, double-quoted, backtick)
        if c == '\'' || c == '"' || c == '`' {
            let quote = c;
            result.push(c);
            i += 1;
            while i < len {
                let nc = chars[i];
                if nc == '\\' {
                    result.push(nc);
                    i += 1;
                    if i < len {
                        result.push(chars[i]);
                        i += 1;
                    }
                    continue;
                }
                result.push(nc);
                i += 1;
                if nc == quote {
                    break;
                }
            }
            continue;
        }

        // Skip line comments
        if c == '/' && i + 1 < len && chars[i + 1] == '/' {
            while i < len && chars[i] != '\n' {
                result.push(chars[i]);
                i += 1;
            }
            continue;
        }

        // Skip block comments
        if c == '/' && i + 1 < len && chars[i + 1] == '*' {
            result.push('/');
            result.push('*');
            i += 2;
            while i + 1 < len {
                if chars[i] == '*' && chars[i + 1] == '/' {
                    result.push('*');
                    result.push('/');
                    i += 2;
                    break;
                }
                result.push(chars[i]);
                i += 1;
            }
            continue;
        }

        if c == 'e' && source[i..].starts_with("export ") {
            i += "export ".len();
            continue;
        }

        if c == ':' {
            let remaining = &source[i + 1..];
            if remaining.starts_with(' ') {
                let mut j = i + 1;
                while j < len {
                    let nc = chars[j];
                    if nc == ',' || nc == ')' || nc == '{' || nc == '=' || nc == ';' || nc == '\n' {
                        break;
                    }
                    if nc == '<' {
                        let mut depth = 1;
                        j += 1;
                        while j < len && depth > 0 {
                            if chars[j] == '<' {
                                depth += 1;
                            }
                            if chars[j] == '>' {
                                depth -= 1;
                            }
                            j += 1;
                        }
                        continue;
                    }
                    if nc == '[' {
                        j += 1;
                        while j < len && chars[j] != ']' {
                            j += 1;
                        }
                        j += 1;
                        continue;
                    }
                    j += 1;
                }
                i = j;
                continue;
            }
        }

        if c == ' ' && source[i..].starts_with(" as ") {
            let mut j = i + 4;
            while j < len {
                match chars[j] {
                    ' ' | ';' | ')' | ',' | '\n' | '\r' => break,
                    _ => j += 1,
                }
            }
            i = j;
            continue;
        }

        if c == 'i' && source[i..].starts_with("interface ") {
            while i < len && chars[i] != '\n' && chars[i] != '{' {
                i += 1;
            }
            if i < len && chars[i] == '{' {
                let mut depth = 1;
                i += 1;
                while i < len && depth > 0 {
                    if chars[i] == '{' {
                        depth += 1;
                    }
                    if chars[i] == '}' {
                        depth -= 1;
                    }
                    i += 1;
                }
            }
            continue;
        }

        if c == 't' && source[i..].starts_with("type ") {
            while i < len && chars[i] != '\n' && chars[i] != ';' {
                i += 1;
            }
            if i < len && chars[i] == ';' {
                i += 1;
            }
            continue;
        }

        result.push(c);
        i += 1;
    }

    result
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

    fn write_file(dir: &Path, name: &str, content: &str) {
        let mut f = std::fs::File::create(dir.join(name)).unwrap();
        f.write_all(content.as_bytes()).unwrap();
    }

    #[test]
    fn test_strip_export_function() {
        let ts = "export function onRx(data: Uint8Array): Uint8Array | null {\n  return data;\n}";
        let js = strip_ts_annotations(ts);
        assert!(!js.contains("export"));
        assert!(!js.contains("Uint8Array"));
        assert!(js.contains("function onRx(data)"));
    }

    #[test]
    fn test_strip_interface() {
        let ts = "interface Config { port: string; }\nfunction onLoad() { }";
        let js = strip_ts_annotations(ts);
        assert!(!js.contains("interface"));
        assert!(js.contains("function onLoad()"));
    }

    #[test]
    fn test_simple_js_passthrough() {
        let js = "function onLoad() { tuiserial.log.info('loaded'); }";
        let result = strip_ts_annotations(js);
        assert_eq!(result.trim(), js);
    }

    #[test]
    fn test_strip_type_annotation() {
        let ts = "function foo(x: number, y: string): boolean { return true; }";
        let js = strip_ts_annotations(ts);
        assert_eq!(js.trim(), "function foo(x, y){ return true; }");
    }

    #[test]
    fn test_strip_preserves_string_literals() {
        // Colon inside a double-quoted string must not be stripped
        let ts = r#"tuiserial.log.warn(" non-JSONRPC: " + line.slice(0, 80));"#;
        let js = strip_ts_annotations(ts);
        assert_eq!(js.trim(), ts.trim());

        // Colon inside a single-quoted string must not be stripped
        let ts2 = "var x: string = 'hello: world';";
        let js2 = strip_ts_annotations(ts2);
        // Type annotation stripping removes ": string " incl. trailing space
        assert_eq!(js2.trim(), "var x= 'hello: world';");

        // Colon inside a template literal must not be stripped
        let ts3 = "var msg = `result: ${value}`;";
        let js3 = strip_ts_annotations(ts3);
        assert_eq!(js3.trim(), ts3.trim());
    }

    #[test]
    fn test_strip_preserves_line_comments() {
        let ts = "// note: this is a comment\nvar x = 1;";
        let js = strip_ts_annotations(ts);
        assert_eq!(js.trim(), ts.trim());
    }

    #[test]
    fn test_strip_preserves_block_comments() {
        let ts = "/* type: foo */ var x: number = 1;";
        let js = strip_ts_annotations(ts);
        assert_eq!(js.trim(), "/* type: foo */ var x= 1;");
    }

    #[test]
    fn test_parse_hook_null() {
        assert!(matches!(parse_hook_result("null"), PluginResult::PassThrough));
    }

    #[test]
    fn test_parse_hook_empty_array() {
        assert!(matches!(parse_hook_result("[]"), PluginResult::Suppressed));
    }

    #[test]
    fn test_parse_hook_modified() {
        let result = parse_hook_result("[72,101,108,108,111]");
        assert!(matches!(result, PluginResult::Modified(ref v) if v == &vec![72, 101, 108, 108, 111]));
    }

    #[test]
    fn test_build_js_array() {
        assert_eq!(build_js_array_literal(&[72, 101, 108]), "[72,101,108]");
        assert_eq!(build_js_array_literal(&[]), "[]");
    }

    // ── Path resolution tests ────────────────────────────────

    #[test]
    fn test_resolve_simple() {
        let base = Path::new("/plugins/my-plugin");
        assert_eq!(
            resolve_plugin_path(base, "utils.js"),
            Some(PathBuf::from("/plugins/my-plugin/utils.js"))
        );
    }

    #[test]
    fn test_resolve_subdir() {
        let base = Path::new("/plugins/my-plugin");
        assert_eq!(
            resolve_plugin_path(base, "lib/utils.js"),
            Some(PathBuf::from("/plugins/my-plugin/lib/utils.js"))
        );
    }

    #[test]
    fn test_resolve_dot_slash() {
        let base = Path::new("/plugins/my-plugin");
        assert_eq!(
            resolve_plugin_path(base, "./utils.js"),
            Some(PathBuf::from("/plugins/my-plugin/utils.js"))
        );
    }

    #[test]
    fn test_resolve_parent_dir() {
        let base = Path::new("/plugins/my-plugin");
        assert_eq!(
            resolve_plugin_path(base, "../shared/utils.js"),
            Some(PathBuf::from("/plugins/shared/utils.js"))
        );
    }

    #[test]
    fn test_resolve_traversal_blocked() {
        let base = Path::new("/plugins/my-plugin");
        assert_eq!(resolve_plugin_path(base, "../../../etc/passwd"), None);
    }

    #[test]
    fn test_resolve_absolute_blocked() {
        let base = Path::new("/plugins/my-plugin");
        assert_eq!(resolve_plugin_path(base, "/etc/passwd"), None);
    }

    #[test]
    fn test_resolve_empty() {
        let base = Path::new("/plugins/my-plugin");
        assert_eq!(resolve_plugin_path(base, ""), None);
    }

    // ── Multi-file integration tests ──────────────────────────

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

        // Verify the add function is available in the context
        let ctx = runtime.context.as_mut().unwrap();
        let val = ctx
            .eval(Source::from_bytes(b"utils.add(10, 20)"))
            .unwrap();
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
        // a and b should be the same cached module, so a.count === b.count
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
        let val = ctx
            .eval(Source::from_bytes(b"cfg"))
            .unwrap();
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
        // Should fail because path traversal is blocked
        let result = runtime.load();
        assert!(result.is_err());
    }
}
