use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use colored::Colorize;

use super::{HOOK_MARKER_END, HOOK_MARKER_START};
use crate::error::{Error, Result};
use crate::git;
use crate::registry;
use crate::report::Report;
const HOOK_CONTENT: &str = "\n# Record commit activity\ngit-recap update\n";

/// Install the post-commit hook, register the repo, and initialise the report.
///
/// # Errors
///
/// Returns an error if the hook is already installed or if any file/git
/// operation fails.
pub fn run() -> Result<()> {
    let sha = git::initial_commit_sha()?;
    let root = git::repo_root()?;
    let name = git::repo_name()?;

    // Install hook
    install_hook()?;

    // Register in registry
    registry::register(&sha, &root)?;

    // Lazy-init report
    let mut report = Report::load_or_init(root, name, sha)?;
    report.activity.last_touched = chrono::Local::now().fixed_offset();
    report.save()?;

    println!(
        "{} hook installed, repo registered",
        "Done.".green().bold()
    );

    Ok(())
}

/// Install the git-recap post-commit hook.
fn install_hook() -> Result<()> {
    let git_dir = git::git_dir()?;
    let hook_path = git_dir.join("hooks").join("post-commit");

    // Check if already installed
    if hook_path.exists() {
        let content = fs::read_to_string(&hook_path).map_err(|e| Error::ReadFile {
            path: hook_path.clone(),
            source: e,
        })?;
        if content.contains(HOOK_MARKER_START) {
            return Err(Error::HookAlreadyInstalled("post-commit".to_string()));
        }
    }

    // Ensure hooks directory exists
    if let Some(parent) = hook_path.parent() {
        fs::create_dir_all(parent).map_err(|e| Error::CreateDir {
            path: parent.to_path_buf(),
            source: e,
        })?;
    }

    // Read existing content or create new
    let existing = if hook_path.exists() {
        fs::read_to_string(&hook_path).map_err(|e| Error::ReadFile {
            path: hook_path.clone(),
            source: e,
        })?
    } else {
        "#!/bin/sh\n".to_string()
    };

    // Append our hook
    let new_content =
        format!("{existing}\n{HOOK_MARKER_START}{HOOK_CONTENT}{HOOK_MARKER_END}\n");

    fs::write(&hook_path, &new_content).map_err(|e| Error::WriteFile {
        path: hook_path.clone(),
        source: e,
    })?;

    // Make executable (Unix only — Windows doesn't need this)
    #[cfg(unix)]
    {
        let mut perms = fs::metadata(&hook_path)
            .map_err(|e| Error::ReadFile {
                path: hook_path.clone(),
                source: e,
            })?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&hook_path, perms).map_err(|e| Error::WriteFile {
            path: hook_path.clone(),
            source: e,
        })?;
    }

    Ok(())
}
