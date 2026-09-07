pub mod discovery;
pub mod error;
pub mod hasher;
pub mod linker;
pub mod locate;
pub mod pipeline;
pub mod stats;
pub mod symbol_table;
pub mod watcher;

// Re-export core abstractions
pub use discovery::{DiscoveryConfig, FileWalker, ZeroCopyBuffer};
pub use error::{EngineError, Result};
pub use hasher::{DiscoveredFile, FileStatus, IncrementalChangeSet, IncrementalDetector};
pub use linker::ReferenceResolver;
pub use locate::{
    LocateCliOptions, LocateQuery, LocateResult, OutputFormat, QueryParser, SubgraphLocator,
};
pub use pipeline::IndexPipeline;
pub use stats::IndexStats;
pub use symbol_table::{NodeSummary, SymbolTable};
pub use watcher::{FileWatcher, WatchEvent, WatcherSettings};
