use std::path::PathBuf;
use std::process::Command;

use crate::error::{Error, Result};

/// Pre-resolved repository context to avoid redundant git subprocess calls.
pub struct RepoContext {
    pub root: PathBuf,
    pub sha: String,
    pub name: String,
}

impl RepoContext {
    /// Resolve the current repository context in a single pass.
    ///
    /// # Errors
    ///
    /// Returns an error if not inside a git repository or if there are no commits.
    pub fn resolve() -> Result<Self> {
        if !is_in_repo() {
            return Err(Error::NotInGitRepo);
        }
        let root = PathBuf::from(run(&["rev-parse", "--show-toplevel"])?);
        let sha_output =
            run(&["rev-list", "--max-parents=0", "HEAD"]).map_err(|_| Error::NoCommits)?;
        let sha = sha_output.lines().next().unwrap_or(&sha_output).to_string();
        let name = root.file_name().map_or_else(
            || "unknown".to_string(),
            |n| n.to_string_lossy().to_string(),
        );
        Ok(Self { root, sha, name })
    }
}

/// Run a git command and return stdout on success.
///
/// # Errors
///
/// Returns an error if the git command fails or cannot be executed.
pub fn run(args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .map_err(|e| Error::GitCommandFailed(format!("failed to execute git: {e}")))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(Error::GitCommandFailed(stderr))
    }
}

/// Check if we're inside a git repository.
#[must_use]
pub fn is_in_repo() -> bool {
    run(&["rev-parse", "--is-inside-work-tree"]).is_ok_and(|s| s == "true")
}

/// Get the root directory of the current git repository.
///
/// # Errors
///
/// Returns an error if not inside a git repository.
pub fn repo_root() -> Result<PathBuf> {
    if !is_in_repo() {
        return Err(Error::NotInGitRepo);
    }
    let root = run(&["rev-parse", "--show-toplevel"])?;
    Ok(PathBuf::from(root))
}

/// Get the .git directory of the current repository.
///
/// # Errors
///
/// Returns an error if not inside a git repository.
pub fn git_dir() -> Result<PathBuf> {
    if !is_in_repo() {
        return Err(Error::NotInGitRepo);
    }
    let dir = run(&["rev-parse", "--git-dir"])?;
    Ok(PathBuf::from(dir))
}

/// Get the initial commit SHA of the repository (project identifier).
///
/// # Errors
///
/// Returns an error if not inside a git repository or there are no commits.
pub fn initial_commit_sha() -> Result<String> {
    if !is_in_repo() {
        return Err(Error::NotInGitRepo);
    }
    let sha = run(&["rev-list", "--max-parents=0", "HEAD"]).map_err(|_| Error::NoCommits)?;

    // Take first line if multiple roots exist (rare but possible)
    Ok(sha.lines().next().unwrap_or(&sha).to_string())
}

/// Derive the repo name from the repo root directory.
///
/// # Errors
///
/// Returns an error if the repo root cannot be determined.
pub fn repo_name() -> Result<String> {
    let root = repo_root()?;
    Ok(root.file_name().map_or_else(
        || "unknown".to_string(),
        |n| n.to_string_lossy().to_string(),
    ))
}
