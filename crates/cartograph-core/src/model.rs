use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    Function,
    Method,
    Class,
    Struct,
    Interface,
    Module,
    File,
    Route,
    Model,
    Component,
    Variable,
    Constant,
    Enum,
    Macro,
    #[serde(untagged)]
    Other(String),
}

impl NodeKind {
    pub fn as_str(&self) -> &str {
        match self {
            NodeKind::Function => "function",
            NodeKind::Method => "method",
            NodeKind::Class => "class",
            NodeKind::Struct => "struct",
            NodeKind::Interface => "interface",
            NodeKind::Module => "module",
            NodeKind::File => "file",
            NodeKind::Route => "route",
            NodeKind::Model => "model",
            NodeKind::Component => "component",
            NodeKind::Variable => "variable",
            NodeKind::Constant => "constant",
            NodeKind::Enum => "enum",
            NodeKind::Macro => "macro",
            NodeKind::Other(s) => s.as_str(),
        }
    }
}

impl fmt::Display for NodeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for NodeKind {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let kind = match s.to_lowercase().as_str() {
            "function" => NodeKind::Function,
            "method" => NodeKind::Method,
            "class" => NodeKind::Class,
            "struct" => NodeKind::Struct,
            "interface" => NodeKind::Interface,
            "module" => NodeKind::Module,
            "file" => NodeKind::File,
            "route" => NodeKind::Route,
            "model" => NodeKind::Model,
            "component" => NodeKind::Component,
            "variable" => NodeKind::Variable,
            "constant" => NodeKind::Constant,
            "enum" => NodeKind::Enum,
            "macro" => NodeKind::Macro,
            other => NodeKind::Other(other.to_string()),
        };
        Ok(kind)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeKind {
    Imports,
    Calls,
    Extends,
    Implements,
    Defines,
    References,
    Contains,
    Returns,
    ParameterOf,
    Tests,
    MapsTo,
    #[serde(untagged)]
    Other(String),
}

impl EdgeKind {
    pub fn as_str(&self) -> &str {
        match self {
            EdgeKind::Imports => "imports",
            EdgeKind::Calls => "calls",
            EdgeKind::Extends => "extends",
            EdgeKind::Implements => "implements",
            EdgeKind::Defines => "defines",
            EdgeKind::References => "references",
            EdgeKind::Contains => "contains",
            EdgeKind::Returns => "returns",
            EdgeKind::ParameterOf => "parameter_of",
            EdgeKind::Tests => "tests",
            EdgeKind::MapsTo => "maps_to",
            EdgeKind::Other(s) => s.as_str(),
        }
    }
}

impl fmt::Display for EdgeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for EdgeKind {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let kind = match s.to_lowercase().as_str() {
            "imports" => EdgeKind::Imports,
            "calls" => EdgeKind::Calls,
            "extends" => EdgeKind::Extends,
            "implements" => EdgeKind::Implements,
            "defines" => EdgeKind::Defines,
            "references" => EdgeKind::References,
            "contains" => EdgeKind::Contains,
            "returns" => EdgeKind::Returns,
            "parameter_of" => EdgeKind::ParameterOf,
            "tests" => EdgeKind::Tests,
            "maps_to" | "mapsto" => EdgeKind::MapsTo,
            other => EdgeKind::Other(other.to_string()),
        };
        Ok(kind)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub name: String,
    pub kind: NodeKind,
    pub filepath: String,
    pub scope: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_line: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_col: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_line: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_col: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docstring: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Edge {
    pub source_id: String,
    pub target_id: String,
    pub kind: EdgeKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,
}

impl Default for Edge {
    fn default() -> Self {
        Self {
            source_id: String::new(),
            target_id: String::new(),
            kind: EdgeKind::Calls,
            metadata: None,
            confidence: Some(1.0),
            weight: Some(1.0),
        }
    }
}

impl Edge {
    pub fn new(source_id: impl Into<String>, target_id: impl Into<String>, kind: EdgeKind) -> Self {
        Self {
            source_id: source_id.into(),
            target_id: target_id.into(),
            kind,
            metadata: None,
            confidence: Some(1.0),
            weight: Some(1.0),
        }
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = Some(confidence);
        self
    }

    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = Some(weight);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchResult {
    pub node: Node,
    pub score: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched_field: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snippet: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct GraphStats {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub total_files: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticChangeType {
    DocstringOnly,
    BodyOnly,
    VisibilityReduced,
    SignatureBreak,
    Deleted,
    Added,
}

impl SemanticChangeType {
    pub fn risk_multiplier(&self) -> f64 {
        match self {
            SemanticChangeType::DocstringOnly => 0.0,
            SemanticChangeType::BodyOnly => 0.6,
            SemanticChangeType::VisibilityReduced => 0.9,
            SemanticChangeType::SignatureBreak => 1.0,
            SemanticChangeType::Deleted => 1.0,
            SemanticChangeType::Added => 0.3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SemVerTarget {
    Patch,
    Minor,
    Major,
}

impl SemVerTarget {
    pub fn as_str(&self) -> &'static str {
        match self {
            SemVerTarget::Patch => "patch",
            SemVerTarget::Minor => "minor",
            SemVerTarget::Major => "major",
        }
    }
}

impl std::str::FromStr for SemVerTarget {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "patch" => Ok(SemVerTarget::Patch),
            "minor" => Ok(SemVerTarget::Minor),
            "major" => Ok(SemVerTarget::Major),
            other => Err(format!("Unknown SemVer target '{other}', expected patch, minor, or major")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BreakingChangeDetail {
    pub symbol_id: String,
    pub symbol_name: String,
    pub filepath: String,
    pub line: u32,
    pub reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_signature: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemVerClassification {
    pub expected_policy: SemVerTarget,
    pub actual_required_tier: SemVerTarget,
    pub is_compliant: bool,
    pub breaking_changes: Vec<BreakingChangeDetail>,
    pub explanation: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArchitecturalCouplingSmell {
    pub symbol_id: String,
    pub symbol_name: String,
    pub in_degree: usize,
    pub subsystems_impacted: Vec<String>,
    pub coupling_ratio: f64,
    pub recommendation: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl RiskLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            RiskLevel::Low => "low",
            RiskLevel::Medium => "medium",
            RiskLevel::High => "high",
            RiskLevel::Critical => "critical",
        }
    }
}

impl std::str::FromStr for RiskLevel {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "low" => Ok(RiskLevel::Low),
            "medium" => Ok(RiskLevel::Medium),
            "high" => Ok(RiskLevel::High),
            "critical" => Ok(RiskLevel::Critical),
            other => Err(format!("Unknown risk level '{other}', expected low, medium, high, or critical")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeStatus {
    Modified,
    Added,
    Deleted,
    Renamed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImpactRoot {
    pub node: Node,
    pub status: ChangeStatus,
    pub semantic_change: SemanticChangeType,
    pub is_public_export: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_lines: Option<Vec<u32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_signature: Option<String>,
    pub in_degree: usize,
    pub out_degree: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CallSiteLocation {
    pub caller_node_id: String,
    pub filepath: String,
    pub line: u32,
    pub column: u32,
    pub invocation_snippet: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remediation_hint: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImpactedNode {
    pub node: Node,
    pub min_hop_distance: usize,
    pub impact_probability: f64,
    pub direct_causes: Vec<String>,
    pub edge_kinds_to_root: std::collections::HashSet<EdgeKind>,
    pub call_sites: Vec<CallSiteLocation>,
    pub is_tested: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PropagationPath {
    pub root_id: String,
    pub target_id: String,
    pub hops: usize,
    pub path_node_ids: Vec<String>,
    pub path_edge_kinds: Vec<EdgeKind>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RiskBreakdown {
    pub raw_score: f64,
    pub semantic_multiplier: f64,
    pub total_score: f64,
    pub level: RiskLevel,
    pub scale_score: f64,
    pub centrality_score: f64,
    pub entrypoint_score: f64,
    pub data_model_score: f64,
    pub boundary_score: f64,
    pub test_deficit_score: f64,
    pub primary_risk_drivers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImpactedTest {
    pub test_node: Node,
    pub targets_tested: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runner_command: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CategorizedImpact {
    pub entrypoints: Vec<ImpactedNode>,
    pub services: Vec<ImpactedNode>,
    pub data_models: Vec<ImpactedNode>,
    pub other_internals: Vec<ImpactedNode>,
    pub affected_tests: Vec<ImpactedTest>,
    pub untested_symbols: Vec<ImpactedNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SimulationMode {
    None,
    SimulateDelete(String),
    SimulateRename(String, String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawImpactRecord {
    pub node: Node,
    pub root_id: String,
    pub depth: usize,
    pub path_str: String,
    pub edge_kind: EdgeKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImpactReport {
    pub query_target: Option<String>,
    pub git_diff_range: Option<String>,
    pub simulation_mode: SimulationMode,
    pub traversal_direction: String,
    pub max_depth: usize,
    pub root_nodes: Vec<ImpactRoot>,
    pub total_affected_nodes: usize,
    pub total_affected_files: usize,
    pub total_call_sites: usize,
    pub risk: RiskBreakdown,
    pub semver_guard: Option<SemVerClassification>,
    pub coupling_insights: Vec<ArchitecturalCouplingSmell>,
    pub categories: CategorizedImpact,
    pub paths_to_entrypoints: Vec<PropagationPath>,
    pub test_coverage_ratio: f64,
    pub elapsed_ms: f64,
}

/// Granularity of test selection and execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TestGranularity {
    /// Auto-detect optimal granularity based on runner and impacted ratio
    Auto,
    /// Exact test function or test case (e.g. test_calc_interest)
    Case,
    /// Test class or describe block (e.g. TestBillingService)
    Suite,
    /// Entire test file (e.g. tests/test_billing.py)
    File,
    /// Entire test package or directory (e.g. crates/billing/...)
    Package,
}

impl TestGranularity {
    pub fn as_str(&self) -> &str {
        match self {
            TestGranularity::Auto => "auto",
            TestGranularity::Case => "case",
            TestGranularity::Suite => "suite",
            TestGranularity::File => "file",
            TestGranularity::Package => "package",
        }
    }
}

impl std::str::FromStr for TestGranularity {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "auto" => Ok(TestGranularity::Auto),
            "case" => Ok(TestGranularity::Case),
            "suite" => Ok(TestGranularity::Suite),
            "file" => Ok(TestGranularity::File),
            "package" => Ok(TestGranularity::Package),
            other => Err(format!("Unknown test granularity: '{other}'. Use auto, case, suite, file, or package.")),
        }
    }
}

/// Supported test runner engines
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TestRunnerKind {
    CargoTest,
    CargoNextest,
    Pytest,
    Jest,
    Vitest,
    Mocha,
    GoTest,
    RSpec,
    Custom(String),
}

impl TestRunnerKind {
    pub fn as_str(&self) -> &str {
        match self {
            TestRunnerKind::CargoTest => "cargo",
            TestRunnerKind::CargoNextest => "cargo-nextest",
            TestRunnerKind::Pytest => "pytest",
            TestRunnerKind::Jest => "jest",
            TestRunnerKind::Vitest => "vitest",
            TestRunnerKind::Mocha => "mocha",
            TestRunnerKind::GoTest => "go test",
            TestRunnerKind::RSpec => "rspec",
            TestRunnerKind::Custom(s) => s.as_str(),
        }
    }
}

/// Test execution classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TestCategory {
    Unit,
    Integration,
    E2E,
    Benchmark,
    Unknown,
}

/// A selected test case or test suite with provenance metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectedTest {
    /// The AST node representing the test function or suite
    pub test_node: Node,
    /// Detected test runner framework
    pub runner_kind: TestRunnerKind,
    /// Test classification (unit, integration, e2e)
    pub category: TestCategory,
    /// Granularity at which this test is addressed
    pub granularity: TestGranularity,
    /// Minimum graph distance (hop count) from modified root symbol
    /// 1 = direct test, 2+ = transitive test
    pub hop_distance: usize,
    /// Computed selection confidence / relevance score (0.0 to 1.0)
    pub relevance_score: f64,
    /// Fail-fast priority ranking score (higher = run earlier)
    pub priority_score: f64,
    /// Target symbols verified by this test
    pub targets_verified: Vec<String>,
    /// Step-by-step propagation path from target symbol to test node
    pub propagation_path: Vec<String>,
    /// Synthesized standalone command to run this specific test
    pub standalone_command: String,
    /// Estimated execution duration in milliseconds (if available from history)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_duration_ms: Option<u64>,
    /// Historical failure rate (0.0 to 1.0) from previous test runs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub historical_failure_rate: Option<f64>,
}

/// Grouped runner invocation batch
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestRunnerInvocation {
    /// Runner framework
    pub runner_kind: TestRunnerKind,
    /// Working directory for execution
    pub working_dir: String,
    /// Executable binary (e.g. "cargo", "pytest", "npx")
    pub program: String,
    /// Command line arguments
    pub args: Vec<String>,
    /// Full display string
    pub display_command: String,
    /// Tests included in this invocation
    pub included_tests: Vec<String>,
    /// Target files executed
    pub target_files: Vec<String>,
}

/// Information about changed symbols that have no test coverage
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UntestedSymbolDeficit {
    /// Node that was modified
    pub node: Node,
    /// Why this deficit is critical (e.g. "Public API exported symbol")
    pub reason: String,
    /// Recommended test file path for adding coverage
    pub suggested_test_file: String,
    /// Scaffolding template snippet for the test
    pub suggested_test_template: String,
}

/// Raw test record directly returned from database graph queries
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawTestRecord {
    pub test_node: Node,
    pub target_symbol_id: String,
    pub hop_distance: usize,
    pub path_str: String,
    pub confidence: f64,
}

/// Complete Predictive Test Selection Report
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestSelectionReport {
    /// Repository root path
    pub repo_root: String,
    /// Number of modified root AST nodes analyzed
    pub modified_symbols_count: usize,
    /// Total test cases selected
    pub total_tests_selected: usize,
    /// Total test files touched
    pub total_test_files_count: usize,
    /// Total test cases in repository (for savings calculation)
    pub total_repo_tests_count: usize,
    /// Percentage of repository test suite avoided (e.g. 94.2%)
    pub test_reduction_percentage: f64,
    /// Estimated time savings in seconds
    pub estimated_time_saved_seconds: f64,
    /// Selected tests ordered by priority (highest priority first)
    pub selected_tests: Vec<SelectedTest>,
    /// Synthesized executable runner invocations
    pub runner_invocations: Vec<TestRunnerInvocation>,
    /// Untested changed symbols (coverage gap)
    pub untested_deficits: Vec<UntestedSymbolDeficit>,
    /// Test coverage ratio: tested changed symbols / total changed symbols (0.0 to 1.0)
    pub test_coverage_ratio: f64,
    /// Whether all changes were non-functional comments/docstrings, resulting in zero tests
    pub is_zero_risk_bypass: bool,
    /// Number of symbols classified as docstring-only (pruned from test selection)
    pub docstring_only_symbols_count: usize,
    /// Number of active functional symbols driving test selection
    pub active_functional_symbols_count: usize,
    /// Total duration of test selection analysis in milliseconds (<15ms)
    pub analysis_duration_ms: f64,
}

// =========================================================================
// Phase 2.3: Full-Stack Route Mapper Data Models
// =========================================================================

/// HTTP Methods supported by standard web frameworks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Options,
    Head,
    Trace,
    Connect,
    /// Wildcard or all methods (e.g. Express app.all, Django view without method guard)
    All,
}

impl HttpMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Patch => "PATCH",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Options => "OPTIONS",
            HttpMethod::Head => "HEAD",
            HttpMethod::Trace => "TRACE",
            HttpMethod::Connect => "CONNECT",
            HttpMethod::All => "ALL",
        }
    }

    /// Whether this HTTP method typically implies data mutation
    pub fn is_mutation(&self) -> bool {
        matches!(
            self,
            HttpMethod::Post | HttpMethod::Put | HttpMethod::Patch | HttpMethod::Delete
        )
    }
}

impl fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for HttpMethod {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(HttpMethod::Get),
            "POST" => Ok(HttpMethod::Post),
            "PUT" => Ok(HttpMethod::Put),
            "PATCH" => Ok(HttpMethod::Patch),
            "DELETE" => Ok(HttpMethod::Delete),
            "OPTIONS" => Ok(HttpMethod::Options),
            "HEAD" => Ok(HttpMethod::Head),
            "TRACE" => Ok(HttpMethod::Trace),
            "CONNECT" => Ok(HttpMethod::Connect),
            "ALL" | "*" => Ok(HttpMethod::All),
            other => Err(format!("Unknown HTTP method: {other}")),
        }
    }
}

/// Authentication and authorization classifications for routes
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteAuthKind {
    /// Explicitly public / unauthenticated endpoint
    Public,
    /// Protected by session, JWT, or bearer token guard
    Protected,
    /// Protected by API Key header
    ApiKey,
    /// OAuth2 / OIDC scope guarded
    OAuth2(Vec<String>),
    /// Custom middleware or decorator authentication
    Custom(String),
    /// Authentication state cannot be determined with certainty
    Unknown,
}

/// Location of a route parameter in the HTTP request
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParamLocation {
    Path,
    Query,
    Header,
    Cookie,
    Body,
}

/// A parsed parameter associated with an HTTP route
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RouteParameter {
    pub name: String,
    pub location: ParamLocation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_hint: Option<String>,
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// A middleware or interceptor in the request processing pipeline
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MiddlewareNode {
    pub name: String,
    pub filepath: String,
    pub start_line: u32,
    pub is_global: bool,
    pub auth_kind: Option<RouteAuthKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Supported clientside API invocation libraries
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiClientKind {
    Fetch,
    Axios,
    TanStackQuery,
    NextServerAction,
    Custom(String),
}

impl ApiClientKind {
    pub fn as_str(&self) -> &str {
        match self {
            ApiClientKind::Fetch => "fetch",
            ApiClientKind::Axios => "axios",
            ApiClientKind::TanStackQuery => "react-query",
            ApiClientKind::NextServerAction => "server-action",
            ApiClientKind::Custom(s) => s.as_str(),
        }
    }
}

impl std::fmt::Display for ApiClientKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A clientside frontend API invocation site linking a UI component to a backend route
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApiClientCallSite {
    pub caller_node_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component_name: Option<String>,
    pub filepath: String,
    pub line: u32,
    pub method: HttpMethod,
    pub url_pattern: String,
    pub client_kind: ApiClientKind,
}

/// Architectural layer classification of a step in the execution trace
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceLayer {
    /// Frontend UI Component (e.g. React/Vue button, page)
    FrontendComponent,
    /// Clientside API fetch/axios invocation
    ApiClient,
    /// Ingress router definition
    Route,
    /// Middleware or authentication guard
    Middleware,
    /// Web controller or request handler view
    Controller,
    /// Domain or business application service
    Service,
    /// Data access object or repository
    Repository,
    /// ORM data model or entity
    Model,
    /// Relational database table
    DatabaseTable,
    /// Background job or task queue dispatcher
    Worker,
    /// Utility or helper function
    Utility,
    /// Unclassified call step
    Unknown,
}

impl TraceLayer {
    pub fn as_str(&self) -> &str {
        match self {
            TraceLayer::FrontendComponent => "FrontendComponent",
            TraceLayer::ApiClient => "ApiClient",
            TraceLayer::Route => "Route",
            TraceLayer::Middleware => "Middleware",
            TraceLayer::Controller => "Controller",
            TraceLayer::Service => "Service",
            TraceLayer::Repository => "Repository",
            TraceLayer::Model => "Model",
            TraceLayer::DatabaseTable => "DatabaseTable",
            TraceLayer::Worker => "Worker",
            TraceLayer::Utility => "Utility",
            TraceLayer::Unknown => "Unknown",
        }
    }
}

impl std::fmt::Display for TraceLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// An individual hop in the full-stack request execution trace
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraceStep {
    pub node_id: String,
    pub name: String,
    pub kind: NodeKind,
    pub filepath: String,
    pub line: u32,
    pub hop: usize,
    pub layer: TraceLayer,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docstring: Option<String>,
    pub is_mutation: bool,
}

/// Severity classification of an API contract modification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContractChangeSeverity {
    /// Incompatible breaking change that breaks existing API clients
    Breaking,
    /// Dangerous modification that may cause regressions under certain conditions
    Dangerous,
    /// Non-breaking backward-compatible enhancement (e.g. new optional parameter)
    Informational,
}

/// Specific type of contract drift or breaking change
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum ContractChangeKind {
    /// A previously published route endpoint was deleted
    RouteRemoved,
    /// A new route endpoint was added
    RouteAdded,
    /// An endpoint's HTTP method changed (e.g. GET -> POST)
    MethodChanged { old_method: HttpMethod, new_method: HttpMethod },
    /// A newly mandatory path, query, or header parameter was added without a default
    NewRequiredParam { name: String, location: ParamLocation },
    /// A parameter was removed from an endpoint
    RemovedParam { name: String },
    /// A parameter's data type changed (e.g. integer -> string)
    ParamTypeChanged { name: String, old_type: String, new_type: String },
    /// A required field was added to a request body DTO
    RequiredBodyFieldAdded { field: String },
    /// A field was deleted from a response payload or DTO model
    ResponseFieldRemoved { field: String },
    /// An endpoint previously Public was elevated to Protected
    AuthElevated { old_auth: RouteAuthKind, new_auth: RouteAuthKind },
}

/// A concrete breaking change detected between two versions of the API contract
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContractBreakingChange {
    pub route_id: String,
    pub path: String,
    pub method: HttpMethod,
    pub severity: ContractChangeSeverity,
    pub change: ContractChangeKind,
    pub description: String,
    pub filepath: String,
    pub line: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remediation_hint: Option<String>,
}

/// Top-level report emitted by `--verify-contracts`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContractDiffReport {
    pub base_ref: String,
    pub head_ref: String,
    pub total_routes_base: usize,
    pub total_routes_head: usize,
    pub breaking_changes: Vec<ContractBreakingChange>,
    pub non_breaking_changes: Vec<ContractBreakingChange>,
    pub passed: bool,
    pub execution_time_ms: f64,
}

/// Route health and security anomaly issues
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum RouteHealthIssue {
    /// Route is registered in router/urls but handler symbol cannot be found in codebase
    DanglingHandler { handler_name: String },
    /// Route is unreachable because a preceding pattern shadows it
    ShadowedBy { shadowing_pattern: String, line: u32 },
    /// Mutating route (POST/PUT/DELETE/PATCH) with no authentication guard
    UnauthenticatedMutation,
    /// Handler exceeds recommended complexity or touches too many models (God Controller)
    HighCoupling { model_count: usize },
    /// Route handler does not validate request body or input parameters
    MissingValidation,
}

/// Complete full-stack trace and metadata for an API endpoint
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FullStackRouteTrace {
    /// Canonical node ID of the route
    pub id: String,
    /// HTTP Method (GET, POST, etc.)
    pub method: HttpMethod,
    /// Original raw route pattern (e.g. "/users/:id" or "path('users/<int:id>/')")
    pub raw_pattern: String,
    /// Canonical normalized URL pattern (RFC 6570, e.g. "/users/{id}")
    pub normalized_path: String,
    /// Web framework detected (e.g. "express", "fastapi", "django", "nextjs", "rails", "axum", "gin")
    pub framework: String,
    /// Filepath where route is registered
    pub filepath: String,
    pub line: u32,
    /// Frontend UI components that call this endpoint
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub frontend_callers: Vec<ApiClientCallSite>,
    /// Extracted path and query parameters
    pub parameters: Vec<RouteParameter>,
    /// Ordered chain of middlewares executed before handler
    pub middlewares: Vec<MiddlewareNode>,
    /// Target controller / view handler node
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handler: Option<Node>,
    /// Sequential vertical execution trace through services and repositories
    pub call_chain: Vec<TraceStep>,
    /// All ORM data models touched by this route
    pub models_touched: Vec<Node>,
    /// All relational SQL database tables touched by this route
    pub sql_tables: Vec<String>,
    /// Authentication classification
    pub auth: RouteAuthKind,
    /// Whether this endpoint modifies data
    pub is_mutation: bool,
    /// Edge resolution confidence score (0.0 to 1.0)
    pub confidence: f64,
    /// Health, shadowing, and security issues
    pub issues: Vec<RouteHealthIssue>,
}

/// Filtering options for route queries
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RouteFilterOptions {
    /// Filter by HTTP method (e.g. GET, POST)
    pub method: Option<HttpMethod>,
    /// Substring, glob, or regex match on normalized path
    pub path_pattern: Option<String>,
    /// Filter routes called by a specific frontend UI component (e.g. "CheckoutButton")
    pub component_name: Option<String>,
    /// Show frontend callers for matched routes
    pub show_callers: bool,
    /// Verify API contracts across git branch diff
    pub verify_contracts: bool,
    /// Git branch diff range for contract verification (e.g. "main...HEAD")
    pub branch: Option<String>,
    /// CI Quality Gate: fail with exit code 1 if breaking changes exist
    pub fail_on_breaking: bool,
    /// Filter routes that touch a specific ORM model name
    pub model_name: Option<String>,
    /// Filter routes that touch a specific SQL table name
    pub sql_table: Option<String>,
    /// Filter by handler function name or controller class
    pub handler_name: Option<String>,
    /// Filter by web framework (e.g. "fastapi", "express", "nextjs")
    pub framework: Option<String>,
    /// Show only authenticated / protected endpoints
    pub auth_only: bool,
    /// Show only public / unauthenticated endpoints
    pub public_only: bool,
    /// Show only routes with health/security issues
    pub issues_only: bool,
    /// Show only dead/dangling routes
    pub dead_only: bool,
    /// Show only shadowed routes
    pub shadowed_only: bool,
    /// Maximum downstream call chain depth to trace (default: 5)
    pub max_depth: usize,
    /// Minimum edge confidence threshold (default: 0.5)
    pub min_confidence: f64,
}

/// Statistical summary of the discovered route inventory
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RouteSummary {
    pub total_routes: usize,
    pub routes_by_method: HashMap<String, usize>,
    pub routes_by_framework: HashMap<String, usize>,
    pub public_routes: usize,
    pub protected_routes: usize,
    pub mutating_routes: usize,
    pub total_models_exposed: usize,
    pub total_frontend_callers: usize,
    pub dead_routes_count: usize,
    pub shadowed_routes_count: usize,
    pub security_warnings_count: usize,
    pub breaking_contract_changes: usize,
}

/// The top-level report emitted by `cartograph routes`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RouteReport {
    /// List of mapped full-stack route traces matching filters
    pub routes: Vec<FullStackRouteTrace>,
    /// Total count of routes discovered before filtering
    pub total_discovered: usize,
    /// Statistical breakdown
    pub summary: RouteSummary,
    /// Contract diff report if --verify-contracts was run
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_diff: Option<ContractDiffReport>,
    /// Execution time in milliseconds
    pub execution_time_ms: f64,
}

/// Raw record from downstream route graph traversal query
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawRouteTraceRecord {
    pub target_node: Node,
    pub route_id: String,
    pub hop_distance: usize,
    pub hop_kind: String,
    pub path_str: String,
    pub confidence: f64,
}

/// Raw record from inverted model-to-route graph query
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawRouteModelRecord {
    pub route_node: Node,
    pub model_id: String,
    pub hop_distance: usize,
    pub path_str: String,
    pub confidence: f64,
}

/// Raw record from frontend caller graph query
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawFrontendCallerRecord {
    pub route_id: String,
    pub caller_node: Node,
    pub component_name: Option<String>,
    pub client_kind: String,
    pub line: u32,
    pub method: String,
    pub url_pattern: String,
}

use chrono::{DateTime, Utc};
use crate::config::{ArchSeverity, CycleGranularity};

/// A single node hop within a circular dependency ring
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CycleHop {
    /// Node canonical URI
    pub node_id: String,
    /// Symbol or file name
    pub name: String,
    /// Filepath of this component
    pub filepath: String,
    /// Line number where this component is declared or where the outgoing call is made
    pub line: Option<usize>,
    /// Column number
    pub column: Option<usize>,
    /// Node kind (File, Module, Function, Struct, etc.)
    pub kind: NodeKind,
    /// Outgoing edge kind leading to the next node in the cycle
    pub outgoing_edge_kind: EdgeKind,
    /// Whether this hop represents a runtime execution edge (calls, value import) or type-only
    pub is_runtime: bool,
}

/// Recommended cut edge to break a circular dependency
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FeedbackArcCut {
    /// Source node in the cycle
    pub source_id: String,
    /// Target node in the cycle
    pub target_id: String,
    /// Edge kind
    pub edge_kind: EdgeKind,
    /// Source filepath
    pub source_file: String,
    /// Target filepath
    pub target_file: String,
    /// Number of distinct cycles broken if this edge is severed
    pub cycles_broken_count: usize,
    /// Concrete refactoring suggestion (e.g. "Extract interface or invert dependency")
    pub remediation_advice: String,
}

/// A detected circular dependency loop
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencyCycle {
    /// Unique SHA-256 fingerprint of this cycle ring (normalized by lexicographically lowest node)
    pub fingerprint: String,
    /// Granularity of this cycle (File, Module, Package, Symbol)
    pub granularity: CycleGranularity,
    /// Number of hops in the cycle
    pub length: usize,
    /// Whether this cycle contains at least one active runtime execution call (deadlock risk)
    pub is_runtime_cycle: bool,
    /// Ordered sequence of hops forming the ring (A -> B -> C -> A)
    pub hops: Vec<CycleHop>,
    /// Recommended cut edge to break this cycle
    pub recommended_cut: Option<FeedbackArcCut>,
    /// Severity (Runtime cycles = Error, Type-only cycles = Warning/Info)
    pub severity: ArchSeverity,
}

/// Specific architectural boundary violation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoundaryViolation {
    /// Unique SHA-256 fingerprint for baseline tracking
    pub fingerprint: String,
    /// Name of the broken rule or layer constraint
    pub rule_name: String,
    /// Category of violation: "layer_violation", "boundary_rule", "package_encapsulation", "cycle"
    pub category: String,
    /// Severity level
    pub severity: ArchSeverity,
    /// Source node URI
    pub source_id: String,
    /// Source symbol name
    pub source_name: String,
    /// Source relative filepath
    pub source_file: String,
    /// Source layer name (if layer violation)
    pub source_layer: Option<String>,
    /// Target node URI
    pub target_id: String,
    /// Target symbol name
    pub target_name: String,
    /// Target relative filepath
    pub target_file: String,
    /// Target layer name (if layer violation)
    pub target_layer: Option<String>,
    /// Kind of edge causing the breach (Imports, Calls, etc.)
    pub edge_kind: EdgeKind,
    /// Exact line number in source file
    pub line: Option<usize>,
    /// Exact column number in source file
    pub column: Option<usize>,
    /// Source code snippet of the offending statement (e.g. "use crate::controllers::*")
    pub snippet: Option<String>,
    /// Diagnostic explanation message
    pub message: String,
    /// Actionable refactoring guidance
    pub remediation_hint: Option<String>,
    /// Whether this violation is suppressed because it matches the baseline
    pub is_baseline_suppressed: bool,
    /// Whether this violation was introduced by the current git branch diff
    pub is_new_in_diff: bool,
}

/// Summary scorecard of architectural linting
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct LintArchSummary {
    /// Total number of files evaluated
    pub files_analyzed: usize,
    /// Total number of dependency edges inspected
    pub edges_inspected: usize,
    /// Total number of layers configured
    pub layers_count: usize,
    /// Total number of custom boundary rules evaluated
    pub rules_count: usize,
    /// Total number of circular dependency loops found
    pub cycles_count: usize,
    /// Number of runtime cycles (deadlock risk)
    pub runtime_cycles_count: usize,
    /// Number of type-only cycles (benign)
    pub type_cycles_count: usize,
    /// Total active violations (excluding baseline suppressed)
    pub active_violations_count: usize,
    /// Number of active errors
    pub active_errors_count: usize,
    /// Number of active warnings
    pub active_warnings_count: usize,
    /// Number of pre-existing violations suppressed by baseline
    pub baseline_suppressed_count: usize,
    /// Number of newly introduced violations in git diff
    pub new_in_diff_count: usize,
    /// Overall CI pass/fail status
    pub passed: bool,
    /// Total execution duration in microseconds
    pub duration_micros: u64,
}

/// Complete architectural linting report
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LintArchReport {
    /// High-level scorecard
    pub summary: LintArchSummary,
    /// Detected circular dependencies
    pub cycles: Vec<DependencyCycle>,
    /// Detected boundary & layer violations
    pub violations: Vec<BoundaryViolation>,
    /// List of suppressed baseline violations (for inspection)
    pub suppressed_violations: Vec<BoundaryViolation>,
    /// Timestamp of execution
    pub timestamp: DateTime<Utc>,
}

/// Serialized baseline file storing accepted technical debt
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchBaseline {
    /// Baseline schema version (e.g. "1.0")
    pub version: String,
    /// ISO 8601 timestamp when baseline was recorded
    pub recorded_at: String,
    /// Git commit SHA at time of recording
    pub git_commit: Option<String>,
    /// Recorded violation entries
    pub violations: Vec<BaselineViolationEntry>,
    /// Recorded cycle entries
    pub cycles: Vec<BaselineCycleEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BaselineViolationEntry {
    /// SHA-256 fingerprint
    pub fingerprint: String,
    /// Rule name
    pub rule_name: String,
    /// Source filepath
    pub source_file: String,
    /// Target filepath
    pub target_file: String,
    /// Edge kind
    pub edge_kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BaselineCycleEntry {
    /// SHA-256 fingerprint of normalized cycle
    pub fingerprint: String,
    /// Cycle length
    pub length: usize,
    /// Ordered filepaths in the cycle
    pub path: Vec<String>,
}

/// Raw record returned by database for inter-file dependency analysis
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawInterFileEdgeRecord {
    pub source_id: String,
    pub source_name: String,
    pub source_kind: String,
    pub source_file: String,
    pub source_line: Option<usize>,
    pub source_col: Option<usize>,
    pub target_id: String,
    pub target_name: String,
    pub target_kind: String,
    pub target_file: String,
    pub edge_kind: String,
    pub confidence: Option<f64>,
    pub metadata: Option<serde_json::Value>,
}

// ============================================================================
// Phase 2.5: Complexity & Hotspot Models
// ============================================================================

/// Granularity level for complexity analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ComplexityGranularity {
    /// Function, method, class, struct, interface symbols
    #[default]
    Symbol,
    /// File-level aggregated metrics
    File,
    /// Directory/module-level aggregated metrics
    Module,
    /// Package/crate-level aggregated metrics
    Package,
}

impl ComplexityGranularity {
    pub fn as_str(&self) -> &'static str {
        match self {
            ComplexityGranularity::Symbol => "symbol",
            ComplexityGranularity::File => "file",
            ComplexityGranularity::Module => "module",
            ComplexityGranularity::Package => "package",
        }
    }
}

impl std::fmt::Display for ComplexityGranularity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for ComplexityGranularity {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "symbol" | "fn" | "class" => Ok(ComplexityGranularity::Symbol),
            "file" => Ok(ComplexityGranularity::File),
            "module" | "dir" => Ok(ComplexityGranularity::Module),
            "package" | "crate" => Ok(ComplexityGranularity::Package),
            other => Err(format!(
                "Unknown complexity granularity: {other}. Expected symbol, file, module, or package"
            )),
        }
    }
}

/// Primary AST-level complexity metrics for a function, method, or file
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ComplexityMetric {
    /// McCabe Cyclomatic Complexity v(G) = decision points + 1
    pub cyclomatic: u32,
    /// SonarSource Cognitive Complexity (nesting-penalized)
    pub cognitive: u32,
    /// Physical total lines (end_line - start_line + 1)
    pub loc: u32,
    /// Source Lines of Code (excluding blank lines and comment-only lines)
    pub sloc: u32,
    /// Comment Lines of Code
    pub cloc: u32,
    /// Comment-to-Code ratio (cloc / (sloc + cloc))
    pub comment_ratio: f64,
    /// Maximum control flow nesting depth
    pub max_nesting_depth: u32,
    /// Number of parameters in signature
    pub parameter_count: u32,
}

/// Topological graph coupling metrics derived from inter-symbol dependency edges
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CouplingMetric {
    /// Afferent Coupling (In-Degree / Ca): number of incoming edges from callers
    pub in_degree: usize,
    /// Efferent Coupling (Out-Degree / Ce): number of outgoing edges to callees
    pub out_degree: usize,
    /// Total direct coupling degree (Ca + Ce)
    pub total_degree: usize,
    /// Instability Index I = Ce / (Ca + Ce), ranges from 0.0 (stable) to 1.0 (unstable)
    pub instability: f64,
    /// Number of distinct foreign files/modules coupled
    pub cross_file_coupling: usize,
}

/// Complexity-to-Test Deficit metric correlating complexity with the Test Graph
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct TestDeficitMetric {
    /// Total number of direct and transitive tests exercising this symbol (from Phase 2.2)
    pub associated_test_count: usize,
    /// True if component has high complexity (CC >= 15) and 0 tests
    pub is_danger_zone: bool,
    /// Ratio of cyclomatic complexity to associated tests
    pub test_deficit_ratio: f64,
}

/// AI Context Friction & Reasonability Score
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct AiContextFrictionMetric {
    /// Estimated token count of the symbol's full definition snippet
    pub estimated_tokens: usize,
    /// Context Complexity Ratio = Cognitive Complexity / Tokens
    pub context_complexity_ratio: f64,
    /// True if Cognitive > 30 and Tokens > 2500 (exceeds single-turn AI reasoning capacity)
    pub is_high_hallucination_risk: bool,
    /// AI agent risk advice
    pub risk_guidance: Option<String>,
}

/// Direction of complexity drift over the time window
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TrendDirection {
    /// Complexity increasing rapidly (+25% or more) 🌋
    Volcanic,
    /// Complexity decreasing through refactoring (-15% or more) 🧊
    CoolingDown,
    /// Complexity stable (drift within -15% to +25%)
    #[default]
    Stable,
}

impl TrendDirection {
    pub fn as_str(&self) -> &'static str {
        match self {
            TrendDirection::Volcanic => "Volcanic (Heating Up)",
            TrendDirection::CoolingDown => "Cooling Down (Improving)",
            TrendDirection::Stable => "Stable",
        }
    }

    pub fn emoji(&self) -> &'static str {
        match self {
            TrendDirection::Volcanic => "🌋",
            TrendDirection::CoolingDown => "🧊",
            TrendDirection::Stable => "⚪",
        }
    }
}

/// Historical complexity drift trend
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ComplexityTrendMetric {
    /// Direction of complexity drift over the time window
    pub trend: TrendDirection,
    /// Percentage change in cyclomatic complexity over time window
    pub cyclomatic_pct_change: f64,
    /// Number of commits analyzing trend across
    pub snapshot_commits_count: usize,
}

/// SEI Maintainability Index & SQALE Technical Debt Hours
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct MaintainabilityMetric {
    /// Standard SEI / Visual Studio Maintainability Index (0.0 to 100.0)
    pub maintainability_index: f64,
    /// Estimated SQALE technical debt remediation hours
    pub estimated_debt_hours: f64,
}

/// Object-Oriented cohesion and class-level metrics for structs, classes, and interfaces
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ClassCohesionMetric {
    /// Weighted Methods per Class: sum of cyclomatic complexities of all member methods
    pub wmc: u32,
    /// Total number of member methods
    pub method_count: usize,
    /// Total number of member attributes/fields
    pub field_count: usize,
    /// Lack of Cohesion in Methods 4: number of connected components in method-field sharing graph
    pub lcom4: usize,
    /// Tight Class Cohesion: fraction of directly connected method pairs (0.0 to 1.0)
    pub tcc: f64,
    /// Access to Foreign Data: count of distinct foreign classes whose fields are accessed
    pub atfd: usize,
}

/// Git change velocity and commit recency metrics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ChurnMetric {
    /// Number of commits modifying this file in the time window
    pub commit_count: usize,
    /// Number of distinct git author contributors
    pub author_count: usize,
    /// Unix timestamp of the most recent commit modifying this file
    pub last_modified_timestamp: i64,
    /// Normalized churn velocity score (0.0 to 1.0) accounting for frequency and recency
    pub churn_velocity_score: f64,
}

/// Adam Tornhill 4-Quadrant Hotspot classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum HotspotQuadrant {
    /// Q1: High Complexity & High Churn (Top Refactoring Priority) 🚨
    CriticalHotspot,
    /// Q2: Low Complexity & High Churn (High Volatility, Needs Good Tests) ⚠️
    ChurnHotspot,
    /// Q3: Low Complexity & Low Churn (Healthy Code) ✅
    #[default]
    Healthy,
    /// Q4: High Complexity & Low Churn (Stable Technical Debt) ⏳
    StableTechnicalDebt,
}

impl HotspotQuadrant {
    pub fn as_str(&self) -> &'static str {
        match self {
            HotspotQuadrant::CriticalHotspot => "Critical Hotspot (Q1)",
            HotspotQuadrant::ChurnHotspot => "Churn Hotspot (Q2)",
            HotspotQuadrant::Healthy => "Healthy (Q3)",
            HotspotQuadrant::StableTechnicalDebt => "Stable Technical Debt (Q4)",
        }
    }

