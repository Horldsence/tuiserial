//! Plugin management operations — install, update, remove, and registry.
//!
//! These methods on `PluginManager` handle the lifecycle of individual plugins:
//! cloning from git, installing from the registry cache, checking for updates,
//! and removing plugins.

use std::path::{Path, PathBuf};

use crate::git;
use crate::manager::PluginManager;
use crate::registry;
use crate::types::{PluginError, PluginUpdateStatus, RegistryEntry};

impl PluginManager {
    /// Clone (or fetch) a GitHub repository into a directory.
    ///
    /// Returns the plugin name (derived from the repo URL).
    pub fn install_plugin(&self, repo_url: &str) -> Result<String, PluginError> {
        let name = repo_name_from_url(repo_url);
        let target = self.plugin_dir.join(&name);

        if target.exists() {
            return Err(PluginError::Git(format!(
                "plugin directory '{}' already exists",
                name
            )));
        }

        git::git_clone(repo_url, &target)?;
        Ok(name)
    }

    /// Return the path to the registry cache directory.
    pub fn registry_cache_dir(&self) -> PathBuf {
        self.config_dir.join("registry-cache")
    }

    /// Ensure the registry monorepo is cloned or up-to-date in the cache.
    pub fn fetch_registry(&self) -> Result<(), PluginError> {
        let cache = self.registry_cache_dir();

        if cache.join(".git").exists() {
            git::git_fetch(&cache)?;
            git::git_pull_ff(&cache)?;
        } else {
            let parent = cache.parent().unwrap_or(Path::new("."));
            std::fs::create_dir_all(parent)?;
            if cache.exists() {
                std::fs::remove_dir_all(&cache)?;
            }
            git::git_clone_sparse(registry::DEFAULT_REGISTRY_REPO, &cache)?;
        }

        Ok(())
    }

    /// Get the list of available plugins from the registry.
    pub fn get_registry(&self) -> Result<Vec<RegistryEntry>, PluginError> {
        self.fetch_registry()?;
        let cache = self.registry_cache_dir();
        Ok(registry::scan_registry_git(&cache))
    }

    /// Install a plugin by copying it from the registry cache.
    pub fn install_plugin_from_cache(&self, name: &str) -> Result<(), PluginError> {
        let target = self.plugin_dir.join(name);
        if target.exists() {
            return Err(PluginError::Git(format!(
                "plugin directory '{}' already exists",
                name
            )));
        }

        let cache = self.registry_cache_dir();
        if !cache.join(".git").exists() {
            return Err(PluginError::Git(
                "registry cache not found — fetch it first".into(),
            ));
        }

        git::git_sparse_checkout_set(&cache, name)?;
        git::git_checkout(&cache)?;

        let source = cache.join(name);
        if !source.exists() {
            return Err(PluginError::Git(format!(
                "plugin '{}' not found in registry",
                name
            )));
        }

        copy_dir_recursive(&source, &target)?;
        Ok(())
    }

    /// Check all installed plugins for updates from their git remotes.
    pub fn check_updates(&self) -> Result<Vec<PluginUpdateStatus>, PluginError> {
        let mut results = Vec::new();

        for entry in std::fs::read_dir(&self.plugin_dir)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let dir_name = path.file_name().unwrap().to_string_lossy();
            if dir_name == "disabled" || !git::is_git_repo(&path) {
                continue;
            }

            let repo = match git::git_remote_url(&path) {
                Ok(r) => r,
                Err(_) => continue,
            };

            if let Err(_e) = git::git_fetch(&path) {
                results.push(PluginUpdateStatus {
                    name: dir_name.to_string(),
                    repo,
                    current_commit: String::new(),
                    latest_commit: String::new(),
                    has_update: false,
                });
                continue;
            }

            let current = git::git_head_commit(&path).unwrap_or_default();
            let latest = git::git_remote_commit(&path).unwrap_or_default();
            let behind = git::git_is_behind(&path).unwrap_or(false);

            results.push(PluginUpdateStatus {
                name: dir_name.to_string(),
                repo,
                current_commit: short_hash(&current),
                latest_commit: short_hash(&latest),
                has_update: behind,
            });
        }

        Ok(results)
    }

    /// Update a single plugin via `git pull --ff-only`.
    pub fn update_plugin(&self, name: &str) -> Result<(), PluginError> {
        let dir = self.plugin_dir.join(name);
        if !dir.exists() {
            return Err(PluginError::Git(format!("plugin '{}' not found", name)));
        }
        if !git::is_git_repo(&dir) {
            return Err(PluginError::Git(format!(
                "'{}' is not a git repository",
                name
            )));
        }
        git::git_fetch(&dir)?;
        git::git_pull_ff(&dir)?;
        Ok(())
    }

    /// Update all plugins that have git remotes.
    ///
    /// Returns `(updated_count, errors)`.
    pub fn update_all(&self) -> (usize, Vec<String>) {
        let mut updated = 0usize;
        let mut errors = Vec::new();

        let entries = match std::fs::read_dir(&self.plugin_dir) {
            Ok(e) => e,
            Err(e) => {
                errors.push(format!("failed to read plugin dir: {}", e));
                return (updated, errors);
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let name = path.file_name().unwrap().to_string_lossy();
            if name == "disabled" || !git::is_git_repo(&path) {
                continue;
            }

            match self.update_plugin(&name) {
                Ok(()) => updated += 1,
                Err(e) => errors.push(format!("{}: {}", name, e)),
            }
        }

        (updated, errors)
    }

    /// Remove a plugin by deleting its directory.
    pub fn remove_plugin(&self, name: &str) -> Result<(), PluginError> {
        let dir = self.plugin_dir.join(name);
        if !dir.exists() {
            return Err(PluginError::Git(format!("plugin '{}' not found", name)));
        }
        std::fs::remove_dir_all(&dir)?;
        Ok(())
    }
}

// ── Helpers ─────────────────────────────────────────────────────────

/// Extract a human-readable plugin name from a GitHub URL.
fn repo_name_from_url(url: &str) -> String {
    url.trim_end_matches('/')
        .trim_end_matches(".git")
        .rsplit('/')
        .next()
        .unwrap_or("unknown-plugin")
        .to_string()
}

/// Recursively copy a directory from `src` to `dst`.
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), std::io::Error> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

/// Truncate a commit hash to 7 characters for display.
fn short_hash(hash: &str) -> String {
    if hash.len() > 7 {
        hash[..7].to_string()
    } else {
        hash.to_string()
    }
}
