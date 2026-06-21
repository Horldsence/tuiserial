//! Native JavaScript functions registered in each plugin's `Context`.
//!
//! These implement `tuiserial.require()`, `tuiserial.fs.read()`, and
//! `tuiserial.fs.readBinary()` by bridging back to Rust from the JS runtime.
//! Path resolution prevents directory traversal outside the plugins root.

use std::path::{Path, PathBuf};

use boa_engine::{
    Context, JsNativeError, JsResult, JsValue, Source, native_function::NativeFunction,
    object::builtins::JsArray, property::PropertyKey, string::JsString,
};

use crate::script;

/// Register all native JS functions (`__tuiserial_native_*__`) in the given context.
pub(crate) fn register_native_functions(context: &mut Context) {
    let realm = context.realm().clone();

    let key = make_interned_key(context, "__tuiserial_native_require__");
    context
        .register_global_property(
            key,
            NativeFunction::from_fn_ptr(native_require).to_js_function(&realm),
            Default::default(),
        )
        .expect("register __tuiserial_native_require__");

    let key = make_interned_key(context, "__tuiserial_native_fs_read__");
    context
        .register_global_property(
            key,
            NativeFunction::from_fn_ptr(native_fs_read).to_js_function(&realm),
            Default::default(),
        )
        .expect("register __tuiserial_native_fs_read__");

    let key = make_interned_key(context, "__tuiserial_native_fs_read_binary__");
    context
        .register_global_property(
            key,
            NativeFunction::from_fn_ptr(native_fs_read_binary).to_js_function(&realm),
            Default::default(),
        )
        .expect("register __tuiserial_native_fs_read_binary__");
}

/// Create a `PropertyKey` from a string, ensuring it is interned in the engine's string table.
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

    let full_path = resolve_module_path(&plugin_dir, &path).ok_or_else(|| {
        JsNativeError::typ().with_message(format!(
            "Path traversal blocked: '{}' is outside plugin directory",
            path
        ))
    })?;

    let source = std::fs::read_to_string(&full_path).map_err(|e| {
        JsNativeError::typ().with_message(format!("Cannot find module '{}': {}", path, e))
    })?;

    let js_source = script::strip_ts_annotations(&source);

    // Wrap in IIFE for module isolation — each required file gets its own scope.
    let wrapped = format!(
        "(function() {{\nvar exports = {{}};\n{}\nreturn exports;\n}})()",
        js_source
    );

    context.eval(Source::from_bytes(wrapped.as_bytes()))
}

fn native_fs_read(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let path = get_arg_string(args, context)?;
    let plugin_dir = get_plugin_dir(context)?;

    let full_path = resolve_fs_path(&plugin_dir, &path).ok_or_else(|| {
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

    let full_path = resolve_fs_path(&plugin_dir, &path).ok_or_else(|| {
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

/// Resolve a module path for `require()` — allows traversal up to the plugins root
/// (parent of `plugin_dir`) so that sibling directories can be reached.
fn resolve_module_path(plugin_dir: &Path, requested: &str) -> Option<PathBuf> {
    resolve_plugin_path(plugin_dir, requested, false)
}

/// Resolve a filesystem path for `tuiserial.fs.*` — restricts access to the plugin's
/// own directory, blocking sibling-directory access.
fn resolve_fs_path(plugin_dir: &Path, requested: &str) -> Option<PathBuf> {
    resolve_plugin_path(plugin_dir, requested, true)
}

/// Resolve a requested path against the plugin directory, blocking traversal escapes.
///
/// Supports normalised relative paths like `./utils.js`, `sub/dep.js`,
/// and `../shared/utils.js`. When `restrict_to_own` is true, access is limited to
/// the plugin's own directory; otherwise access is allowed up to the plugins root
/// (parent of `plugin_dir`).
fn resolve_plugin_path(
    plugin_dir: &Path,
    requested: &str,
    restrict_to_own: bool,
) -> Option<PathBuf> {
    let mut resolved = plugin_dir.to_path_buf();

    if requested.is_empty() || requested.starts_with('/') {
        return None;
    }

    for component in requested.split('/') {
        match component {
            "" | "." => continue,
            ".." => {
                if !resolved.pop() {
                    return None;
                }
            }
            _ => resolved.push(component),
        }
    }

    let boundary = if restrict_to_own {
        plugin_dir.to_path_buf()
    } else {
        plugin_dir.parent().unwrap_or(plugin_dir).to_path_buf()
    };

    if resolved.starts_with(&boundary) {
        Some(resolved)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resolve_path(plugin_dir: &Path, requested: &str) -> Option<PathBuf> {
        resolve_plugin_path(plugin_dir, requested, false)
    }

    #[test]
    fn test_resolve_simple() {
        let base = Path::new("/plugins/my-plugin");
        assert_eq!(
            resolve_path(base, "utils.js"),
            Some(PathBuf::from("/plugins/my-plugin/utils.js"))
        );
    }

    #[test]
    fn test_resolve_subdir() {
        let base = Path::new("/plugins/my-plugin");
        assert_eq!(
            resolve_path(base, "lib/utils.js"),
            Some(PathBuf::from("/plugins/my-plugin/lib/utils.js"))
        );
    }

    #[test]
    fn test_resolve_dot_slash() {
        let base = Path::new("/plugins/my-plugin");
        assert_eq!(
            resolve_path(base, "./utils.js"),
            Some(PathBuf::from("/plugins/my-plugin/utils.js"))
        );
    }

    #[test]
    fn test_resolve_parent_dir() {
        let base = Path::new("/plugins/my-plugin");
        assert_eq!(
            resolve_path(base, "../shared/utils.js"),
            Some(PathBuf::from("/plugins/shared/utils.js"))
        );
    }

    #[test]
    fn test_resolve_traversal_blocked() {
        let base = Path::new("/plugins/my-plugin");
        assert_eq!(resolve_path(base, "../../../etc/passwd"), None);
    }

    #[test]
    fn test_resolve_absolute_blocked() {
        let base = Path::new("/plugins/my-plugin");
        assert_eq!(resolve_path(base, "/etc/passwd"), None);
    }

    #[test]
    fn test_resolve_empty() {
        let base = Path::new("/plugins/my-plugin");
        assert_eq!(resolve_path(base, ""), None);
    }

    #[test]
    fn test_fs_path_restricted_to_own_dir() {
        let base = Path::new("/plugins/my-plugin");
        // fs.read/readBinary should NOT allow parent directory access
        assert_eq!(resolve_plugin_path(base, "../sibling/data.bin", true), None);
        // But same-directory access should work
        assert_eq!(
            resolve_plugin_path(base, "config.json", true),
            Some(PathBuf::from("/plugins/my-plugin/config.json"))
        );
    }
}
