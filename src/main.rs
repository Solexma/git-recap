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
    /// Manage git hooks
    Hook {
        #[command(subcommand)]
        action: HookAction,
    },

    /// Manual "I'm here" ping (lazy init, no commit needed)
    Touch,

    /// Show this repo's activity report
    Status,

    /// Show info about git-recap and current project
    Info,

    /// Compact summary of registered repos
    Digest {
        #[command(subcommand)]
        filter: DigestFilter,
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

#[derive(Subcommand)]
enum DigestFilter {
    /// Show all registered repos
    All {
        /// Output as JSON for scripts/Claude
        #[arg(long)]
        json: bool,
    },

    /// Filter repos with activity since a period
    Since {
        /// Period: yesterday, today, last-week, last-month, Nd (e.g. 7d), or YYYY-MM-DD
        value: String,

        /// Output as JSON for scripts/Claude
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum HookAction {
    /// Install git hook to run recap on commits
    Install {
        /// Hook to install (default: post-commit)
        #[arg(long, default_value = "post-commit")]
        on: String,
    },

    /// Remove git hook
    Uninstall {
        /// Hook to remove (default: post-commit)
        #[arg(long, default_value = "post-commit")]
        on: String,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Hook { action } => match action {
            HookAction::Install { on } => commands::hook::install(&on),
            HookAction::Uninstall { on } => commands::hook::uninstall(&on),
        },
        Commands::Touch => commands::touch::run(),
        Commands::Status => commands::status::run(),
        Commands::Info => commands::info::run(),
        Commands::Digest { filter } => match filter {
            DigestFilter::All { json } => commands::digest::run(None, json),
            DigestFilter::Since { value, json } => commands::digest::run(Some(&value), json),
        },
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