    pub fn emoji(&self) -> &'static str {
        match self {
            HotspotQuadrant::CriticalHotspot => "🚨",
            HotspotQuadrant::ChurnHotspot => "⚠️",
            HotspotQuadrant::Healthy => "✅",
            HotspotQuadrant::StableTechnicalDebt => "⏳",
        }
    }
}

/// God-Class severity classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum GodClassSeverity {
    #[default]
    Clean,
    Minor,
    Major,
    Critical,
}

/// Candidate class decomposition cluster extracted via LCOM4 connected components
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecompositionCluster {
    /// Synthesized name suggestion for the new extracted class/service (e.g. "UserAuthenticationService")
    pub candidate_name: String,
    /// Names of member methods assigned to this cluster
    pub methods: Vec<String>,
    /// Names of fields/attributes mutated or shared by these methods
    pub fields: Vec<String>,
    /// Percentage of total class methods represented in this component
    pub method_percentage: f64,
    /// Concrete architectural rationale for extraction
    pub rationale: String,
}

/// Detailed evaluation of a class/struct for God-Class anti-pattern
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct GodClassEvaluation {
    /// True if component meets God-Class heuristic criteria
    pub is_god_class: bool,
    /// Severity classification
    pub severity: Option<GodClassSeverity>,
    /// Composite God-Class Index (GCI, 0.0 to 100.0)
    pub god_class_index: f64,
    /// List of specific heuristic triggers (e.g. "WMC (68) exceeds threshold (50)")
    pub triggers: Vec<String>,
    /// Concrete decomposition clusters if LCOM4 > 1
    pub candidate_extractions: Vec<DecompositionCluster>,
}

