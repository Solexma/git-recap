use std::collections::BTreeMap;
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
    config_dir().join("registry")
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

/// Read a key=value file into a `BTreeMap`.
///
/// # Errors
///
/// Returns an error if the file cannot be read.
pub fn read_kv_file(path: &Path) -> Result<BTreeMap<String, String>> {
    if !path.exists() {
        return Ok(BTreeMap::new());
    }

    let content = fs::read_to_string(path).map_err(|e| Error::ReadFile {
        path: path.to_path_buf(),
        source: e,
    })?;

    let mut map = BTreeMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            map.insert(key.to_string(), value.to_string());
        }
    }
    Ok(map)
}

/// Write a `BTreeMap` to a key=value file.
///
/// # Errors
///
/// Returns an error if the file cannot be written.
pub fn write_kv_file(path: &Path, map: &BTreeMap<String, String>) -> Result<()> {
    if let Some(parent) = path.parent() {
        ensure_dir(parent)?;
    }

    let mut content: String = map
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("\n");
    if !content.is_empty() {
        content.push('\n');
    }

    fs::write(path, content).map_err(|e| Error::WriteFile {
        path: path.to_path_buf(),
        source: e,
    })
}
