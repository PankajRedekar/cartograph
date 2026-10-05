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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestConfig {
    #[serde(default = "default_test_granularity")]
    pub granularity: String,

    #[serde(default)]
    pub min_coverage: f64,

    #[serde(default)]
    pub fail_on_untested: bool,

    #[serde(default = "default_max_tests")]
    pub max_tests: usize,

    #[serde(default = "default_min_confidence")]
    pub min_confidence: f64,

    #[serde(default = "default_max_test_depth")]
    pub max_depth: usize,

    #[serde(default = "default_true")]
    pub ignore_docstrings: bool,

    #[serde(default)]
    pub history: TestHistoryConfig,

    #[serde(default)]
    pub runners: HashMap<String, TestRunnerConfig>,
}

fn default_test_granularity() -> String {
    "auto".to_string()
}

fn default_max_tests() -> usize {
    500
}

fn default_min_confidence() -> f64 {
    0.7
}

fn default_max_test_depth() -> usize {
    5
}

fn default_true() -> bool {
    true
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            granularity: default_test_granularity(),
            min_coverage: 0.0,
            fail_on_untested: false,
            max_tests: default_max_tests(),
            min_confidence: default_min_confidence(),
            max_depth: default_max_test_depth(),
            ignore_docstrings: true,
            history: TestHistoryConfig::default(),
            runners: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestHistoryConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,

    #[serde(default = "default_history_path")]
    pub path: String,

    #[serde(default = "default_failure_weight")]
    pub failure_weight: u32,
}

fn default_history_path() -> String {
    ".cartograph/test_history.json".to_string()
}

fn default_failure_weight() -> u32 {
    25
}

impl Default for TestHistoryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            path: default_history_path(),
            failure_weight: default_failure_weight(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestRunnerConfig {
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
}

use crate::model::EdgeKind;

/// Granularity level for circular dependency detection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CycleGranularity {
    /// File-level dependencies (e.g. src/auth/login.rs -> src/user/profile.rs)
    #[default]
    File,
    /// Module/directory-level dependencies (e.g. src/auth -> src/billing)
    Module,
    /// Package/crate-level dependencies (e.g. crates/core -> crates/engine)
    Package,
    /// Symbol-level dependencies (e.g. fn authenticate -> fn fetch_user)
    Symbol,
}

impl CycleGranularity {
    pub fn as_str(&self) -> &'static str {
        match self {
            CycleGranularity::File => "file",
            CycleGranularity::Module => "module",
            CycleGranularity::Package => "package",
            CycleGranularity::Symbol => "symbol",
        }
    }
}

impl std::fmt::Display for CycleGranularity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for CycleGranularity {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "file" => Ok(CycleGranularity::File),
            "module" => Ok(CycleGranularity::Module),
            "package" | "crate" => Ok(CycleGranularity::Package),
            "symbol" | "fn" => Ok(CycleGranularity::Symbol),
            other => Err(format!(
                "Unknown cycle granularity: {other}. Expected file, module, package, or symbol"
            )),
        }
    }
}

/// Filter for dependency coupling nature: runtime execution vs type-only
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CouplingFilter {
    /// Only evaluate runtime execution dependencies (Calls, value Imports, object instantiations).
    /// Benign type-only imports (e.g. `import type`, `if TYPE_CHECKING:`) are excluded.
    #[default]
    Runtime,
    /// Only evaluate type-only contract dependencies (interface implementations, type hints).
    TypeOnly,
    /// Evaluate all dependency edges without distinction.
    All,
}

impl CouplingFilter {
    pub fn as_str(&self) -> &'static str {
        match self {
            CouplingFilter::Runtime => "runtime",
            CouplingFilter::TypeOnly => "type_only",
            CouplingFilter::All => "all",
        }
    }
}

impl std::fmt::Display for CouplingFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for CouplingFilter {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "runtime" => Ok(CouplingFilter::Runtime),
            "type" | "type_only" | "types" => Ok(CouplingFilter::TypeOnly),
            "all" => Ok(CouplingFilter::All),
            other => Err(format!(
                "Unknown coupling filter: {other}. Expected runtime, type_only, or all"
            )),
        }
    }
}

/// Severity classification of an architectural violation
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
#[serde(rename_all = "snake_case")]
pub enum ArchSeverity {
    /// Informational advisory; does not fail CI
    Info,
    /// Warning; logged visibly, fails CI only under --strict or --fail-on warning
    Warning,
    /// Error; fails CI under default policies
    #[default]
    Error,
}

impl ArchSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            ArchSeverity::Info => "info",
            ArchSeverity::Warning => "warning",
            ArchSeverity::Error => "error",
        }
    }
}

impl std::fmt::Display for ArchSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for ArchSeverity {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "info" => Ok(ArchSeverity::Info),
            "warn" | "warning" => Ok(ArchSeverity::Warning),
            "error" | "err" => Ok(ArchSeverity::Error),
            other => Err(format!(
                "Unknown severity: {other}. Expected info, warning, or error"
            )),
        }
    }
}

/// Action to take when a rule matches an edge
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RuleAction {
    /// Deny this dependency (violation if matched)
    #[default]
    Deny,
    /// Allow this dependency (whitelisted)
    Allow,
}

/// Configuration for an architectural layer
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchLayerConfig {
    /// Unique identifier for this layer (e.g. "domain", "application", "infrastructure", "presentation")
    pub name: String,
    /// Human-readable explanation of this layer's responsibilities
    #[serde(default)]
    pub description: Option<String>,
    /// Filepath glob patterns identifying components belonging to this layer
    #[serde(default)]
    pub patterns: Vec<String>,
    /// Explicit list of layer names that this layer is forbidden from depending on
    #[serde(default)]
    pub forbidden: Vec<String>,
    /// Explicit list of layer names that this layer is exclusively permitted to depend on
    #[serde(default)]
    pub allowed: Option<Vec<String>>,
    /// Severity if a boundary violation occurs in this layer
    #[serde(default = "default_severity_error")]
    pub severity: ArchSeverity,
}

