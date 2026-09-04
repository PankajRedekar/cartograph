use std::sync::Arc;

use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::config::StorageConfig;
use cartograph_core::error::{CartographError, Result};

use crate::memory::MemoryBackend;
use crate::sqlite::SqliteBackend;

pub fn create_backend(config: &StorageConfig) -> Result<Arc<dyn GraphStoreBackend>> {
    match config.backend.to_lowercase().as_str() {
        "sqlite" => {
            let sqlite_cfg = config.sqlite.clone().unwrap_or_default();
            let backend = SqliteBackend::open(&sqlite_cfg)?;
            Ok(Arc::new(backend))
        }
        "memory" => {
            let memory_cfg = config.memory.clone().unwrap_or_default();
            let backend = MemoryBackend::with_config(memory_cfg);
            Ok(Arc::new(backend))
        }
        other => Err(CartographError::Configuration(format!(
            "Unsupported storage backend '{other}'. Supported backends: 'sqlite', 'memory'"
        ))),
    }
}
