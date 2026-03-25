use chrono::{DateTime, FixedOffset, Local, NaiveDate};
use colored::Colorize;
use serde::Serialize;

use crate::error::{Error, Result};
use crate::registry::Registry;
use crate::report::Report;

#[derive(Serialize)]
struct DigestOutput {
    date: NaiveDate,
    since: Option<NaiveDate>,
    total_repos: usize,
    active_repos: usize,
    repos: Vec<DigestRepo>,
}

#[derive(Serialize)]
struct DigestRepo {
    name: String,
    path: String,
    branch: String,
    commits: usize,
    last_message: String,
    last_date: DateTime<FixedOffset>,
}

/// Show a compact summary of all registered repos.
///
/// # Errors
///
/// Returns an error if the registry or report files cannot be read.
pub fn run(since: Option<&String>, json: bool) -> Result<()> {
    let registry = Registry::load()?;

    let since_date = since.map(String::as_str).and_then(parse_since);
    let now = Local::now().fixed_offset();

    let mut active_repos = Vec::new();

    for entry in &registry.repos {
        let Ok(report) = Report::load(&entry.sha) else {
            continue;
        };

        // Determine latest activity date
        let latest = report
            .activity
            .commits
            .first()
            .map_or(report.activity.last_touched, |c| c.date);

        // Filter by --since
        if let Some(since) = since_date {
            if latest.date_naive() < since {
                continue;
            }
        }

        active_repos.push((report, latest));
    }

    // Sort by latest activity descending
    active_repos.sort_by(|a, b| b.1.cmp(&a.1));

    let total_repos = registry.repos.len();

    if json {
        output_json(&active_repos, total_repos, now.date_naive(), since_date)?;
    } else {
        output_text(&active_repos, since_date);
    }

    Ok(())
}

fn output_json(
    active_repos: &[(Report, DateTime<FixedOffset>)],
    total_repos: usize,
    date: NaiveDate,
    since: Option<NaiveDate>,
) -> Result<()> {
    let output = DigestOutput {
        date,
        since,
        total_repos,
        active_repos: active_repos.len(),
        repos: active_repos
            .iter()
            .map(|(report, _)| {
                let first = report.activity.commits.first();

                DigestRepo {
                    name: report.repo.name.clone(),
                    path: report.repo.path.to_string_lossy().to_string(),
                    branch: first.map_or_else(String::new, |c| c.branch.clone()),
                    commits: report.activity.commits.len(),
                    last_message: first
                        .map_or_else(|| "touched".to_string(), |c| c.message.clone()),
                    last_date: first.map_or(report.activity.last_touched, |c| c.date),
                }
            })
            .collect(),
    };

    let json = serde_json::to_string_pretty(&output)
        .map_err(|e| Error::GitCommandFailed(e.to_string()))?;
    println!("{json}");

    Ok(())
}

fn output_text(active_repos: &[(Report, DateTime<FixedOffset>)], since_date: Option<NaiveDate>) {
    if active_repos.is_empty() {
        println!("No active repos.");
        return;
    }

    let label = since_date.map_or_else(
        || format!("{} repos registered:", active_repos.len()),
        |since| format!("{} repos active since {since}:", active_repos.len()),
    );
    println!("{label}");
    println!();

    for (report, _) in active_repos {
        let branch = report
            .activity
            .commits
            .first()
            .map_or("", |c| c.branch.as_str());

        if report.activity.commits.is_empty() {
            println!("{}", report.repo.name.bold());
            println!(
                "  touched — {}",
                report.activity.last_touched.format("%Y-%m-%d %H:%M")
            );
        } else {
            let first = &report.activity.commits[0];
            if branch.is_empty() {
                println!("{}", report.repo.name.bold());
            } else {
                println!("{} ({})", report.repo.name.bold(), branch.cyan());
            }
            println!(
                "  {} commits, last: {} — {}",
                report.activity.commits.len(),
                first.message,
                first.date.format("%Y-%m-%d %H:%M")
            );
        }

        println!();
    }
}

fn parse_since(s: &str) -> Option<NaiveDate> {
    let today = Local::now().date_naive();
    match s {
        "yesterday" => Some(today - chrono::Duration::days(1)),
        "today" => Some(today),
        "last-week" => Some(today - chrono::Duration::days(7)),
        "last-month" => Some(today - chrono::Duration::days(30)),
        _ => {
            // Try Nd format (e.g. "7d", "28d")
            if let Some(days_str) = s.strip_suffix('d') {
                if let Ok(days) = days_str.parse::<i64>() {
                    return Some(today - chrono::Duration::days(days));
                }
            }
            // Try YYYYMMDD format
            NaiveDate::parse_from_str(s, "%Y%m%d")
                .or_else(|_| NaiveDate::parse_from_str(s, "%Y-%m-%d"))
                .ok()
        }
    }
}
