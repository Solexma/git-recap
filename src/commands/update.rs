use chrono::{DateTime, FixedOffset, Local, NaiveDate};

use crate::error::Result;
use crate::git;
use crate::registry;
use crate::report::{DayStats, LastCommit, Report};

/// Record the latest commit's activity into the report (post-commit hook entry point).
///
/// # Errors
///
/// Returns an error if git operations or file I/O fails.
pub fn run() -> Result<()> {
    let sha = git::initial_commit_sha()?;
    let root = git::repo_root()?;
    let name = git::repo_name()?;

    if !registry::is_registered(&sha)? {
        registry::register(&sha, &root)?;
    }

    let mut report = Report::load_or_init(root, name, sha)?;

    let now: DateTime<FixedOffset> = Local::now().fixed_offset();
    report.activity.last_touched = now;

    // Get commit info
    let date_str = git::run(&["log", "-1", "--format=%aI"])?;
    let message = git::run(&["log", "-1", "--format=%s"])?;
    let branch = git::run(&["rev-parse", "--abbrev-ref", "HEAD"])?;
    let author = git::run(&["log", "-1", "--format=%aN"])?;

    let commit_date: DateTime<FixedOffset> = date_str.parse().unwrap_or(now);

    report.activity.last_commit = Some(LastCommit {
        date: commit_date,
        message,
        branch,
        author,
    });

    // Get diff stats
    let (files_changed, insertions, deletions) = parse_diff_stats();

    // Update today stats
    let today = now.date_naive();
    report.activity.today = Some(accumulate_today(
        report.activity.today,
        today,
        commit_date,
        files_changed,
        insertions,
        deletions,
    ));

    report.save()?;

    Ok(())
}

fn parse_diff_stats() -> (u32, u32, u32) {
    let output = git::run(&["show", "--stat", "--format=", "HEAD"]).unwrap_or_default();

    let Some(summary_line) = output.lines().last() else {
        return (0, 0, 0);
    };

    let mut files = 0u32;
    let mut ins = 0u32;
    let mut del = 0u32;

    for part in summary_line.split(',') {
        let part = part.trim();
        if let Some(n) = part
            .split_whitespace()
            .next()
            .and_then(|s| s.parse().ok())
        {
            if part.contains("file") {
                files = n;
            } else if part.contains("insertion") {
                ins = n;
            } else if part.contains("deletion") {
                del = n;
            }
        }
    }

    (files, ins, del)
}

fn accumulate_today(
    existing: Option<DayStats>,
    today: NaiveDate,
    commit_date: DateTime<FixedOffset>,
    files_changed: u32,
    insertions: u32,
    deletions: u32,
) -> DayStats {
    match existing {
        Some(mut stats) if stats.date == today => {
            stats.commits += 1;
            stats.last = commit_date;
            stats.files_changed += files_changed;
            stats.insertions += insertions;
            stats.deletions += deletions;
            stats
        }
        _ => DayStats {
            date: today,
            commits: 1,
            first: commit_date,
            last: commit_date,
            files_changed,
            insertions,
            deletions,
        },
    }
}
