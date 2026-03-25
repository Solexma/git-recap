use std::fs;

use colored::Colorize;

use super::{HOOK_MARKER_END, HOOK_MARKER_START};
use crate::error::{Error, Result};
use crate::git;
use crate::registry::Registry;

/// Remove hook and deregister repo.
///
/// # Errors
///
/// Returns an error if hook removal or file I/O fails.
pub fn run() -> Result<()> {
    let ctx = git::RepoContext::resolve()?;

    uninstall_hook()?;

    let mut registry = Registry::load()?;
    registry.deregister(&ctx.sha);
    registry.save()?;

    println!("{} hook removed, repo deregistered", "Done.".green().bold());

    Ok(())
}

fn uninstall_hook() -> Result<()> {
    let git_dir = git::git_dir()?;
    let hook_path = git_dir.join("hooks").join("post-commit");

    if !hook_path.exists() {
        return Err(Error::HookNotInstalled("post-commit".to_string()));
    }

    let content = fs::read_to_string(&hook_path).map_err(|e| Error::ReadFile {
        path: hook_path.clone(),
        source: e,
    })?;

    if !content.contains(HOOK_MARKER_START) {
        return Err(Error::HookNotInstalled("post-commit".to_string()));
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