/// Complete complexity and hotspot evaluation record for a single symbol, file, or module
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComplexityRecord {
    /// Canonical URI identifier (e.g. "cartograph://src/services/billing.rs::BillingManager#struct")
    pub id: String,
    /// Short symbol name (e.g. "BillingManager" or "process_payment")
    pub name: String,
    /// AST kind (Function, Method, Class, Struct, Interface, File, Module)
    pub kind: String,
    /// Relative file path
    pub filepath: String,
    /// Scope qualifier
    pub scope: String,
    /// Source coordinates
    pub start_line: u32,
    pub end_line: u32,
    /// AST Complexity metrics
    pub complexity: ComplexityMetric,
    /// Topological graph coupling metrics
    pub coupling: CouplingMetric,
    /// Complexity-to-Test Deficit metrics
    pub test_deficit: TestDeficitMetric,
    /// AI Context Friction metrics
    pub ai_friction: AiContextFrictionMetric,
    /// Complexity Trend
    pub trend: ComplexityTrendMetric,
    /// Maintainability Index & Debt Hours
    pub maintainability: MaintainabilityMetric,
    /// Class cohesion metrics (if struct/class)
    pub class_cohesion: Option<ClassCohesionMetric>,
    /// Git churn velocity
    pub churn: ChurnMetric,
    /// Composite Hotspot Score (0.0 to 100.0)
    pub hotspot_score: f64,
    /// Hotspot matrix quadrant
    pub quadrant: HotspotQuadrant,
    /// God-Class evaluation (if applicable)
    pub god_class: Option<GodClassEvaluation>,
    /// SHA-256 fingerprint for baseline comparison
    pub fingerprint: String,
}

