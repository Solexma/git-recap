use std::path::Path;

use crate::config;
use crate::error::Result;

/// Register a repo in the registry (SHA -> path).
pub fn register(sha: &str, path: &Path) -> Result<()> {
    let mut map = config::read_kv_file(&config::registry_file())?;
    map.insert(sha.to_string(), path.to_string_lossy().to_string());
    config::write_kv_file(&config::registry_file(), &map)
}

/// Deregister a repo from the registry.
pub fn deregister(sha: &str) -> Result<()> {
    let mut map = config::read_kv_file(&config::registry_file())?;
    map.remove(sha);
    config::write_kv_file(&config::registry_file(), &map)
}

/// Check if a repo is registered.
pub fn is_registered(sha: &str) -> Result<bool> {
    let map = config::read_kv_file(&config::registry_file())?;
    Ok(map.contains_key(sha))
}
