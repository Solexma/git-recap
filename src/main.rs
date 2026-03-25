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

    /// Compact summary of all registered repos
    Digest {
        /// Filter repos with activity since date (yesterday, today, or YYYY-MM-DD)
        #[arg(long)]
        since: Option<String>,

        /// Output as JSON for scripts/Claude
        #[arg(long)]
        json: bool,
    },

    /// Recap the latest commits into the activity report
    This {
        /// Number of commits to recap (saves as default for this repo)
        #[arg(long)]
        count: Option<u32>,

        /// Clear per-repo count, fall back to global default
        #[arg(long)]
        default: bool,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Install => commands::install::run(),
        Commands::Uninstall => commands::uninstall::run(),
        Commands::Touch => commands::touch::run(),
        Commands::Status => commands::status::run(),
        Commands::Info => commands::info::run(),
        Commands::Digest { ref since, json } => commands::digest::run(since.as_ref(), json),
        Commands::This { count, default } => commands::this::run(count, default),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{} {e}", "error:".red().bold());
            ExitCode::FAILURE
        }
    }
}
