//! Git operations for plugin management.
//!
//! Thin wrappers around `std::process::Command` that call the system `git` binary.
//! All functions return `Result<..., PluginError>` with the `Git` variant on failure.

use std::path::Path;
use std::process::Command;

use crate::types::PluginError;

/// Check whether `git` is available on the system.
pub fn git_available() -> bool {
    Command::new("git")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Run git with the given args in an optional working directory.
fn git(args: &[&str], cwd: Option<&Path>) -> Result<String, PluginError> {
    let mut cmd = Command::new("git");
    cmd.args(args).stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped());
    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }
    let output = cmd.output().map_err(|e| {
        PluginError::Git(format!("failed to run git: {}", e))
    })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(PluginError::Git(stderr.trim().to_string()));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// `git clone <url> <target_dir>`
pub fn git_clone(url: &str, target: &Path) -> Result<(), PluginError> {
    let parent = target.parent().unwrap_or(Path::new("."));
    let name = target.file_name().unwrap();
    git(
        &["clone", url, name.to_str().unwrap()],
        Some(parent),
    )?;
    Ok(())
}

/// `git clone --filter=blob:none --sparse <url> <target_dir>`
///
/// Creates a partial clone that downloads only commit/tree metadata
/// without file contents. Combined with sparse-checkout, this allows
/// downloading only specific subdirectories on demand.
pub fn git_clone_sparse(url: &str, target: &Path) -> Result<(), PluginError> {
    let parent = target.parent().unwrap_or(Path::new("."));
    let name = target.file_name().unwrap();
    git(
        &[
            "clone",
            "--filter=blob:none",
            "--sparse",
            url,
            name.to_str().unwrap(),
        ],
        Some(parent),
    )?;
    Ok(())
}

/// `git sparse-checkout set <path>` in the given repo.
///
/// Configures the working tree to only contain files matching the
/// given path pattern. Requires a sparse-checkout–enabled clone.
pub fn git_sparse_checkout_set(repo_dir: &Path, path: &str) -> Result<(), PluginError> {
    git(&["sparse-checkout", "set", path], Some(repo_dir))?;
    Ok(())
}

/// `git sparse-checkout add <path>` — add a path to the existing set.
pub fn git_sparse_checkout_add(repo_dir: &Path, path: &str) -> Result<(), PluginError> {
    git(&["sparse-checkout", "add", path], Some(repo_dir))?;
    Ok(())
}

/// `git checkout` in the given repo (after sparse-checkout changes).
pub fn git_checkout(repo_dir: &Path) -> Result<(), PluginError> {
    git(&["checkout"], Some(repo_dir))?;
    Ok(())
}

/// `git ls-tree -d HEAD:` — list top-level directories.
///
/// Returns the names of top-level directories in the repo, which
/// correspond to available plugins in a monorepo registry.
pub fn git_list_top_dirs(repo_dir: &Path) -> Result<Vec<String>, PluginError> {
    let output = git(&["ls-tree", "-d", "HEAD:"], Some(repo_dir))?;
    let dirs: Vec<String> = output
        .lines()
        .filter_map(|line| {
            // Format: <mode> tree <hash>\t<name>
            let parts: Vec<&str> = line.split('\t').collect();
            parts.get(1).map(|s| s.to_string())
        })
        .collect();
    Ok(dirs)
}

/// List the contents of a tree object at `HEAD:<path>`.
///
/// Returns file and directory names in the given tree. This reads only
/// metadata — no blobs are downloaded even with `--filter=blob:none`.
pub fn git_ls_tree_path(repo_dir: &Path, path: &str) -> Result<Vec<String>, PluginError> {
    let output = git(
        &["ls-tree", &format!("HEAD:{}", path)],
        Some(repo_dir),
    )?;
    let names: Vec<String> = output
        .lines()
        .filter_map(|line| line.split('\t').nth(1).map(|s| s.to_string()))
        .collect();
    Ok(names)
}

/// Read a file from a specific path in the git tree without checking
/// it out (uses `git show HEAD:<path>`).
///
/// With `--filter=blob:none`, git will download the requested blob
/// on demand the first time it's accessed.
pub fn git_show_file(repo_dir: &Path, path: &str) -> Result<String, PluginError> {
    git(&["show", &format!("HEAD:{}", path)], Some(repo_dir))
}

/// `git fetch` in the given repo directory.
pub fn git_fetch(repo_dir: &Path) -> Result<(), PluginError> {
    git(&["fetch"], Some(repo_dir))?;
    Ok(())
}

/// Get the HEAD commit hash.
pub fn git_head_commit(repo_dir: &Path) -> Result<String, PluginError> {
    git(&["rev-parse", "HEAD"], Some(repo_dir))
}

/// Get the remote HEAD commit hash (`@{u}` = upstream branch).
pub fn git_remote_commit(repo_dir: &Path) -> Result<String, PluginError> {
    git(&["rev-parse", "@{u}"], Some(repo_dir))
}

/// Check whether the repo is behind its upstream.
/// Returns `true` if there are new commits on the remote.
pub fn git_is_behind(repo_dir: &Path) -> Result<bool, PluginError> {
    // `rev-list --count HEAD..@{u}` → number of commits remote is ahead
    let count = git(&["rev-list", "--count", "HEAD..@{u}"], Some(repo_dir))?;
    Ok(count.parse::<u32>().unwrap_or(0) > 0)
}

/// Fast-forward only pull. Safe — won't touch local changes.
pub fn git_pull_ff(repo_dir: &Path) -> Result<String, PluginError> {
    git(&["pull", "--ff-only"], Some(repo_dir))
}

/// Get the remote origin URL.
pub fn git_remote_url(repo_dir: &Path) -> Result<String, PluginError> {
    git(&["remote", "get-url", "origin"], Some(repo_dir))
}

/// Check if a directory is a git repository.
pub fn is_git_repo(dir: &Path) -> bool {
    dir.join(".git").exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_available() {
        // Git should be available in dev / CI environments
        let _ = git_available();
    }
}
