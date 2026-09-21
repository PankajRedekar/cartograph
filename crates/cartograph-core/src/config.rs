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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
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