/// Complexity change introduced by a Git PR branch diff
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComplexityDiffDelta {
    pub symbol_name: String,
    pub filepath: String,
    pub line_number: u32,
    pub previous_cyclomatic: Option<u32>,
    pub new_cyclomatic: u32,
    pub cyclomatic_delta: i32,
    pub previous_cognitive: Option<u32>,
    pub new_cognitive: u32,
    pub cognitive_delta: i32,
    pub is_new_god_class: bool,
}

/// Overall repository-wide complexity and hotspot report
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComplexityReport {
    /// Timestamp of analysis execution
    pub timestamp: String,
    /// Granularity analyzed
    pub granularity: ComplexityGranularity,
    /// Total components analyzed
    pub total_analyzed: usize,
    /// Total God-Classes flagged
    pub god_classes_detected: usize,
    /// Total Critical Hotspots (Q1) flagged
    pub critical_hotspots_count: usize,
    /// Total Danger Zone (0 Tests) flagged
    pub danger_zone_count: usize,
    /// Total estimated technical debt hours across repository
    pub total_debt_hours: f64,
    /// Repository average metrics
    pub avg_cyclomatic: f64,
    pub avg_cognitive: f64,
    pub avg_loc: f64,
    pub avg_maintainability_index: f64,
    /// Top N ranked hotspot records
    pub top_hotspots: Vec<ComplexityRecord>,
    /// All God-Classes (if filtered or requested)
    pub god_classes: Vec<ComplexityRecord>,
    /// Danger Zone records
    pub danger_zone: Vec<ComplexityRecord>,
    /// Git diff delta records (if --branch or --staged)
    pub diff_deltas: Option<Vec<ComplexityDiffDelta>>,
    /// Suppressed baseline technical debt items count
    pub suppressed_baseline_count: usize,
}

