use colored::Colorize;

use crate::config;
use crate::error::Result;
use crate::git;
use crate::report::Report;

/// Display project and version information.
///
/// # Errors
///
/// Returns an error if git operations fail.
pub fn run() -> Result<()> {
    println!("{}", "git-recap".bold());
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
    println!();
    println!("Hook-driven activity reporter for git repos.");
    println!("Foundation data layer for gitnapped.");
    println!();
    println!("{}", "Author:".cyan());
    println!("  MiPnamic <mipnamic@mipnamic.net>");
    println!("  https://github.com/MiPnamic");
    println!();
    println!("{}", "Project:".cyan());
    println!("  https://github.com/Solexma/git-recap");
    println!("  MIT License - Solexma LLC");
    println!();

    if let Ok(sha) = git::initial_commit_sha() {
        println!("{}", "Current project:".cyan());
        println!("  Root SHA: {sha}");
        println!("  Report: {}", config::report_file(&sha).display());

        match Report::load(&sha) {
            Ok(_) => println!("  Tracked: {}", "yes".green()),
            Err(_) => println!("  Tracked: {}", "no".yellow()),
        }
    }

    Ok(())
}
