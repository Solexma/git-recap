use colored::Colorize;

use crate::error::Result;
use crate::git;
use crate::report::Report;

/// Show the current status of this repository's activity report.
///
/// # Errors
///
/// Returns an error if not in a git repo or no report exists.
pub fn run() -> Result<()> {
    let ctx = git::RepoContext::resolve()?;
    let report = Report::load(&ctx.sha)?;

    let branch = report
        .activity
        .last_commit
        .as_ref()
        .map_or("unknown", |c| c.branch.as_str());

    println!("{} ({})", report.repo.name.bold(), branch.cyan());

    if let Some(ref commit) = report.activity.last_commit {
        println!(
            "  last commit: {} \u{2014} {}",
            commit.date.format("%Y-%m-%d %H:%M"),
            commit.message
        );
    }

    println!(
        "  last touched: {}",
        report.activity.last_touched.format("%Y-%m-%d %H:%M")
    );

    if let Some(ref today) = report.activity.today {
        println!(
            "  today: {} commits, {} files, {} {}",
            today.commits,
            today.files_changed,
            format!("+{}", today.insertions).green(),
            format!("-{}", today.deletions).red()
        );
    }

    Ok(())
}