/// Baseline file for complexity technical debt
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComplexityBaseline {
    pub version: String,
    pub recorded_at: String,
    pub git_commit: Option<String>,
    pub hotspots: Vec<BaselineHotspotEntry>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BaselineHotspotEntry {
    pub fingerprint: String,
    pub symbol_name: String,
    pub filepath: String,
    pub cyclomatic: u32,
    pub cognitive: u32,
    pub is_god_class: bool,
}

/// Raw record returned by database for batch coupling analysis
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawSymbolCouplingRecord {
    pub symbol_id: String,
    pub in_degree: usize,
    pub out_degree: usize,
}

/// Raw record returned by database for class member analysis
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawClassMemberRecord {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub start_line: Option<u32>,
    pub end_line: Option<u32>,
    pub metadata: Option<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Phase 2.6: Dead Code & Abandoned Component Detection Models
// ---------------------------------------------------------------------------

/// Classification category of detected dead code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeadCodeCategory {
    DeadFunction,
    DeadMethod,
    DeadClass,
    AbandonedComponent,
    DanglingRoute,
    DeadRouteParameter,
    UnusedRpcOperation,
    AbandonedModel,
    DeadIsland,
    ZombieFile,
    TestOnlyZombie,
    UnreadField,
    UnusedEnumVariant,
    DeadBarrelExport,
    GhostDependency,
    DeprecatedSymbol,
}

impl DeadCodeCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            DeadCodeCategory::DeadFunction => "dead_function",
            DeadCodeCategory::DeadMethod => "dead_method",
            DeadCodeCategory::DeadClass => "dead_class",
            DeadCodeCategory::AbandonedComponent => "abandoned_component",
            DeadCodeCategory::DanglingRoute => "dangling_route",
            DeadCodeCategory::DeadRouteParameter => "dead_route_parameter",
            DeadCodeCategory::UnusedRpcOperation => "unused_rpc_operation",
            DeadCodeCategory::AbandonedModel => "abandoned_model",
            DeadCodeCategory::DeadIsland => "dead_island",
            DeadCodeCategory::ZombieFile => "zombie_file",
            DeadCodeCategory::TestOnlyZombie => "test_only_zombie",
            DeadCodeCategory::UnreadField => "unread_field",
            DeadCodeCategory::UnusedEnumVariant => "unused_enum_variant",
            DeadCodeCategory::DeadBarrelExport => "dead_barrel_export",
            DeadCodeCategory::GhostDependency => "ghost_dependency",
            DeadCodeCategory::DeprecatedSymbol => "deprecated_symbol",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            DeadCodeCategory::DeadFunction => "Dead Function",
            DeadCodeCategory::DeadMethod => "Dead Method",
            DeadCodeCategory::DeadClass => "Dead Class",
            DeadCodeCategory::AbandonedComponent => "Abandoned UI Component",
            DeadCodeCategory::DanglingRoute => "Dangling API Route",
            DeadCodeCategory::DeadRouteParameter => "Dead Route Parameter",
            DeadCodeCategory::UnusedRpcOperation => "Unused GraphQL/gRPC Operation",
            DeadCodeCategory::AbandonedModel => "Abandoned Database Model",
            DeadCodeCategory::DeadIsland => "Dead Island (Mutual Cycle)",
            DeadCodeCategory::ZombieFile => "Zombie File",
            DeadCodeCategory::TestOnlyZombie => "Test-Only Zombie",
            DeadCodeCategory::UnreadField => "Unread Field",
            DeadCodeCategory::UnusedEnumVariant => "Unused Enum Variant",
            DeadCodeCategory::DeadBarrelExport => "Dead Barrel Re-Export",
            DeadCodeCategory::GhostDependency => "Ghost Manifest Dependency",
            DeadCodeCategory::DeprecatedSymbol => "Deprecated Symbol",
        }
    }
}

