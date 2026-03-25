use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use colored::Colorize;

use super::{HOOK_MARKER_END, HOOK_MARKER_START};
use crate::error::{Error, Result};
use crate::git;
use crate::registry::Registry;
use crate::report::Report;

const HOOK_CONTENT: &str = "\n# Record commit activity\ngit-recap this\n";

/// Install hook and register repo.
///
/// # Errors
///
/// Returns an error if hook installation or file I/O fails.
pub fn install(hook_name: &str) -> Result<()> {
    let ctx = git::RepoContext::resolve()?;

    install_hook(hook_name)?;

    let mut registry = Registry::load()?;
    registry.register(&ctx.sha, &ctx.root);
    registry.save()?;

    let mut report = Report::load_or_init(ctx.root, ctx.name, ctx.sha)?;
    report.activity.last_touched = chrono::Local::now().fixed_offset();
    report.save()?;

    println!(
        "{} {} hook installed, repo registered",
        "Done.".green().bold(),
        hook_name.cyan()
    );

    Ok(())
}

/// Remove hook and deregister repo.
///
/// # Errors
///
/// Returns an error if hook removal or file I/O fails.
pub fn uninstall(hook_name: &str) -> Result<()> {
    let ctx = git::RepoContext::resolve()?;

    uninstall_hook(hook_name)?;

    let mut registry = Registry::load()?;
    registry.deregister(&ctx.sha);
    registry.save()?;

    println!(
        "{} {} hook removed, repo deregistered",
        "Done.".green().bold(),
        hook_name.cyan()
    );

    Ok(())
}

fn install_hook(hook_name: &str) -> Result<()> {
    let git_dir = git::git_dir()?;
    let hook_path = git_dir.join("hooks").join(hook_name);

    // Check if already installed
    if hook_path.exists() {
        let content = fs::read_to_string(&hook_path).map_err(|e| Error::ReadFile {
            path: hook_path.clone(),
            source: e,
        })?;
        if content.contains(HOOK_MARKER_START) {
            return Err(Error::HookAlreadyInstalled(hook_name.to_string()));
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
    let new_content = format!("{existing}\n{HOOK_MARKER_START}{HOOK_CONTENT}{HOOK_MARKER_END}\n");

    fs::write(&hook_path, &new_content).map_err(|e| Error::WriteFile {
        path: hook_path.clone(),
        source: e,
    })?;

    // Make executable (Unix only)
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

fn uninstall_hook(hook_name: &str) -> Result<()> {
    let git_dir = git::git_dir()?;
    let hook_path = git_dir.join("hooks").join(hook_name);

    if !hook_path.exists() {
        return Err(Error::HookNotInstalled(hook_name.to_string()));
    }

    let content = fs::read_to_string(&hook_path).map_err(|e| Error::ReadFile {
        path: hook_path.clone(),
        source: e,
    })?;

    if !content.contains(HOOK_MARKER_START) {
        return Err(Error::HookNotInstalled(hook_name.to_string()));
    }

    let mut new_lines = Vec::new();
    let mut in_our_section = false;

    for line in content.lines() {
        if line.contains(HOOK_MARKER_START) {
            in_our_section = true;
            continue;
        }
        if line.contains(HOOK_MARKER_END) {
            in_our_section = false;
            continue;
        }
        if !in_our_section {
            new_lines.push(line);
        }
    }

    let new_content = new_lines.join("\n");

    let trimmed = new_content.trim();
    if trimmed.is_empty() || trimmed == "#!/bin/sh" || trimmed == "#!/bin/bash" {
        fs::remove_file(&hook_path).map_err(|e| Error::WriteFile {
            path: hook_path.clone(),
            source: e,
        })?;
    } else {
        fs::write(&hook_path, new_content).map_err(|e| Error::WriteFile {
            path: hook_path.clone(),
            source: e,
        })?;
    }

    Ok(())
}
