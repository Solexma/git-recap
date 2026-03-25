use colored::Colorize;

use crate::error::Result;
use crate::git;
use crate::registry::Registry;
use crate::report::Report;

/// Manual "I'm here" ping.
///
/// # Errors
///
/// Returns an error if git operations or file I/O fails.
pub fn run() -> Result<()> {
    let ctx = git::RepoContext::resolve()?;

    let mut registry = Registry::load()?;
    if !registry.is_registered(&ctx.sha) {
        registry.register(&ctx.sha, &ctx.root);
        registry.save()?;
    }

    let mut report = Report::load_or_init(ctx.root, ctx.name, ctx.sha)?;
    report.activity.last_touched = chrono::Local::now().fixed_offset();
    report.save()?;

    println!("{} touched", "Done.".green().bold());

    Ok(())
}
