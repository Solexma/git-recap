use std::process::ExitCode;

use clap::{Parser, Subcommand};
use colored::Colorize;

use git_recap::commands;

#[derive(Parser)]
#[command(
    name = "git-recap",
    about = "Hook-driven activity reporter for git repos",
    version,
    propagate_version = true
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Install post-commit hook and register repo
    Install,

    /// Remove hook and deregister repo
    Uninstall,

    /// Manual "I'm here" ping (lazy init, no commit needed)
    Touch,

    /// Show this repo's activity report
    Status,

    /// Show info about git-recap and current project
    Info,

    /// Update report (called by post-commit hook, not user-facing)
    #[command(hide = true)]
    Update,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Install => commands::install::run(),
        Commands::Uninstall => commands::uninstall::run(),
        Commands::Touch => commands::touch::run(),
        Commands::Status => commands::status::run(),
        Commands::Info => commands::info::run(),
        Commands::Update => commands::update::run(),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{} {e}", "error:".red().bold());
            ExitCode::FAILURE
        }
    }
}
