//! TypeScript/JavaScript source processing and hook utilities.
//!
//! Provides the TS→JS transpiler (type annotation stripping), hook detection,
//! and helpers for translating between JS and Rust data representations.

use std::cell::RefCell;
use std::rc::Rc;

use boa_engine::{Context, Source};
use serde_json;
use tuiserial_core::NotificationLevel;

use crate::types::{PluginContext, PluginError, PluginHooks, PluginResult};

/// Escape a string for safe embedding in a JS single-quoted string literal.
pub(crate) fn escape_js_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

/// Build the JS assignment that updates `__tuiserial_config__` from a `PluginContext`.
pub(crate) fn make_config_json(ctx: &PluginContext) -> String {
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

/// Drain the JS-side `__tuiserial_log_queue__` into the plugin's Rust log buffer.
pub(crate) fn drain_log_queue(ctx: &mut Context, pctx: &Rc<RefCell<PluginContext>>) {
    let code = "var q = __tuiserial_log_queue__; __tuiserial_log_queue__ = []; JSON.stringify(q);";
    if let Ok(v) = ctx.eval(Source::from_bytes(code.as_bytes())) {
        let json = v
            .as_string()
            .map(|s| s.to_std_string_escaped())
            .unwrap_or_default();
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

/// Detect which lifecycle/data hooks (`onLoad`, `onRx`, …) are defined in the JS context.
pub(crate) fn detect_hooks(context: &mut Context) -> Result<PluginHooks, PluginError> {
    let hook_names = [
        "onLoad",
        "onUnload",
        "onConnect",
        "onDisconnect",
        "onRx",
        "onTx",
    ];
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

/// Build a JS array literal from a byte slice, e.g. `[72,101,108]`.
pub(crate) fn build_js_array_literal(data: &[u8]) -> String {
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

/// Parse a JSON-serialised hook return value into a `PluginResult`.
///
/// - `null` → `PassThrough`
/// - `[]` (empty array) → `Suppressed`
/// - `[1,2,3]` → `Modified(vec![1,2,3])`
/// - anything else → `PassThrough`
pub(crate) fn parse_hook_result(json: &str) -> PluginResult {
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
                let bytes: Vec<u8> = arr
                    .iter()
                    .filter_map(|v| v.as_u64().map(|n| n as u8))
                    .collect();
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

/// Strip TypeScript type annotations from source, producing plain JavaScript.
///
/// This is a lightweight transpiler — it does not parse the full TS grammar.
/// It handles:
/// - `: type` annotations after identifiers
/// - `export ` keyword prefix
/// - `as Type` cast expressions
/// - `interface Name { ... }` declarations
/// - `type Name = ...;` declarations
/// - Generic type parameters (`<T>`, `<T extends U>`)
/// - String literals and comments are preserved
///
/// The result is valid ES5/ES6 JavaScript suitable for evaluation by Boa.
pub(crate) fn strip_ts_annotations(source: &str) -> String {
    let mut result = String::with_capacity(source.len());
    let chars: Vec<char> = source.chars().collect();
    let len = chars.len();
    // Map character index → byte offset in `source`, so we can safely slice
    // `source` when `i` is a character index. Needed because multi-byte UTF-8
    // characters (e.g. `—`) make byte offsets larger than character indices.
    let char_byte_positions: Vec<usize> = source
        .char_indices()
        .map(|(pos, _)| pos)
        .chain(std::iter::once(source.len()))
        .collect();
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

        if c == 'e' && source[char_byte_positions[i]..].starts_with("export ") {
            i += "export ".len();
            continue;
        }

        // Strip `: type` annotations. Skip when `:` follows a string literal
        // (e.g. `"key": value` in object literals — the `:` is a property
        // separator, not a type annotation).
        if c == ':' && i > 0 {
            let prev = chars[i - 1];
            if prev == '\'' || prev == '"' || prev == '`' {
                result.push(c);
                i += 1;
                continue;
            }
            let remaining = &source[char_byte_positions[i + 1]..];
            if remaining.starts_with(' ') {
                let mut j = i + 1;
                let mut depth: i32 = 0;
                let mut seen_type_content = false;
                while j < len {
                    let nc = chars[j];
                    // Track bracket depth so we skip nested {…}, […], (…)
                    if nc == '{' || nc == '[' || nc == '(' {
                        depth += 1;
                        seen_type_content = true;
                        j += 1;
                        continue;
                    }
                    if depth > 0 && (nc == '}' || nc == ']' || nc == ')') {
                        depth -= 1;
                        j += 1;
                        continue;
                    }
                    if depth == 0 {
                        match nc {
                            // End-of-type delimiters
                            ',' | '=' | ';' | '\n' | ')' => break,
                            // Closing brackets at top-level: consume and break
                            // (the bracket belongs to the type, e.g. `{…}` type literal)
                            '}' | ']' => {
                                j += 1;
                                break;
                            }
                            // A `{` after we've already seen type content means
                            // the type ended and a new block started (e.g. function body).
                            '{' if seen_type_content => break,
                            // Whitespace at depth 0: peek ahead — if a break
                            // character follows, stop now so the formatting space
                            // is preserved in the output. Otherwise consume it
                            // (it's whitespace inside the type, e.g. `|` spacing).
                            // `{` only terminates when we've already seen type
                            // content (same as the explicit `{` rule above).
                            c if c.is_whitespace() => {
                                let mut peek = j + 1;
                                while peek < len && chars[peek].is_whitespace() {
                                    peek += 1;
                                }
                                if peek < len {
                                    let pc = chars[peek];
                                    if matches!(pc, ',' | '=' | ';' | '\n' | ')') {
                                        break;
                                    }
                                    if pc == '{' && seen_type_content {
                                        break;
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    if nc == '<' {
                        // Skip generic type parameters, e.g. `Array<…>`
                        let mut gdepth = 1;
                        j += 1;
                        while j < len && gdepth > 0 {
                            if chars[j] == '<' {
                                gdepth += 1;
                            }
                            if chars[j] == '>' {
                                gdepth -= 1;
                            }
                            j += 1;
                        }
                        seen_type_content = true;
                        continue;
                    }
                    if !nc.is_whitespace() {
                        seen_type_content = true;
                    }
                    j += 1;
                }
                i = j;
                continue;
            }
        }

        if c == ' ' && source[char_byte_positions[i]..].starts_with(" as ") {
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

        if c == 'i' && source[char_byte_positions[i]..].starts_with("interface ") {
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

        if c == 't' && source[char_byte_positions[i]..].starts_with("type ") {
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
        assert_eq!(js.trim(), "function foo(x, y) { return true; }");
    }

    #[test]
    fn test_strip_preserves_string_literals() {
        let ts = r#"tuiserial.log.warn(" non-JSONRPC: " + line.slice(0, 80));"#;
        let js = strip_ts_annotations(ts);
        assert_eq!(js.trim(), ts.trim());

        let ts2 = "var x: string = 'hello: world';";
        let js2 = strip_ts_annotations(ts2);
        assert_eq!(js2.trim(), "var x = 'hello: world';");

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
        assert_eq!(js.trim(), "/* type: foo */ var x = 1;");
    }

    #[test]
    fn test_parse_hook_null() {
        assert!(matches!(
            parse_hook_result("null"),
            PluginResult::PassThrough
        ));
    }

    #[test]
    fn test_parse_hook_empty_array() {
        assert!(matches!(parse_hook_result("[]"), PluginResult::Suppressed));
    }

    #[test]
    fn test_parse_hook_modified() {
        let result = parse_hook_result("[72,101,108,108,111]");
        assert!(
            matches!(result, PluginResult::Modified(ref v) if v == &vec![72, 101, 108, 108, 111])
        );
    }

    #[test]
    fn test_strip_with_unicode() {
        let ts = "var msg = \"loaded — OK\";\nfunction onLoad(): void { }";
        let js = strip_ts_annotations(ts);
        assert!(!js.contains(": void"));
        assert!(js.contains("loaded — OK"));
    }

    #[test]
    fn test_strip_bridge_plugin_style() {
        // Simulates the real mcp-bridge plugin.ts that crashed
        let ts = r#"var msg = tuiserial.log.success("MCP Bridge loaded — NDJSON/JSON-RPC 2.0 over serial");"#;
        let js = strip_ts_annotations(ts);
        assert_eq!(js.trim(), ts.trim());
    }

    #[test]
    fn test_strip_jsonrpc_with_unicode_and_interface() {
        // jsonrpc.ts uses interface, type annotations, index signatures, and △ chars
        let ts = r#"interface JsonRpcMessage {
    jsonrpc: string;
    method?: string;
    error?: { code: number; message: string; data?: any };
    id?: string | number | null;
}
var MCP_METHODS: { [key: string]: string } = {
    "initialize": "init",
    "notifications/tools/list_changed": "ntf/tools△",
};
function parse(line: string): JsonRpcMessage | null {
    var msg: any;
    return null;
}"#;
        let js = strip_ts_annotations(ts);
        // Interface should be removed
        assert!(!js.contains("interface"));
        // Type annotations should be removed
        assert!(!js.contains("JsonRpcMessage"));
        // Unicode chars in string literals preserved
        assert!(js.contains("ntf/tools△"), "expected unicode △ preserved");
        // Function signature simplified
        assert!(!js.contains(": string"));
        assert!(!js.contains(": any"));
        assert!(!js.contains("| null"));
    }

    #[test]
    fn test_strip_index_signature() {
        // Index signatures like { [key: string]: string } are tricky
        let ts = r#"var MCP_METHODS: { [key: string]: string } = {
    "tools/list": "tools/list",
};"#;
        let js = strip_ts_annotations(ts);
        assert!(!js.contains(": {"), "should not contain ': {{'");
        assert!(
            js.contains("MCP_METHODS = {"),
            "expected 'MCP_METHODS = {{', got: {}",
            js
        );
        assert!(
            js.contains("\"tools/list\": \"tools/list\""),
            "expected property preserved"
        );
    }

    #[test]
    fn test_build_js_array() {
        assert_eq!(build_js_array_literal(&[72, 101, 108]), "[72,101,108]");
        assert_eq!(build_js_array_literal(&[]), "[]");
    }
}
