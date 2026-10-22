pub mod ai_review;
pub mod complexity;
pub mod deadcode;
pub mod diagnose;
pub mod diagram;
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
pub mod skills;
pub mod stats;
pub mod symbol_table;
pub mod test_selector;
pub mod watcher;

// Re-export core abstractions
pub use ai_review::{
    AiReviewEngine, ChangelogGenerator, ComplexityGate, ContextCompiler, ContractAnalyzer,
    GitNotesPublisher, JsonPublisher, LlmClient, MarkdownPublisher, PatchApplier, PatchPublisher,
    ReviewMemoryManager, ReviewPublisher, RiskEvaluator, SarifPublisher, SideEffectAnalyzer,
    StructuralGates, TerminalPublisher, WebhookPublisher,
};
pub use complexity::{ComplexityAnalysisOptions, ComplexityEngine};
pub use deadcode::{
    BaselineManager, DeadCodeEngine, DeadCodeRunOptions, PruneAdvisor, export_ai_ignore_mask,
};
pub use diagnose::{
    CallPathReconstructor, DiagnoseEngine, FaultClassifier, FrameAnchor, GitCorrelator,
    InvariantAuditor, PatchSynthesizer, PathNormalizer, ProvenanceSlicer, RegressionTestScaffolder,
    TopologicalFaultScorer, TracebackParser,
};
pub use diagram::{DiagramEngine, SugiyamaLayoutEngine, render_diagram};
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
pub use skills::{
    CheckStatus, DoctorReport, InstallOptions, InstallReport, RoiStatsReport, SkillDefinition,
    SkillDoctor, SkillInstaller, SkillPreset, SkillRegistry, SkillUpdateReport, SkillUpdater,
    TargetIde, TelemetryEvent, TelemetryRecorder, VerificationCheck, VerificationGateEngine,
    VerificationGateReport,
};
pub use stats::IndexStats;
pub use symbol_table::{NodeSummary, SymbolTable};
pub use test_selector::{
    ExecutionSummary, TestExecutor, TestSelector, TestSelectorOptions, TestSelectorOutputFormat,
};
pub use watcher::{FileWatcher, WatchEvent, WatcherSettings};
