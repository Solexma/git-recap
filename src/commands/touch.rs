use colored::Colorize;

use crate::error::Result;
use crate::git;
use crate::registry;
use crate::report::Report;

pub fn run() -> Result<()> {
    let sha = git::initial_commit_sha()?;
    let root = git::repo_root()?;
    let name = git::repo_name()?;

    if !registry::is_registered(&sha)? {
        registry::register(&sha, &root)?;
    }

    let mut report = Report::load_or_init(root, name, sha)?;
    report.activity.last_touched = chrono::Local::now().fixed_offset();
    report.save()?;

    println!("{} touched", "Done.".green().bold());

    Ok(())
}