fn default_severity_error() -> ArchSeverity {
    ArchSeverity::Error
}

/// Monorepo package boundary and encapsulation configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageEncapsulationConfig {
    /// Whether package boundary encapsulation is enforced
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Glob patterns defining package root directories (e.g. ["crates/*", "packages/*", "apps/*"])
    #[serde(default = "default_package_roots")]
    pub package_roots: Vec<String>,
    /// Forbidden internal patterns for external consumers (e.g. ["**/internal/**", "**/private/**"])
    #[serde(default = "default_internal_patterns")]
    pub internal_patterns: Vec<String>,
    /// Permitted public entry point file names
    #[serde(default = "default_entrypoints")]
    pub entrypoints: Vec<String>,
    /// Severity when an external package deep-imports an internal module
    #[serde(default = "default_severity_error")]
    pub severity: ArchSeverity,
}

fn default_package_roots() -> Vec<String> {
    vec![
        "crates/*".to_string(),
        "packages/*".to_string(),
        "apps/*".to_string(),
    ]
}

fn default_internal_patterns() -> Vec<String> {
    vec!["**/internal/**".to_string(), "**/private/**".to_string()]
}

fn default_entrypoints() -> Vec<String> {
    vec![
        "index.ts".to_string(),
        "index.js".to_string(),
        "lib.rs".to_string(),
        "mod.rs".to_string(),
        "__init__.py".to_string(),
    ]
}

impl Default for PackageEncapsulationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            package_roots: default_package_roots(),
            internal_patterns: default_internal_patterns(),
            entrypoints: default_entrypoints(),
            severity: ArchSeverity::Error,
        }
    }
}

/// Fine-grained architectural boundary rule (bounded contexts, package isolation)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchRuleConfig {
    /// Name or ID of the rule (e.g. "auth-isolation", "no-direct-db")
    pub name: String,
    /// Human-readable description
    #[serde(default)]
    pub description: Option<String>,
    /// Source glob pattern (originating module/file)
    pub source: String,
    /// Target glob pattern (imported or called module/file)
    pub target: String,
    /// Action: deny or allow
    #[serde(default = "default_rule_action_deny")]
    pub action: RuleAction,
    /// Specific edge kinds to match (e.g. ["imports", "calls"]). If None, matches all dependency edges.
    #[serde(default)]
    pub edge_kinds: Option<Vec<EdgeKind>>,
    /// Severity level
    #[serde(default = "default_severity_error")]
    pub severity: ArchSeverity,
    /// Custom error message to display when violated
    #[serde(default)]
    pub message: Option<String>,
    /// Concrete remediation hint for fixing the violation
    #[serde(default)]
    pub hint: Option<String>,
}

fn default_rule_action_deny() -> RuleAction {
    RuleAction::Deny
}

/// Architectural exemption rule (suppression / debt waiver)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchExemptionRule {
    /// Source filepath or glob pattern
    pub source: String,
    /// Target filepath or glob pattern
    pub target: String,
    /// Justification or tech-debt tracking ticket
    #[serde(default)]
    pub reason: Option<String>,
    /// Expiration date (YYYY-MM-DD); if current date > expires, exemption is revoked
    #[serde(default)]
    pub expires: Option<String>,
}

/// Exemptions container
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ArchExemptionsConfig {
    /// Glob patterns of paths completely excluded from all architectural linting (e.g. tests/**, fixtures/**)
    #[serde(default)]
    pub ignore_paths: Vec<String>,
    /// Explicit allowed cross-layer edge exemptions
    #[serde(default)]
    pub allow: Vec<ArchExemptionRule>,
}

/// Complete architectural linting configuration in cartograph.toml
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LintArchConfig {
    /// Whether architectural linting is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Preset architecture template (e.g. "clean-architecture", "hexagonal", "mvc", "monorepo")
    #[serde(default)]
    pub preset: Option<String>,
    /// Enable circular dependency detection
    #[serde(default = "default_true")]
    pub detect_cycles: bool,
    /// Granularity for cycle detection
    #[serde(default)]
    pub cycle_granularity: CycleGranularity,
    /// Coupling filter: runtime, type_only, or all
    #[serde(default)]
    pub coupling: CouplingFilter,
    /// Whether in-code AST @arch tags in docstrings/attributes are honored
    #[serde(default = "default_true")]
    pub honor_arch_tags: bool,
    /// Monorepo package boundary encapsulation guard
    #[serde(default)]
    pub package_encapsulation: PackageEncapsulationConfig,
    /// Maximum search depth / length of cycle paths to explore
    #[serde(default = "default_max_cycle_depth")]
    pub max_cycle_depth: usize,
    /// Maximum number of cycles to extract before truncation
    #[serde(default = "default_max_cycles")]
    pub max_cycles: usize,
    /// CI failure threshold: "error", "warning", or "never"
    #[serde(default = "default_fail_on")]
    pub fail_on: String,
    /// Strict mode: treat all warnings as errors
    #[serde(default)]
    pub strict: bool,
    /// Maximum permitted violations before failing CI
    #[serde(default)]
    pub max_violations: usize,
    /// Path to technical debt baseline file
    #[serde(default)]
    pub baseline: Option<String>,
    /// Automatically update baseline when violations are resolved
    #[serde(default)]
    pub ratchet: bool,
    /// Ordered list of architectural layers
    #[serde(default)]
    pub layers: Vec<ArchLayerConfig>,
    /// Fine-grained boundary rules
    #[serde(default)]
    pub rules: Vec<ArchRuleConfig>,
    /// Exemptions and waivers
    #[serde(default)]
    pub exemptions: ArchExemptionsConfig,
}

