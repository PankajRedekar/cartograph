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


