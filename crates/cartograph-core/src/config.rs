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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GitDiffMode {
    StagedOnly,
    BranchRange(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexConfig {
    #[serde(default = "default_exclude_patterns")]
    pub exclude_patterns: Vec<String>,

    #[serde(default = "default_max_file_size_bytes")]
    pub max_file_size_bytes: usize,

    #[serde(default = "default_mmap_threshold_bytes")]
    pub mmap_threshold_bytes: usize,

    #[serde(default)]
    pub follow_symlinks: bool,

    #[serde(default)]
    pub num_threads: Option<usize>,
}

fn default_exclude_patterns() -> Vec<String> {
    vec![
        ".git".to_string(),
        ".cartograph".to_string(),
        "node_modules".to_string(),
        "vendor".to_string(),
        "target".to_string(),
        "dist".to_string(),
        "build".to_string(),
        ".venv".to_string(),
        "venv".to_string(),
        "__pycache__".to_string(),
        ".idea".to_string(),
        ".vscode".to_string(),
    ]
}

fn default_max_file_size_bytes() -> usize {
    2 * 1024 * 1024 // 2MB
}

fn default_mmap_threshold_bytes() -> usize {
    64 * 1024 // 64KB
}

impl Default for IndexConfig {
    fn default() -> Self {
        Self {
            exclude_patterns: default_exclude_patterns(),
            max_file_size_bytes: default_max_file_size_bytes(),
            mmap_threshold_bytes: default_mmap_threshold_bytes(),
            follow_symlinks: false,
            num_threads: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WatchConfig {
    #[serde(default = "default_debounce_ms")]
    pub debounce_ms: u64,

    #[serde(default = "default_cst_cache_capacity")]
    pub cst_cache_capacity: usize,

    #[serde(default = "default_track_blast_radius")]
    pub track_blast_radius: bool,
}

fn default_debounce_ms() -> u64 {
    25
}

fn default_cst_cache_capacity() -> usize {
    500
}

fn default_track_blast_radius() -> bool {
    true
}

impl Default for WatchConfig {
    fn default() -> Self {
        Self {
            debounce_ms: default_debounce_ms(),
            cst_cache_capacity: default_cst_cache_capacity(),
            track_blast_radius: default_track_blast_radius(),
        }
    }
}