fn default_max_cycle_depth() -> usize {
    10
}

fn default_max_cycles() -> usize {
    50
}

fn default_fail_on() -> String {
    "error".to_string()
}

impl Default for LintArchConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            preset: None,
            detect_cycles: true,
            cycle_granularity: CycleGranularity::File,
            coupling: CouplingFilter::Runtime,
            honor_arch_tags: true,
            package_encapsulation: PackageEncapsulationConfig::default(),
            max_cycle_depth: default_max_cycle_depth(),
            max_cycles: default_max_cycles(),
            fail_on: default_fail_on(),
            strict: false,
            max_violations: 0,
            baseline: None,
            ratchet: false,
            layers: Vec::new(),
            rules: Vec::new(),
            exemptions: ArchExemptionsConfig::default(),
        }
    }
}

/// Project-level configuration for cartograph complexity in cartograph.toml
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComplexityConfig {
    /// Scoring weights for composite hotspot score (must sum to ~1.0)
    #[serde(default = "default_weight_cyclomatic")]
    pub weight_cyclomatic: f64,
    #[serde(default = "default_weight_cognitive")]
    pub weight_cognitive: f64,
    #[serde(default = "default_weight_loc")]
    pub weight_loc: f64,
    #[serde(default = "default_weight_coupling")]
    pub weight_coupling: f64,
    #[serde(default = "default_weight_churn")]
    pub weight_churn: f64,

    /// Thresholds for functions/methods
    #[serde(default = "default_max_function_cyclomatic")]
    pub max_function_cyclomatic: u32,
    #[serde(default = "default_max_function_cognitive")]
    pub max_function_cognitive: u32,
    #[serde(default = "default_max_function_loc")]
    pub max_function_loc: u32,
    #[serde(default = "default_max_nesting_depth")]
    pub max_nesting_depth: u32,
    #[serde(default = "default_max_parameters")]
    pub max_parameters: u32,

    /// Thresholds for classes/structs (God-Class detection)
    #[serde(default = "default_max_class_wmc")]
    pub max_class_wmc: u32,
    #[serde(default = "default_max_class_loc")]
    pub max_class_loc: u32,
    #[serde(default = "default_max_class_methods")]
    pub max_class_methods: usize,
    #[serde(default = "default_min_lcom4")]
    pub min_lcom4: usize,
    #[serde(default = "default_max_god_class_index")]
    pub max_god_class_index: f64,

    /// AI Context Reasonability Limits
    #[serde(default = "default_max_ai_tokens")]
    pub max_ai_tokens: usize,
    #[serde(default = "default_max_ai_cognitive")]
    pub max_ai_cognitive: u32,

    /// Git Churn Mining Settings
    #[serde(default = "default_churn_window_days")]
    pub churn_window_days: u32,
    #[serde(default = "default_churn_max_commits")]
    pub churn_max_commits: usize,

    /// Default top N items to display
    #[serde(default = "default_top_n")]
    pub default_top_n: usize,
}

fn default_weight_cyclomatic() -> f64 {
    0.35
}
fn default_weight_cognitive() -> f64 {
    0.25
}
fn default_weight_loc() -> f64 {
    0.15
}
fn default_weight_coupling() -> f64 {
    0.15
}
fn default_weight_churn() -> f64 {
    0.10
}

fn default_max_function_cyclomatic() -> u32 {
    15
}
fn default_max_function_cognitive() -> u32 {
    20
}
fn default_max_function_loc() -> u32 {
    100
}
fn default_max_nesting_depth() -> u32 {
    4
}
fn default_max_parameters() -> u32 {
    5
}

fn default_max_class_wmc() -> u32 {
    50
}
fn default_max_class_loc() -> u32 {
    500
}
fn default_max_class_methods() -> usize {
    20
}
fn default_min_lcom4() -> usize {
    2
}
fn default_max_god_class_index() -> f64 {
    75.0
}

fn default_max_ai_tokens() -> usize {
    2500
}
fn default_max_ai_cognitive() -> u32 {
    30
}

fn default_churn_window_days() -> u32 {
    90
}
fn default_churn_max_commits() -> usize {
    100
}
fn default_top_n() -> usize {
    15
}

impl Default for ComplexityConfig {
    fn default() -> Self {
        Self {
            weight_cyclomatic: default_weight_cyclomatic(),
            weight_cognitive: default_weight_cognitive(),
            weight_loc: default_weight_loc(),
            weight_coupling: default_weight_coupling(),
            weight_churn: default_weight_churn(),
            max_function_cyclomatic: default_max_function_cyclomatic(),
            max_function_cognitive: default_max_function_cognitive(),
            max_function_loc: default_max_function_loc(),
            max_nesting_depth: default_max_nesting_depth(),
            max_parameters: default_max_parameters(),
            max_class_wmc: default_max_class_wmc(),
            max_class_loc: default_max_class_loc(),
            max_class_methods: default_max_class_methods(),
            min_lcom4: default_min_lcom4(),
            max_god_class_index: default_max_god_class_index(),
            max_ai_tokens: default_max_ai_tokens(),
            max_ai_cognitive: default_max_ai_cognitive(),
            churn_window_days: default_churn_window_days(),
            churn_max_commits: default_churn_max_commits(),
            default_top_n: default_top_n(),
        }
    }
}

// ---------------------------------------------------------------------------
// Phase 2.6: Dead Code & Abandoned Component Detection Config
// ---------------------------------------------------------------------------

