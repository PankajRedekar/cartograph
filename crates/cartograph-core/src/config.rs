use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SqliteConfig {
    #[serde(default = "default_sqlite_path")]
    pub path: String,

    #[serde(default = "default_mmap_size")]
    pub mmap_size: usize,

    #[serde(default = "default_busy_timeout")]
    pub busy_timeout_ms: u64,

    #[serde(default = "default_journal_mode")]
    pub journal_mode: String,

    #[serde(default = "default_synchronous")]
    pub synchronous: String,
}

fn default_sqlite_path() -> String {
    ".cartograph/cartograph.db".to_string()
}

fn default_mmap_size() -> usize {
    268_435_456 // 256MB
}

fn default_busy_timeout() -> u64 {
    5000 // 5 seconds
}

fn default_journal_mode() -> String {
    "WAL".to_string()
}

fn default_synchronous() -> String {
    "NORMAL".to_string()
}

impl Default for SqliteConfig {
    fn default() -> Self {
        Self {
            path: default_sqlite_path(),
            mmap_size: default_mmap_size(),
            busy_timeout_ms: default_busy_timeout(),
            journal_mode: default_journal_mode(),
            synchronous: default_synchronous(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct MemoryConfig {
    #[serde(default)]
    pub initial_capacity: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageConfig {
    #[serde(default = "default_backend")]
    pub backend: String,

    #[serde(default)]
    pub sqlite: Option<SqliteConfig>,

    #[serde(default)]
    pub memory: Option<MemoryConfig>,
}

fn default_backend() -> String {
    "sqlite".to_string()
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            backend: default_backend(),
            sqlite: Some(SqliteConfig::default()),
            memory: None,
        }
    }
}

impl StorageConfig {
    pub fn sqlite(path: impl Into<String>) -> Self {
        Self {
            backend: "sqlite".to_string(),
            sqlite: Some(SqliteConfig {
                path: path.into(),
                ..Default::default()
            }),
            memory: None,
        }
    }

    pub fn memory() -> Self {
        Self {
            backend: "memory".to_string(),
            sqlite: None,
            memory: Some(MemoryConfig::default()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ProfileConfig {
    #[serde(default = "default_active_profile")]
    pub active_profile: String,

    #[serde(default)]
    pub profiles: HashMap<String, StorageConfig>,
}

fn default_active_profile() -> String {
    "default".to_string()
}

impl ProfileConfig {
    pub fn active_storage(&self) -> StorageConfig {
        self.profiles
            .get(&self.active_profile)
            .cloned()
            .unwrap_or_default()
    }
}
