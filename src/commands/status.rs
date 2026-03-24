use colored::Colorize;

use crate::error::Result;
use crate::git;
use crate::report::Report;

pub fn run() -> Result<()> {
    let sha = git::initial_commit_sha()?;
    let report = Report::load(&sha)?;

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
