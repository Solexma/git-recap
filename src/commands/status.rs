use colored::Colorize;

use crate::error::Result;
use crate::git;
use crate::report::Report;

/// Show this repo's activity report.
///
/// # Errors
///
/// Returns an error if the report cannot be loaded.
pub fn run() -> Result<()> {
    let ctx = git::RepoContext::resolve()?;
    let report = Report::load(&ctx.sha)?;

    let branch = report
        .activity
        .commits
        .first()
        .map_or("unknown", |c| c.branch.as_str());

    println!("{} ({})", report.repo.name.bold(), branch.cyan());
    println!(
        "  last touched: {}",
        report.activity.last_touched.format("%Y-%m-%d %H:%M")
    );

    if report.activity.commits.is_empty() {
        println!("  no commits recapped");
    } else {
        println!("  {} commits recapped:", report.activity.commits.len());
        for commit in &report.activity.commits {
            println!(
                "    {} {} — {}",
                commit.sha[..7].dimmed(),
                commit.date.format("%Y-%m-%d %H:%M"),
                commit.message
            );
        }
    }

    Ok(())
}