/// Confidence tier indicating certainty that code is truly dead
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfidenceTier {
    Suppressed,
    Low,
    Medium,
    High,
}

impl ConfidenceTier {
    pub fn badge(&self) -> &'static str {
        match self {
            ConfidenceTier::High => "🟢 High",
            ConfidenceTier::Medium => "🟡 Medium",
            ConfidenceTier::Low => "🟠 Low",
            ConfidenceTier::Suppressed => "⚪ Suppressed",
        }
    }
}

/// Specific factor influencing the confidence score calculation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfidenceFactor {
    pub description: String,
    pub delta: f64,
}

/// Individual detected dead code symbol record
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeadSymbolItem {
    pub id: String,
    pub name: String,
    pub kind: NodeKind,
    pub filepath: String,
    pub scope: String,
    pub start_line: u32,
    pub end_line: u32,
    pub loc: usize,
    pub sloc: usize,
    pub estimated_tokens: usize,
    pub category: DeadCodeCategory,
    pub confidence: f64,
    pub tier: ConfidenceTier,
    pub confidence_factors: Vec<ConfidenceFactor>,
    pub island_id: Option<String>,
    pub parent_id: Option<String>,
    pub associated_tests: Vec<String>,
    pub fingerprint: String,
}

/// Dead Island: A cluster of mutually dependent dead nodes forming an unreachable subgraph
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeadIsland {
    pub id: String,
    pub island_roots: Vec<String>,
    pub member_symbol_ids: Vec<String>,
    pub filepaths: Vec<String>,
    pub total_loc: usize,
    pub total_sloc: usize,
    pub total_tokens: usize,
    pub deletion_order: Vec<String>,
    pub cascading_package_savings: Vec<String>,
}

