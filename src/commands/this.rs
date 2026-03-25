use chrono::{DateTime, FixedOffset, Local};
use colored::Colorize;

use crate::error::Result;
use crate::git;
use crate::registry::Registry;
use crate::report::{Commit, Report};

/// Recap the latest commits into the activity report.
///
/// # Errors
///
/// Returns an error if git operations or file I/O fails.
pub fn run(count: Option<u32>, default: bool) -> Result<()> {
    let ctx = git::RepoContext::resolve()?;
    let mut registry = Registry::load()?;

    // Ensure registered
    if !registry.is_registered(&ctx.sha) {
        registry.register(&ctx.sha, &ctx.root);
    }

    // Handle --default flag
    if default {
        registry.clear_count(&ctx.sha);
        registry.save()?;
    }

    // Handle --count flag (persist for future use)
    if let Some(n) = count {
        registry.set_count(&ctx.sha, n);
        registry.save()?;
    }

    let effective_count = count.unwrap_or_else(|| registry.count_for(&ctx.sha));

    // Save registry if we just registered
    if !default && count.is_none() {
        // Only save if we modified it (registration)
        registry.save()?;
    }

    let mut report = Report::load_or_init(ctx.root, ctx.name, ctx.sha)?;
    report.activity.last_touched = Local::now().fixed_offset();

    // Fetch last N commits
    let commits = fetch_commits(effective_count)?;
    report.activity.commits = commits;

    report.save()?;

    let n = report.activity.commits.len();
    if n == 0 {
        println!("{} no commits to recap", "Done.".green().bold());
    } else {
        let first = &report.activity.commits[0];
        println!(
            "{} {n} commits recapped — latest: {}",
            "Done.".green().bold(),
            first.message
        );
    }

    Ok(())
}

fn fetch_commits(count: u32) -> Result<Vec<Commit>> {
    // Format: sha<SEP>date<SEP>subject<SEP>author
    let separator = "\x1f"; // ASCII unit separator
    let format = format!("%H{separator}%aI{separator}%s{separator}%aN");
    let output = git::run(&["log", &format!("-{count}"), &format!("--format={format}")])?;

    let branch = git::run(&["rev-parse", "--abbrev-ref", "HEAD"])?;
    let now: DateTime<FixedOffset> = Local::now().fixed_offset();

    let mut commits = Vec::new();
    for line in output.lines() {
        let parts: Vec<&str> = line.splitn(4, '\x1f').collect();
        if parts.len() == 4 {
            let date: DateTime<FixedOffset> = parts[1].parse().unwrap_or(now);
            commits.push(Commit {
                sha: parts[0].to_string(),
                date,
                message: parts[2].to_string(),
                branch: branch.clone(),
                author: parts[3].to_string(),
            });
        }
    }

    Ok(commits)
}
