use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

/// Get the data directory path (platform-specific).
#[must_use]
pub fn data_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("~/.local/share"))
        .join("git-recap")
}

/// Get the config directory path (platform-specific).
#[must_use]
pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("git-recap")
}

/// Get the registry file path.
#[must_use]
pub fn registry_file() -> PathBuf {
    config_dir().join("registry.toml")
}

/// Get the report file path for a given SHA.
#[must_use]
pub fn report_file(sha: &str) -> PathBuf {
    data_dir().join(format!("{sha}.toml"))
}

/// Ensure a directory exists.
///
/// # Errors
///
/// Returns an error if the directory cannot be created.
pub fn ensure_dir(dir: &Path) -> Result<()> {
    fs::create_dir_all(dir).map_err(|e| Error::CreateDir {
        path: dir.to_path_buf(),
        source: e,
    })
}