/// Ghost Dependency: Installed manifest package with zero imports across the entire graph
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GhostDependencyReport {
    pub package_name: String,
    pub manifest_file: String,
    pub version: String,
    pub estimated_disk_saved_mb: f64,
    pub estimated_bundle_saved_kb: f64,
    pub removal_command: String,
}

/// Status of a deprecated symbol
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeprecatedSymbolStatus {
    pub id: String,
    pub name: String,
    pub filepath: String,
    pub start_line: u32,
    pub deprecation_note: Option<String>,
    pub remaining_callers_count: usize,
    pub caller_locations: Vec<String>,
    pub is_ready_for_cremation: bool,
    pub migration_progress_pct: f64,
}

/// Zombie File: A source file where 100% of defined symbols are dead
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ZombieFileReport {
    pub filepath: String,
    pub total_loc: usize,
    pub total_sloc: usize,
    pub symbol_count: usize,
    pub total_tokens: usize,
    pub prune_command: String,
}

/// Financial, operational, and code health impact metrics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct DeadCodeImpactMetrics {
    pub total_dead_symbols: usize,
    pub total_dead_loc: usize,
    pub total_dead_sloc: usize,
    pub total_wasted_tokens: usize,
    pub estimated_monthly_token_cost: f64,
    pub estimated_bundle_bloat_kb: f64,
    pub zombie_file_count: usize,
    pub dead_island_count: usize,
    pub ghost_dependency_count: usize,
    pub ready_for_cremation_count: usize,
    pub test_only_zombie_count: usize,
}

