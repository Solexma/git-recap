use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

/// Get the data directory path (platform-specific).
pub fn data_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("~/.local/share"))
        .join("git-recap")
}

/// Get the config directory path (platform-specific).
pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("git-recap")
}

/// Get the registry file path.
pub fn registry_file() -> PathBuf {
    config_dir().join("registry")
}

/// Get the report file path for a given SHA.
pub fn report_file(sha: &str) -> PathBuf {
    data_dir().join(format!("{sha}.toml"))
}

/// Ensure a directory exists.
pub fn ensure_dir(dir: &Path) -> Result<()> {
    if !dir.exists() {
        fs::create_dir_all(dir).map_err(|e| Error::CreateDir {
            path: dir.to_path_buf(),
            source: e,
        })?;
    }
    Ok(())
}

/// Read a key=value file into a HashMap.
pub fn read_kv_file(path: &Path) -> Result<HashMap<String, String>> {
    if !path.exists() {
        return Ok(HashMap::new());
    }

    let content = fs::read_to_string(path).map_err(|e| Error::ReadFile {
        path: path.to_path_buf(),
        source: e,
    })?;

    let mut map = HashMap::new();
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

/// Write a HashMap to a key=value file.
pub fn write_kv_file(path: &Path, map: &HashMap<String, String>) -> Result<()> {
    if let Some(parent) = path.parent() {
        ensure_dir(parent)?;
    }

    let content: String = map
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("\n");

    fs::write(path, content).map_err(|e| Error::WriteFile {
        path: path.to_path_buf(),
        source: e,
    })
}
