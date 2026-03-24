use std::fs;
use std::path::PathBuf;

use chrono::{DateTime, FixedOffset, Local, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::config;
use crate::error::{Error, Result};

#[derive(Debug, Serialize, Deserialize)]
pub struct Report {
    pub repo: Repo,
    pub activity: Activity,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Repo {
    pub path: PathBuf,
    pub name: String,
    pub sha: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Activity {
    pub last_commit: Option<LastCommit>,
    pub last_touched: DateTime<FixedOffset>,
    pub today: Option<DayStats>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LastCommit {
    pub date: DateTime<FixedOffset>,
    pub message: String,
    pub branch: String,
    pub author: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DayStats {
    pub date: NaiveDate,
    pub commits: u32,
    pub first: DateTime<FixedOffset>,
    pub last: DateTime<FixedOffset>,
    pub files_changed: u32,
    pub insertions: u32,
    pub deletions: u32,
}

impl Report {
    /// Create a new report with no commit data (for touch/lazy init).
    #[must_use]
    pub fn new_empty(path: PathBuf, name: String, sha: String) -> Self {
        let now: DateTime<FixedOffset> = Local::now().fixed_offset();
        Self {
            repo: Repo { path, name, sha },
            activity: Activity {
                last_commit: None,
                last_touched: now,
                today: None,
            },
        }
    }

    /// Load a report from disk by SHA.
    ///
    /// # Errors
    ///
    /// Returns an error if the report file cannot be read or parsed.
    pub fn load(sha: &str) -> Result<Self> {
        let path = config::report_file(sha);
        if !path.exists() {
            return Err(Error::NoReport);
        }
        let content = fs::read_to_string(&path).map_err(|e| Error::ReadFile {
            path: path.clone(),
            source: e,
        })?;
        toml::from_str(&content).map_err(|e| Error::ParseReport { path, source: e })
    }

    /// Save the report to disk.
    ///
    /// # Errors
    ///
    /// Returns an error if the report file cannot be written.
    pub fn save(&self) -> Result<()> {
        let path = config::report_file(&self.repo.sha);
        if let Some(parent) = path.parent() {
            config::ensure_dir(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        fs::write(&path, content).map_err(|e| Error::WriteFile {
            path: path.clone(),
            source: e,
        })
    }

    /// Load existing report or create a new empty one (lazy init).
    ///
    /// # Errors
    ///
    /// Returns an error if the report file exists but cannot be read or parsed.
    pub fn load_or_init(path: PathBuf, name: String, sha: String) -> Result<Self> {
        match Self::load(&sha) {
            Ok(report) => Ok(report),
            Err(Error::NoReport) => Ok(Self::new_empty(path, name, sha)),
            Err(e) => Err(e),
        }
    }
}