/// Git Diff Delta for PR evaluation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct DeadCodePRDelta {
    pub newly_introduced: Vec<DeadSymbolItem>,
    pub newly_orphaned: Vec<DeadSymbolItem>,
    pub resolved_symbols: Vec<String>,
    pub net_dead_loc_delta: i64,
}

/// Technical debt baseline storage structure (`.cartograph-deadcode-baseline.json`)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeadCodeBaseline {
    pub version: String,
    pub recorded_at: String,
    pub git_commit: Option<String>,
    pub dead_items: Vec<BaselineDeadItemEntry>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BaselineDeadItemEntry {
    pub fingerprint: String,
    pub symbol_name: String,
    pub filepath: String,
    pub category: DeadCodeCategory,
    pub loc: usize,
    pub confidence: f64,
}

/// Complete top-level report payload for `cartograph deadcode`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeadCodeReport {
    pub repo_path: String,
    pub db_profile: String,
    pub total_scanned_symbols: usize,
    pub dead_symbols: Vec<DeadSymbolItem>,
    pub dead_islands: Vec<DeadIsland>,
    pub zombie_files: Vec<ZombieFileReport>,
    pub ghost_dependencies: Vec<GhostDependencyReport>,
    pub deprecated_statuses: Vec<DeprecatedSymbolStatus>,
    pub impact: DeadCodeImpactMetrics,
    pub pr_delta: Option<DeadCodePRDelta>,
    pub baseline_suppressed_count: usize,
    pub duration_ms: f64,
}

/// Raw record returned by database for unreachable subgraph query
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawDeadSymbolRecord {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub filepath: String,
    pub scope: String,
    pub start_line: Option<u32>,
    pub end_line: Option<u32>,
    pub signature: Option<String>,
    pub docstring: Option<String>,
    pub checksum: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub in_degree: usize,
    pub out_degree: usize,
}


