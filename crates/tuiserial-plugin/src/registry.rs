//! Plugin registry — monorepo-based plugin discovery.
//!
//! The default registry is a single GitHub repository where each
//! top-level folder is a plugin. Discovery uses `git ls-tree` so it
//! works with sparse / partial clones without downloading any blobs.

use std::path::Path;

use crate::git;
use crate::types::RegistryEntry;

/// Default plugin registry repository.
pub const DEFAULT_REGISTRY_REPO: &str = "https://github.com/Horldsence/tuiserial-plugin";

/// Scan the registry cache for available plugins using filesystem.
///
/// This is used after a full checkout of the registry. Each subdirectory
/// containing a `plugin.ts` or `plugin.js` file is treated as a plugin.
pub fn scan_registry_cache(cache_dir: &Path) -> Vec<RegistryEntry> {
    let mut entries = Vec::new();

    let dirs = match std::fs::read_dir(cache_dir) {
        Ok(d) => d,
        Err(_) => return entries,
    };

    for entry in dirs.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let dir_name = path.file_name().unwrap().to_string_lossy().to_string();
        if dir_name.starts_with('.') {
            continue;
        }

        if !has_plugin_entry(&path) {
            continue;
        }

        let metadata = read_plugin_json(&path);
        entries.push(entry_from_dir(dir_name, metadata));
    }

    entries.sort_by(|a, b| a.name.cmp(&b.name));
    entries
}

/// Scan the registry cache for available plugins using git metadata.
///
/// This works with sparse / partial clones (`--filter=blob:none`)
/// because it only reads tree objects via `git ls-tree`. No blobs
/// are downloaded during discovery.
pub fn scan_registry_git(cache_dir: &Path) -> Vec<RegistryEntry> {
    let mut entries = Vec::new();

    let dirs = match git::git_list_top_dirs(cache_dir) {
        Ok(d) => d,
        Err(_) => return entries,
    };

    for dir_name in dirs {
        if dir_name.starts_with('.') {
            continue;
        }

        // Check if this dir contains plugin.ts / plugin.js via ls-tree
        let contents = match git::git_ls_tree_path(cache_dir, &dir_name) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let has_ep = contents.iter().any(|f| f == "plugin.ts" || f == "plugin.js");
        if !has_ep {
            continue;
        }

        // Try to read plugin.json via git show (downloads blob on demand)
        let metadata = read_plugin_json_git(cache_dir, &dir_name);
        entries.push(entry_from_dir(dir_name, metadata));
    }

    entries.sort_by(|a, b| a.name.cmp(&b.name));
    entries
}

// ── helpers ──────────────────────────────────────────────────────

#[derive(serde::Deserialize, Clone)]
struct PluginJson {
    name: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    author: Option<String>,
}

fn entry_from_dir(dir_name: String, metadata: Option<PluginJson>) -> RegistryEntry {
    let name = metadata
        .as_ref()
        .and_then(|m| m.name.clone())
        .unwrap_or(dir_name);
    let description = metadata.as_ref().and_then(|m| m.description.clone());
    let author = metadata.as_ref().and_then(|m| m.author.clone());
    RegistryEntry {
        name,
        description,
        author,
    }
}

fn has_plugin_entry(dir: &Path) -> bool {
    dir.join("plugin.ts").exists() || dir.join("plugin.js").exists()
}

fn read_plugin_json(dir: &Path) -> Option<PluginJson> {
    let bytes = std::fs::read(dir.join("plugin.json")).ok()?;
    serde_json::from_slice(&bytes).ok()
}

fn read_plugin_json_git(cache_dir: &Path, dir_name: &str) -> Option<PluginJson> {
    let path = format!("{}/plugin.json", dir_name);
    let content = git::git_show_file(cache_dir, &path).ok()?;
    serde_json::from_str(&content).ok()
}

/// Parse a remote registry JSON into entries (for future extensibility).
pub fn parse_registry(json: &str) -> Result<Vec<RegistryEntry>, serde_json::Error> {
    #[derive(serde::Deserialize)]
    struct RegistryFile {
        plugins: Vec<RegistryEntry>,
    }

    let registry: RegistryFile = serde_json::from_str(json)?;
    Ok(registry.plugins)
}
