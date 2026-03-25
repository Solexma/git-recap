use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::config;
use crate::error::{Error, Result};

const DEFAULT_COUNT: u32 = 10;

#[derive(Debug, Serialize, Deserialize)]
pub struct Registry {
    #[serde(default = "default_count")]
    pub default_count: u32,
    #[serde(default)]
    pub repos: Vec<RepoEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RepoEntry {
    pub sha: String,
    pub path: PathBuf,
    pub count: Option<u32>,
}

const fn default_count() -> u32 {
    DEFAULT_COUNT
}

impl Registry {
    /// Load the registry from disk, or return a default empty one.
    ///
    /// # Errors
    ///
    /// Returns an error if the registry file cannot be read or parsed.
    pub fn load() -> Result<Self> {
        let path = config::registry_file();
        if !path.exists() {
            return Ok(Self {
                default_count: DEFAULT_COUNT,
                repos: Vec::new(),
            });
        }
        let content = fs::read_to_string(&path).map_err(|e| Error::ReadFile {
            path: path.clone(),
            source: e,
        })?;
        toml::from_str(&content).map_err(|e| Error::ParseReport { path, source: e })
    }

    /// Save the registry to disk.
    ///
    /// # Errors
    ///
    /// Returns an error if the registry file cannot be written.
    pub fn save(&self) -> Result<()> {
        let path = config::registry_file();
        if let Some(parent) = path.parent() {
            config::ensure_dir(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        fs::write(&path, content).map_err(|e| Error::WriteFile {
            path: path.clone(),
            source: e,
        })
    }

    /// Find a repo entry by SHA.
    #[must_use]
    pub fn find(&self, sha: &str) -> Option<&RepoEntry> {
        self.repos.iter().find(|r| r.sha == sha)
    }

    /// Find a mutable repo entry by SHA.
    pub fn find_mut(&mut self, sha: &str) -> Option<&mut RepoEntry> {
        self.repos.iter_mut().find(|r| r.sha == sha)
    }

    /// Register a repo. If already registered, update the path.
    pub fn register(&mut self, sha: &str, path: &Path) {
        if let Some(entry) = self.find_mut(sha) {
            entry.path = path.to_path_buf();
        } else {
            self.repos.push(RepoEntry {
                sha: sha.to_string(),
                path: path.to_path_buf(),
                count: None,
            });
        }
    }

    /// Deregister a repo by SHA.
    pub fn deregister(&mut self, sha: &str) {
        self.repos.retain(|r| r.sha != sha);
    }

    /// Check if a repo is registered.
    #[must_use]
    pub fn is_registered(&self, sha: &str) -> bool {
        self.find(sha).is_some()
    }

    /// Get the effective count for a repo (per-repo override or default).
    #[must_use]
    pub fn count_for(&self, sha: &str) -> u32 {
        self.find(sha)
            .and_then(|r| r.count)
            .unwrap_or(self.default_count)
    }

    /// Set per-repo count override.
    pub fn set_count(&mut self, sha: &str, count: u32) {
        if let Some(entry) = self.find_mut(sha) {
            entry.count = Some(count);
        }
    }

    /// Clear per-repo count override (fall back to default).
    pub fn clear_count(&mut self, sha: &str) {
        if let Some(entry) = self.find_mut(sha) {
            entry.count = None;
        }
    }
}