/// Project classification profile affecting entrypoint and visibility heuristics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProjectProfile {
    /// End-user application (CLI, backend server, web app). Public symbols without callers are dead.
    #[default]
    Application,
    /// Reusable library or SDK. Publicly exported symbols are presumed living roots.
    Library,
    /// Monorepo containing both internal applications and shared published libraries.
    Monorepo,
}

/// Category filtering for dead code analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DeadCodeCategoryFilter {
    #[default]
    All,
    Components,
    Routes,
    Models,
    Islands,
    Files,
    Functions,
    Methods,
    UnreadFields,
    UnusedVariants,
    BarrelExports,
    GhostDependencies,
    Deprecated,
    TestOrphans,
}

/// Project-level configuration for cartograph deadcode in cartograph.toml
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeadCodeConfig {
    #[serde(default)]
    pub profile: ProjectProfile,

    #[serde(default = "default_deadcode_min_lines")]
    pub min_lines: usize,

    #[serde(default = "default_deadcode_min_confidence")]
    pub min_confidence: f64,

    #[serde(default)]
    pub include_tests: bool,

    #[serde(default)]
    pub custom_roots: Vec<String>,

    #[serde(default = "default_deadcode_ignore_patterns")]
    pub ignore_patterns: Vec<String>,

    #[serde(default = "default_deadcode_frameworks")]
    pub frameworks: Vec<String>,

    #[serde(default)]
    pub ai_ignore_file: Option<std::path::PathBuf>,

    #[serde(default)]
    pub max_dead_loc: Option<usize>,
    #[serde(default)]
    pub max_wasted_tokens: Option<usize>,
    #[serde(default)]
    pub fail_on_new_deadcode: bool,
}

fn default_deadcode_min_lines() -> usize {
    5
}
fn default_deadcode_min_confidence() -> f64 {
    0.75
}
fn default_deadcode_ignore_patterns() -> Vec<String> {
    vec![
        "**/node_modules/**".to_string(),
        "**/target/**".to_string(),
        "**/vendor/**".to_string(),
        "**/.git/**".to_string(),
        "**/dist/**".to_string(),
        "**/build/**".to_string(),
    ]
}
fn default_deadcode_frameworks() -> Vec<String> {
    vec![
        "react".to_string(),
        "nextjs".to_string(),
        "vue".to_string(),
        "fastapi".to_string(),
        "django".to_string(),
        "rails".to_string(),
        "axum".to_string(),
        "actix".to_string(),
        "graphql".to_string(),
        "grpc".to_string(),
    ]
}

