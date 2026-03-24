use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("not in a git repository")]
    NotInGitRepo,

    #[error("no commits in repository (cannot determine project identity)")]
    NoCommits,

    #[error("git command failed: {0}")]
    GitCommandFailed(String),

    #[error("hook already installed: {0}")]
    HookAlreadyInstalled(String),

    #[error("hook not installed: {0}")]
    HookNotInstalled(String),

    #[error("no report found for this repository")]
    NoReport,

    #[error("failed to read {}: {source}", path.display())]
    ReadFile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to write {}: {source}", path.display())]
    WriteFile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to create directory {}: {source}", path.display())]
    CreateDir {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to parse report {}: {source}", path.display())]
    ParseReport {
        path: PathBuf,
        source: toml::de::Error,
    },

    #[error("failed to serialize report: {0}")]
    SerializeReport(#[from] toml::ser::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
