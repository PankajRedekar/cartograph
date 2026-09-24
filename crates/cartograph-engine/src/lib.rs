pub mod complexity;
pub mod deadcode;
pub mod discovery;
pub mod error;
pub mod extract_logic;
pub mod hasher;
pub mod impact;
pub mod linker;
pub mod lint_arch;
pub mod locate;
pub mod pipeline;
pub mod routes;
pub mod stats;
pub mod symbol_table;
pub mod test_selector;
pub mod watcher;

// Re-export core abstractions
pub use complexity::{ComplexityAnalysisOptions, ComplexityEngine};
pub use deadcode::{
    BaselineManager, DeadCodeEngine, DeadCodeRunOptions, PruneAdvisor, export_ai_ignore_mask,
};
pub use discovery::{
    DiscoveredGitRepo, DiscoveryConfig, FileWalker, GitWorkspaceResolver, ZeroCopyBuffer,
};
pub use error::{EngineError, Result};
pub use extract_logic::{
    ConceptTaxonomyMatcher, InterProceduralTracer, LogicExtractionEngine, LogicOutputFormat,
    RuleDiffEngine, RuleVerbalizer,
};
pub use hasher::{DiscoveredFile, FileStatus, IncrementalChangeSet, IncrementalDetector};
pub use impact::{ImpactAnalyzer, ImpactOutputFormat, ImpactQuery, SemVerGuard};
pub use linker::ReferenceResolver;
pub use lint_arch::{
    ArchLinter, baseline::ArchBaselineManager, diff::GitDiffScope, formatters::ArchOutputFormat,
};
pub use locate::{
    LocateCliOptions, LocateQuery, LocateResult, OutputFormat, QueryParser, SubgraphLocator,
};
pub use pipeline::IndexPipeline;
pub use routes::RouteEngine;
pub use stats::IndexStats;
pub use symbol_table::{NodeSummary, SymbolTable};
pub use test_selector::{
    ExecutionSummary, TestExecutor, TestSelector, TestSelectorOptions, TestSelectorOutputFormat,
};
pub use watcher::{FileWatcher, WatchEvent, WatcherSettings};