impl Default for DeadCodeConfig {
    fn default() -> Self {
        Self {
            profile: ProjectProfile::default(),
            min_lines: default_deadcode_min_lines(),
            min_confidence: default_deadcode_min_confidence(),
            include_tests: false,
            custom_roots: Vec::new(),
            ignore_patterns: default_deadcode_ignore_patterns(),
            frameworks: default_deadcode_frameworks(),
            ai_ignore_file: None,
            max_dead_loc: None,
            max_wasted_tokens: None,
            fail_on_new_deadcode: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Phase 3.1: Business Logic & Validation Rule Extractor (`cartograph extract-logic`)
// ---------------------------------------------------------------------------

/// Business logic and validation rule extraction settings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtractLogicConfig {
    /// Minimum confidence threshold (0.0 to 1.0) to filter technical boilerplate
    #[serde(default = "default_extract_logic_min_confidence")]
    pub min_confidence: f32,

    /// Default inter-procedural call tracing depth
    #[serde(default = "default_extract_logic_max_depth")]
    pub max_depth: usize,

    /// Whether to include technical hygiene checks (null checks, generic error wrappers)
    #[serde(default)]
    pub include_technical_guards: bool,

    /// Custom domain keywords to boost rule confidence
    #[serde(default = "default_extract_logic_domain_keywords")]
    pub domain_keywords: Vec<String>,

    /// Excluded symbol name globs (e.g. test mocks, harness utilities)
    #[serde(default = "default_extract_logic_ignore_symbols")]
    pub ignore_symbols: Vec<String>,

    /// Concept aliases and taxonomy synonyms (e.g. discount -> [rebate, coupon, promo])
    #[serde(default = "default_extract_logic_concept_aliases")]
    pub concept_aliases: HashMap<String, Vec<String>>,
}

fn default_extract_logic_min_confidence() -> f32 {
    0.60
}

fn default_extract_logic_max_depth() -> usize {
    3
}

fn default_extract_logic_domain_keywords() -> Vec<String> {
    vec![
        "discount".to_string(),
        "rebate".to_string(),
        "tax".to_string(),
        "fee".to_string(),
        "price".to_string(),
        "credit".to_string(),
        "balance".to_string(),
        "quota".to_string(),
        "limit".to_string(),
        "kyc".to_string(),
        "subscription".to_string(),
        "plan".to_string(),
        "rate".to_string(),
        "age".to_string(),
        "status".to_string(),
        "role".to_string(),
        "permission".to_string(),
    ]
}

fn default_extract_logic_ignore_symbols() -> Vec<String> {
    vec![
        "*_test".to_string(),
        "test_*".to_string(),
        "mock_*".to_string(),
        "Mock*".to_string(),
        "*Harness".to_string(),
    ]
}

fn default_extract_logic_concept_aliases() -> HashMap<String, Vec<String>> {
    let mut map = HashMap::new();
    map.insert(
        "discount".to_string(),
        vec![
            "rebate".to_string(),
            "coupon".to_string(),
            "promo".to_string(),
            "voucher".to_string(),
            "markdown".to_string(),
        ],
    );
    map.insert(
        "tax".to_string(),
        vec![
            "vat".to_string(),
            "gst".to_string(),
            "sales_tax".to_string(),
            "duty".to_string(),
            "withholding".to_string(),
        ],
    );
    map.insert(
        "auth".to_string(),
        vec![
            "role".to_string(),
            "permission".to_string(),
            "scope".to_string(),
            "tenant".to_string(),
            "acl".to_string(),
        ],
    );
    map.insert(
        "inventory".to_string(),
        vec![
            "stock".to_string(),
            "quantity".to_string(),
            "sku".to_string(),
            "warehouse".to_string(),
        ],
    );
    map
}

impl Default for ExtractLogicConfig {
    fn default() -> Self {
        Self {
            min_confidence: default_extract_logic_min_confidence(),
            max_depth: default_extract_logic_max_depth(),
            include_technical_guards: false,
            domain_keywords: default_extract_logic_domain_keywords(),
            ignore_symbols: default_extract_logic_ignore_symbols(),
            concept_aliases: default_extract_logic_concept_aliases(),
        }
    }
}

// ---------------------------------------------------------------------------
// Phase 3.2: Contract-Aware AI Reviewer Configuration
// ---------------------------------------------------------------------------

/// Contract-aware code review settings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiReviewConfig {
    /// Default execution mode: "offline" or "ai"
    #[serde(default = "default_ai_review_mode")]
    pub mode: String,

    /// AI provider: "anthropic", "openai", "gemini", "ollama", "custom"
    #[serde(default = "default_ai_review_provider")]
    pub provider: String,

    /// Model name
    #[serde(default = "default_ai_review_model")]
    pub model: String,

    /// Optional API base endpoint (e.g. for Ollama or LiteLLM)
    #[serde(default)]
    pub api_base: Option<String>,

    /// Sampling temperature (lower for deterministic code review)
    #[serde(default = "default_ai_review_temperature")]
    pub temperature: f32,

    /// Maximum token budget for completions
    #[serde(default = "default_ai_review_max_tokens")]
    pub max_tokens: usize,

    /// Fallback to offline deterministic engine if API call fails or times out
    #[serde(default = "default_true")]
    pub fallback_to_offline: bool,

    /// Default target branch if omitted
    #[serde(default = "default_ai_review_target")]
    pub default_target: String,

    /// CI gating severities (e.g. ["blocker"])
    #[serde(default = "default_ai_review_fail_on")]
    pub fail_on: Vec<String>,

    /// Fail if public SemVer breaking change detected
    #[serde(default = "default_true")]
    pub fail_on_breaking: bool,

    /// Fail if unhandled downstream side-effects detected
    #[serde(default = "default_true")]
    pub fail_on_side_effects: bool,

    /// Fail if untested functional symbols exceed threshold
    #[serde(default)]
    pub fail_on_untested: bool,

    /// Minimum test coverage ratio (0.0 to 1.0)
    #[serde(default = "default_ai_review_min_coverage")]
    pub min_test_coverage: f64,

    /// Maximum tolerable PR risk score before failing (0.0 to 100.0)
    #[serde(default = "default_ai_review_max_risk_score")]
    pub max_risk_score: f64,

    /// Cognitive complexity delta spike threshold (e.g. 15)
    #[serde(default = "default_ai_review_max_complexity_delta")]
    pub max_complexity_delta: i32,

    /// Call-graph slicing depth for downstream caller discovery
    #[serde(default = "default_ai_review_caller_depth")]
    pub caller_depth: usize,

    /// Maximum downstream call-site snippets to include per modified symbol
    #[serde(default = "default_ai_review_max_call_sites")]
    pub max_call_sites: usize,

    /// Whether to track review findings across multi-commit pushes
    #[serde(default = "default_true")]
    pub review_memory: bool,

    /// Path to review history file
    #[serde(default = "default_ai_review_history_path")]
    pub history_path: String,

    /// Whether to generate Keep-a-Changelog Markdown and semantic PR summary
    #[serde(default)]
    pub summarize: bool,

    /// Fast mode for pre-commit hooks (<10ms): 1st-hop only, skips LLM and deep sweeps
    #[serde(default)]
    pub fast: bool,

    /// Verify iteration delta against previous review round (highlighting resolved findings and new regressions)
    #[serde(default)]
    pub verify_fixes: bool,

    /// Publisher configuration
    #[serde(default)]
    pub publishers: AiReviewPublishersConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AiReviewPublishersConfig {
    #[serde(default)]
    pub git_notes: bool,

    #[serde(default)]
    pub webhook_url: Option<String>,

    #[serde(default)]
    pub github: Option<PlatformPublisherConfig>,

    #[serde(default)]
    pub gitlab: Option<PlatformPublisherConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformPublisherConfig {
    #[serde(default = "default_true")]
    pub auto_detect: bool,

    #[serde(default = "default_true")]
    pub inline_comments: bool,

    #[serde(default = "default_true")]
    pub step_summary: bool,
}

impl Default for PlatformPublisherConfig {
    fn default() -> Self {
        Self {
            auto_detect: true,
            inline_comments: true,
            step_summary: true,
        }
    }
}

fn default_ai_review_mode() -> String {
    "ai".to_string()
}

fn default_ai_review_provider() -> String {
    "anthropic".to_string()
}

fn default_ai_review_model() -> String {
    "claude-3-7-sonnet".to_string()
}

fn default_ai_review_temperature() -> f32 {
    0.1
}

fn default_ai_review_max_tokens() -> usize {
    2048
}

fn default_ai_review_target() -> String {
    "origin/main".to_string()
}

fn default_ai_review_fail_on() -> Vec<String> {
    vec!["blocker".to_string()]
}

fn default_ai_review_min_coverage() -> f64 {
    0.80
}

fn default_ai_review_max_risk_score() -> f64 {
    80.0
}

fn default_ai_review_max_complexity_delta() -> i32 {
    15
}

fn default_ai_review_caller_depth() -> usize {
    2
}

fn default_ai_review_max_call_sites() -> usize {
    8
}

fn default_ai_review_history_path() -> String {
    ".cartograph/review_history.json".to_string()
}

impl Default for AiReviewConfig {
    fn default() -> Self {
        Self {
            mode: default_ai_review_mode(),
            provider: default_ai_review_provider(),
            model: default_ai_review_model(),
            api_base: None,
            temperature: default_ai_review_temperature(),
            max_tokens: default_ai_review_max_tokens(),
            fallback_to_offline: true,
            default_target: default_ai_review_target(),
            fail_on: default_ai_review_fail_on(),
            fail_on_breaking: true,
            fail_on_side_effects: true,
            fail_on_untested: false,
            min_test_coverage: default_ai_review_min_coverage(),
            max_risk_score: default_ai_review_max_risk_score(),
            max_complexity_delta: default_ai_review_max_complexity_delta(),
            caller_depth: default_ai_review_caller_depth(),
            max_call_sites: default_ai_review_max_call_sites(),
            review_memory: true,
            history_path: default_ai_review_history_path(),
            summarize: false,
            fast: false,
            verify_fixes: false,
            publishers: AiReviewPublishersConfig::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// Phase 3.3: Topological Root-Cause Detective (`cartograph diagnose`)
// ---------------------------------------------------------------------------

/// Configuration for Topological Root-Cause Detective (cartograph diagnose)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiagnoseConfig {
    /// Offline deterministic mode (zero-LLM, <15ms)
    #[serde(default = "default_true")]
    pub offline: bool,

    /// Deep mode: traverse callers beyond stack frames into upstream orchestrators
    #[serde(default)]
    pub deep: bool,

    /// Weight lines by git blame recency
    #[serde(default = "default_true")]
    pub git_blame: bool,

    /// Recency decay window in days (default 30)
    #[serde(default = "default_diagnose_recency_days")]
    pub recency_days: u64,

    /// Correlate with active branch/PR diff (e.g. "origin/main")
    #[serde(default)]
    pub branch: Option<String>,

    /// AI provider: "anthropic", "openai", "gemini", "ollama", "custom"
    #[serde(default = "default_diagnose_provider")]
    pub provider: String,

    /// Model name
    #[serde(default = "default_diagnose_model")]
    pub model: String,

    /// Optional API base endpoint
    #[serde(default)]
    pub api_base: Option<String>,

    /// Fallback to offline deterministic engine if AI call fails
    #[serde(default = "default_true")]
    pub fallback_to_offline: bool,

    /// Export unified git patch to file
    #[serde(default)]
    pub export_patch: Option<std::path::PathBuf>,

    /// Safe single-shot git apply of patch
    #[serde(default)]
    pub apply_patch: bool,

    /// Synthesize framework-native regression test reproducer
    #[serde(default)]
    pub scaffold_test: bool,

    /// Output format: "terminal", "markdown", "sarif", "json", "xml"
    #[serde(default = "default_diagnose_format")]
    pub format: String,
}

fn default_diagnose_recency_days() -> u64 {
    30
}

fn default_diagnose_provider() -> String {
    "anthropic".to_string()
}

fn default_diagnose_model() -> String {
    "claude-3-7-sonnet".to_string()
}

fn default_diagnose_format() -> String {
    "terminal".to_string()
}

impl Default for DiagnoseConfig {
    fn default() -> Self {
        Self {
            offline: true,
            deep: false,
            git_blame: true,
            recency_days: default_diagnose_recency_days(),
            branch: None,
            provider: default_diagnose_provider(),
            model: default_diagnose_model(),
            api_base: None,
            fallback_to_offline: true,
            export_patch: None,
            apply_patch: false,
            scaffold_test: false,
            format: default_diagnose_format(),
        }
    }
}

// ---------------------------------------------------------------------------
// Phase 2.7: Master Unified Project Configuration (`cartograph.toml`)
// ---------------------------------------------------------------------------

/// Project identity and classification metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectInfoConfig {
    /// Human-readable project name
    #[serde(default = "default_project_name")]
    pub name: String,

    /// Project classification profile: application, library, or monorepo
    #[serde(default)]
    pub profile: ProjectProfile,

    /// Optional semantic version string
    #[serde(default)]
    pub version: Option<String>,

    /// Project authors or maintainers
    #[serde(default)]
    pub authors: Vec<String>,

    /// Brief description of the project
    #[serde(default)]
    pub description: Option<String>,
}

fn default_project_name() -> String {
    "my-project".to_string()
}

impl Default for ProjectInfoConfig {
    fn default() -> Self {
        Self {
            name: default_project_name(),
            profile: ProjectProfile::default(),
            version: None,
            authors: Vec::new(),
            description: None,
        }
    }
}

/// Master configuration container mapping 1:1 to cartograph.toml
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ProjectConfig {
    /// Project metadata
    #[serde(default)]
    pub project: ProjectInfoConfig,

    /// Primary storage backend settings
    #[serde(default)]
    pub storage: StorageConfig,

    /// Multi-database named profiles (--db <profile>)
    #[serde(default)]
    pub profiles: ProfileConfig,

    /// AST file discovery and parallel indexing settings
    #[serde(default)]
    pub index: IndexConfig,

    /// Real-time file watcher settings
    #[serde(default)]
    pub watch: WatchConfig,

    /// Predictive test selector settings and custom runners
    #[serde(default)]
    pub tests: TestConfig,

    /// Architectural boundary, cycle, and layer linting
    #[serde(default)]
    pub lint_arch: LintArchConfig,

    /// Code complexity, God-class, and churn hotspot heatmap settings
    #[serde(default)]
    pub complexity: ComplexityConfig,

    /// Dead code, abandoned component, and ghost dependency settings
    #[serde(default)]
    pub deadcode: DeadCodeConfig,

    /// Business logic, validation rule, and invariant extraction settings
    #[serde(default)]
    pub extract_logic: ExtractLogicConfig,

    /// Contract-aware code review settings
    #[serde(default)]
    pub ai_review: AiReviewConfig,

    /// Topological root-cause detective and crash analyzer settings
    #[serde(default)]
    pub diagnose: DiagnoseConfig,
}

impl ProjectConfig {
    /// Load and deserialize configuration from a specific file path
    pub fn load_from_path(path: &std::path::Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file at {}: {e}", path.display()))?;
        toml::from_str(&content)
            .map_err(|e| format!("Invalid TOML syntax in {}: {e}", path.display()))
    }

    /// Search for cartograph.toml or .cartograph/config.toml in workspace root.
    /// If not found or invalid, returns default configuration.
    pub fn find_and_load(workspace_root: &std::path::Path) -> Self {
        let primary = workspace_root.join("cartograph.toml");
        if primary.is_file() {
            match Self::load_from_path(&primary) {
                Ok(cfg) => return cfg,
                Err(err) => {
                    eprintln!(
                        "⚠️  [cartograph] Warning loading cartograph.toml: {err}. Using defaults."
                    );
                }
            }
        }

        let secondary = workspace_root.join(".cartograph/config.toml");
        if secondary.is_file() {
            match Self::load_from_path(&secondary) {
                Ok(cfg) => return cfg,
                Err(err) => {
                    eprintln!(
                        "⚠️  [cartograph] Warning loading .cartograph/config.toml: {err}. Using defaults."
                    );
                }
            }
        }

        Self::default()
    }

    /// Serialize this configuration to a formatted TOML string
    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    /// Generate a fully documented, comprehensive cartograph.toml starter template
    pub fn generate_template(project_name: &str, preset: Option<&str>, minimal: bool) -> String {
        let profile = if preset.is_some_and(|p| p.eq_ignore_ascii_case("monorepo")) {
            "monorepo"
        } else {
            "application"
        };

        if minimal {
            return format!(
                r#"# 🗺️ Cartograph Minimal Configuration (cartograph.toml)

[project]
name = "{project_name}"
profile = "{profile}"

[storage]
backend = "sqlite"

[storage.sqlite]
path = ".cartograph/cartograph.db"
"#
            );
        }

        let arch_preset_comment = match preset {
            Some(p) => format!("preset = \"{p}\""),
            None => "# preset = \"clean-architecture\"".to_string(),
        };

        let layers_block = match preset
            .unwrap_or("clean-architecture")
            .to_lowercase()
            .as_str()
        {
            "hexagonal" => {
                r#"[[lint_arch.layers]]
name = "domain"
description = "Domain core business logic and entities"
patterns = ["**/domain/**", "**/core/**"]
forbidden = ["adapters"]

[[lint_arch.layers]]
name = "ports"
description = "Inbound and outbound interfaces/ports"
patterns = ["**/ports/**"]
forbidden = ["adapters"]

[[lint_arch.layers]]
name = "adapters"
description = "Inbound HTTP/CLI and outbound DB/Messaging adapters"
patterns = ["**/adapters/**", "**/infra/**"]
forbidden = []"#
            }
            "mvc" => {
                r#"[[lint_arch.layers]]
name = "models"
description = "Data entities and database models"
patterns = ["**/models/**"]
forbidden = ["controllers", "views"]

[[lint_arch.layers]]
name = "views"
description = "Templates and UI presentation components"
patterns = ["**/views/**", "**/templates/**"]
forbidden = ["controllers"]

[[lint_arch.layers]]
name = "controllers"
description = "Request handlers and orchestration"
patterns = ["**/controllers/**", "**/handlers/**"]
forbidden = []"#
            }
            _ => {
                r#"[[lint_arch.layers]]
name = "domain"
description = "Core business entities and enterprise rules"
patterns = ["**/domain/**", "**/models/**"]
forbidden = ["application", "infrastructure", "presentation"]

[[lint_arch.layers]]
name = "application"
description = "Use cases, domain services, and orchestrators"
patterns = ["**/services/**", "**/usecases/**"]
forbidden = ["infrastructure", "presentation"]

[[lint_arch.layers]]
name = "infrastructure"
description = "Database, HTTP clients, message brokers, external APIs"
patterns = ["**/db/**", "**/adapters/**", "**/repositories/**"]
forbidden = ["presentation"]

[[lint_arch.layers]]
name = "presentation"
description = "HTTP controllers, GraphQL resolvers, CLI commands"
patterns = ["**/controllers/**", "**/routes/**", "**/cli/**"]
forbidden = []"#
            }
        };

        format!(
            r#"# 🗺️ Cartograph Configuration Specification (cartograph.toml)
# Documentation: https://github.com/PankajRedekar/cartograph

[project]
name = "{project_name}"
profile = "{profile}"              # "application" | "library" | "monorepo"

[storage]
backend = "sqlite"                 # "sqlite" | "memory"

[storage.sqlite]
path = ".cartograph/cartograph.db"
mmap_size = 268435456              # 256MB memory-mapped I/O
busy_timeout_ms = 5000
journal_mode = "WAL"
synchronous = "NORMAL"

[index]
exclude_patterns = [
    "**/.git",
    "**/.cartograph",
    "**/node_modules",
    "**/vendor",
    "**/target",
    "**/dist",
    "**/build",
    "**/.venv",
    "**/__pycache__"
]
max_file_size_bytes = 2097152      # 2MB
mmap_threshold_bytes = 65536       # 64KB
follow_symlinks = false

[watch]
debounce_ms = 25                   # Milliseconds to debounce multiple file saves
cst_cache_capacity = 500           # AST CST parsing cache limit
track_blast_radius = true          # Calculate downstream blast radius on save

[tests]
granularity = "auto"               # "auto" | "case" | "suite" | "file" | "package"
min_confidence = 0.5               # Minimum edge confidence for test selection
max_depth = 5                      # Maximum transitive hop depth
ignore_docstrings = true           # Skip all tests if change is comments/docs only
fail_on_untested = false           # Gate CI if modified code has 0 tests
min_coverage = 0.0                 # Target test coverage percentage

[tests.runners.cargo]
command = "cargo test --test {{test_file}} -- {{test_case}}"

[tests.runners.pytest]
command = "pytest {{test_file}}::{{test_case}}"

[tests.runners.jest]
command = "npx jest {{test_file}} -t '{{test_case}}'"

[lint_arch]
enabled = true
{arch_preset_comment}
detect_cycles = true
cycle_granularity = "package"      # "file" | "module" | "package" | "symbol"
coupling = "runtime"               # "runtime" | "type_only" | "all"
honor_arch_tags = true             # Honor in-code /// @arch tags
fail_on = "error"                  # "error" | "warning" | "never"
strict = false
max_violations = 0

[lint_arch.package_encapsulation]
enabled = true
package_roots = ["*-service", "crates/*", "packages/*", "apps/*"]
internal_patterns = ["**/internal/**", "**/private/**"]
entrypoints = ["lib.rs", "index.ts", "index.js", "mod.rs", "__init__.py"]

{layers_block}

[lint_arch.exemptions]
ignore_paths = ["**/tests/**", "**/fixtures/**", "**/benches/**"]

[complexity]
weight_cyclomatic = 0.35
weight_cognitive = 0.25
weight_loc = 0.15
weight_coupling = 0.15
weight_churn = 0.10

max_function_cyclomatic = 15
max_function_cognitive = 20
max_function_loc = 100
max_nesting_depth = 4
max_parameters = 5

max_class_wmc = 50
max_class_loc = 500
max_class_methods = 20
min_lcom4 = 2
max_god_class_index = 75.0

max_ai_tokens = 2500
max_ai_cognitive = 30
churn_window_days = 90
churn_max_commits = 100
default_top_n = 15

[deadcode]
profile = "{profile}"
min_lines = 5                      # Skip stubs under 5 lines
min_confidence = 0.75              # Confidence threshold (0.0 - 1.0)
include_tests = false              # Production reachability only
custom_roots = []                  # Extra entrypoint globs
ignore_patterns = [
    "**/node_modules/**",
    "**/target/**",
    "**/vendor/**",
    "**/.git/**"
]
frameworks = [
    "react",
    "nextjs",
    "vue",
    "fastapi",
    "django",
    "rails",
    "axum",
    "actix"
]
ai_ignore_file = ".contextignore"
fail_on_new_deadcode = false

[extract_logic]
min_confidence = 0.60              # Filter low-level technical boilerplate
max_depth = 3                      # Inter-procedural call tracing depth
include_technical_guards = false   # Keep false to extract pure domain logic
domain_keywords = [
    "discount", "rebate", "tax", "fee", "price", "credit",
    "balance", "quota", "limit", "kyc", "subscription", "plan"
]
ignore_symbols = ["*_test", "test_*", "mock_*", "Mock*", "*Harness"]

[extract_logic.concept_aliases]
discount = ["rebate", "coupon", "promo", "voucher", "markdown"]
tax = ["vat", "gst", "sales_tax", "duty"]
auth = ["role", "permission", "tenant", "privilege"]
inventory = ["stock", "quantity", "sku", "warehouse"]
"#
        )
    }
}

#[cfg(test)]
mod project_config_tests {
    use super::*;

    #[test]
    fn test_project_config_defaults() {
        let config = ProjectConfig::default();
        assert_eq!(config.project.name, "my-project");
        assert_eq!(config.storage.backend, "sqlite");
        assert_eq!(config.complexity.max_function_cyclomatic, 15);
        assert_eq!(config.deadcode.min_lines, 5);
        assert_eq!(config.tests.max_depth, 5);
        assert!(config.lint_arch.detect_cycles);
    }

    #[test]
    fn test_project_config_template_parsing() {
        let template =
            ProjectConfig::generate_template("test-service", Some("clean-architecture"), false);
        let parsed: ProjectConfig = toml::from_str(&template).expect("Template must be valid TOML");
        assert_eq!(parsed.project.name, "test-service");
        assert_eq!(parsed.storage.backend, "sqlite");
        assert_eq!(parsed.lint_arch.layers.len(), 4);
    }

    #[test]
    fn test_project_config_minimal_template() {
        let template = ProjectConfig::generate_template("mini-app", None, true);
        let parsed: ProjectConfig =
            toml::from_str(&template).expect("Minimal template must be valid TOML");
        assert_eq!(parsed.project.name, "mini-app");
        assert_eq!(parsed.storage.backend, "sqlite");
    }

    #[test]
    fn test_project_config_roundtrip_serialization() {
        let config = ProjectConfig::default();
        let toml_str = config.to_toml_string().expect("Must serialize to TOML");
        let parsed: ProjectConfig = toml::from_str(&toml_str).expect("Must deserialize from TOML");
        assert_eq!(config, parsed);
    }
}
