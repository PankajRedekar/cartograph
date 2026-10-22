use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TargetIde {
    Claude,
    Cursor,
    Windsurf,
    Antigravity,
    Aider,
    Roo,
    Cline,
    OpenHands,
    All,
}

impl std::str::FromStr for TargetIde {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "claude" | "claudecode" => Ok(TargetIde::Claude),
            "cursor" => Ok(TargetIde::Cursor),
            "windsurf" => Ok(TargetIde::Windsurf),
            "antigravity" | "gemini" => Ok(TargetIde::Antigravity),
            "aider" => Ok(TargetIde::Aider),
            "roo" | "roomodes" => Ok(TargetIde::Roo),
            "cline" | "clinerules" => Ok(TargetIde::Cline),
            "openhands" => Ok(TargetIde::OpenHands),
            "all" => Ok(TargetIde::All),
            _ => Err(format!(
                "Unknown target IDE: '{}'. Supported: claude, cursor, windsurf, antigravity, aider, roo, cline, openhands, all",
                s
            )),
        }
    }
}

impl std::fmt::Display for TargetIde {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TargetIde::Claude => write!(f, "claude"),
            TargetIde::Cursor => write!(f, "cursor"),
            TargetIde::Windsurf => write!(f, "windsurf"),
            TargetIde::Antigravity => write!(f, "antigravity"),
            TargetIde::Aider => write!(f, "aider"),
            TargetIde::Roo => write!(f, "roo"),
            TargetIde::Cline => write!(f, "cline"),
            TargetIde::OpenHands => write!(f, "openhands"),
            TargetIde::All => write!(f, "all"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SkillPreset {
    Minimal,
    Standard,
    Strict,
}

impl std::str::FromStr for SkillPreset {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "minimal" => Ok(SkillPreset::Minimal),
            "standard" => Ok(SkillPreset::Standard),
            "strict" => Ok(SkillPreset::Strict),
            _ => Err(format!(
                "Unknown preset: '{}'. Supported: minimal, standard, strict",
                s
            )),
        }
    }
}

impl std::fmt::Display for SkillPreset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkillPreset::Minimal => write!(f, "minimal"),
            SkillPreset::Standard => write!(f, "standard"),
            SkillPreset::Strict => write!(f, "strict"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDefinition {
    pub id: String,
    pub name: String,
    pub title: String,
    pub description: String,
    pub schema_version: String,
    pub cartograph_version: String,
    pub content_hash: String,
    pub triggers: Vec<String>,
    pub cli_command: String,
    pub template_content: &'static str,
}

pub struct SkillRegistry;

impl SkillRegistry {
    pub const TEMPLATE_VERIFY: &'static str = include_str!("templates/verify.md");
    pub const TEMPLATE_LOCATE: &'static str = include_str!("templates/locate.md");
    pub const TEMPLATE_IMPACT: &'static str = include_str!("templates/impact.md");
    pub const TEMPLATE_TEST_SELECTOR: &'static str = include_str!("templates/test_selector.md");
    pub const TEMPLATE_ROUTES: &'static str = include_str!("templates/routes.md");
    pub const TEMPLATE_DIAGNOSE: &'static str = include_str!("templates/diagnose.md");
    pub const TEMPLATE_COMPLEXITY: &'static str = include_str!("templates/complexity.md");
    pub const TEMPLATE_LINT_ARCH: &'static str = include_str!("templates/lint_arch.md");
    pub const TEMPLATE_DEADCODE: &'static str = include_str!("templates/deadcode.md");
    pub const TEMPLATE_EXTRACT_LOGIC: &'static str = include_str!("templates/extract_logic.md");
    pub const TEMPLATE_DIAGRAM: &'static str = include_str!("templates/diagram.md");

    pub fn all_skills() -> Vec<SkillDefinition> {
        vec![
            SkillDefinition {
                id: "cartograph-verify".to_string(),
                name: "verify".to_string(),
                title: "Pre-Commit Closed-Loop Verification Gate".to_string(),
                description: "Closed-loop pre-commit verification gate combining syntax, architectural boundaries, SemVer contract impact, and predictive tests into a sub-200ms check.".to_string(),
                schema_version: "2.0".to_string(),
                cartograph_version: "0.2.0".to_string(),
                content_hash: "sha256:verify_v2_gate_hash_020".to_string(),
                triggers: vec![
                    "verify my changes".to_string(),
                    "check if ready to commit".to_string(),
                    "run pre-commit checks".to_string(),
                    "did my edit break anything".to_string(),
                    "run verification gate".to_string(),
                ],
                cli_command: "cartograph verify --staged --format xml".to_string(),
                template_content: Self::TEMPLATE_VERIFY,
            },
            SkillDefinition {
                id: "cartograph-locate".to_string(),
                name: "locate".to_string(),
                title: "AST Subgraph Retrieval & Localization".to_string(),
                description: "1-turn AST subgraph retrieval replacing blind grep loops with instant target definition, docstring, inlined type contracts, and callers/callees.".to_string(),
                schema_version: "2.0".to_string(),
                cartograph_version: "0.2.0".to_string(),
                content_hash: "sha256:locate_v2_gate_hash_020".to_string(),
                triggers: vec![
                    "where is function X defined".to_string(),
                    "find caller of Y".to_string(),
                    "show implementation of Z".to_string(),
                    "read class structure".to_string(),
                    "how is this type used".to_string(),
                ],
                cli_command: "cartograph locate \"<symbol>\" --format xml --resolve-types".to_string(),
                template_content: Self::TEMPLATE_LOCATE,
            },
            SkillDefinition {
                id: "cartograph-impact".to_string(),
                name: "impact".to_string(),
                title: "Blast Radius & Downstream Risk Analysis".to_string(),
                description: "Pre-edit blast radius & breaking change detector calculating downstream risk score, contract breaks, and missing test coverage.".to_string(),
                schema_version: "2.0".to_string(),
                cartograph_version: "0.2.0".to_string(),
                content_hash: "sha256:impact_v2_gate_hash_020".to_string(),
                triggers: vec![
                    "refactor function X".to_string(),
                    "rename parameter in Y".to_string(),
                    "delete method Z".to_string(),
                    "what breaks if I change this".to_string(),
                    "check blast radius of my changes".to_string(),
                ],
                cli_command: "cartograph impact \"<symbol>\" --format xml --direction downstream".to_string(),
                template_content: Self::TEMPLATE_IMPACT,
            },
            SkillDefinition {
                id: "cartograph-test-selector".to_string(),
                name: "test-selector".to_string(),
                title: "Predictive Test Selection".to_string(),
                description: "Predictive test selection mapping changed symbols and files to minimal sufficient test suites, saving up to 98% in test runtime.".to_string(),
                schema_version: "2.0".to_string(),
                cartograph_version: "0.2.0".to_string(),
                content_hash: "sha256:test_selector_v2_gate_hash_020".to_string(),
                triggers: vec![
                    "run tests for my changes".to_string(),
                    "which tests cover this function".to_string(),
                    "verify my edits".to_string(),
                    "run affected tests".to_string(),
                ],
                cli_command: "cartograph test-selector --staged --run".to_string(),
                template_content: Self::TEMPLATE_TEST_SELECTOR,
            },
            SkillDefinition {
                id: "cartograph-routes".to_string(),
                name: "routes".to_string(),
                title: "Full-Stack HTTP Route & ORM Tracer".to_string(),
                description: "Full-stack HTTP route tracer mapping endpoints from URL patterns through middlewares and handlers down to ORM models and database tables.".to_string(),
                schema_version: "2.0".to_string(),
                cartograph_version: "0.2.0".to_string(),
                content_hash: "sha256:routes_v2_gate_hash_020".to_string(),
                triggers: vec![
                    "what handler serves /api/v1/users".to_string(),
                    "find route for checkout".to_string(),
                    "trace route to database table".to_string(),
                    "list all public API routes".to_string(),
                ],
                cli_command: "cartograph routes --pattern \"<route>\" --format xml".to_string(),
                template_content: Self::TEMPLATE_ROUTES,
            },
            SkillDefinition {
                id: "cartograph-diagnose".to_string(),
                name: "diagnose".to_string(),
                title: "Bug Diagnosis & AST Backward Slicing".to_string(),
                description: "Crash log and stack trace AST backward slicer that anchors frames to code nodes, isolates root causes, and scaffolds test reproducers.".to_string(),
                schema_version: "2.0".to_string(),
                cartograph_version: "0.2.0".to_string(),
                content_hash: "sha256:diagnose_v2_gate_hash_020".to_string(),
                triggers: vec![
                    "fix this panic".to_string(),
                    "diagnose test failure".to_string(),
                    "here is a stack trace".to_string(),
                    "why did this error happen".to_string(),
                    "reproduce this bug".to_string(),
                ],
                cli_command: "cartograph diagnose \"<trace>\" --xml --scaffold-test".to_string(),
                template_content: Self::TEMPLATE_DIAGNOSE,
            },
            SkillDefinition {
                id: "cartograph-complexity".to_string(),
                name: "complexity".to_string(),
                title: "Cognitive Complexity & Danger Zone Refactoring".to_string(),
                description: "Cognitive complexity & Danger Zone refactoring advisor pinpointing God classes and generating modular decomposition blueprints.".to_string(),
                schema_version: "2.0".to_string(),
                cartograph_version: "0.2.0".to_string(),
                content_hash: "sha256:complexity_v2_gate_hash_020".to_string(),
                triggers: vec![
                    "find complex code".to_string(),
                    "identify refactoring candidates".to_string(),
                    "show god classes".to_string(),
                    "find danger zones".to_string(),
                    "how can I simplify this file".to_string(),
                ],
                cli_command: "cartograph complexity --danger-zone --format xml".to_string(),
                template_content: Self::TEMPLATE_COMPLEXITY,
            },
            SkillDefinition {
                id: "cartograph-lint-arch".to_string(),
                name: "lint-arch".to_string(),
                title: "Architectural Boundary & Cycle Linter".to_string(),
                description: "Architectural boundaries & circular dependency linter enforcing Clean/Hexagonal layers and calculating MFAS cut-edge inversion.".to_string(),
                schema_version: "2.0".to_string(),
                cartograph_version: "0.2.0".to_string(),
                content_hash: "sha256:lint_arch_v2_gate_hash_020".to_string(),
                triggers: vec![
                    "check architecture rules".to_string(),
                    "are there circular dependencies".to_string(),
                    "did I violate package boundaries".to_string(),
                    "lint architecture".to_string(),
                ],
                cli_command: "cartograph lint-arch --staged --format xml".to_string(),
                template_content: Self::TEMPLATE_LINT_ARCH,
            },
            SkillDefinition {
                id: "cartograph-deadcode".to_string(),
                name: "deadcode".to_string(),
                title: "Dead Code & Ghost Dependency Purger".to_string(),
                description: "Reachability analysis uncovering unreferenced functions, orphaned UI components, unread struct fields, and ghost dependencies.".to_string(),
                schema_version: "2.0".to_string(),
                cartograph_version: "0.2.0".to_string(),
                content_hash: "sha256:deadcode_v2_gate_hash_020".to_string(),
                triggers: vec![
                    "find unused code".to_string(),
                    "clean up dead functions".to_string(),
                    "find orphaned components".to_string(),
                    "detect ghost dependencies".to_string(),
                ],
                cli_command: "cartograph deadcode --staged --format xml".to_string(),
                template_content: Self::TEMPLATE_DEADCODE,
            },
            SkillDefinition {
                id: "cartograph-extract-logic".to_string(),
                name: "extract-logic".to_string(),
                title: "Business Logic & Invariant Extractor".to_string(),
                description: "Business rule extractor synthesizing validation assertions, algorithms, and invariants into summaries, BDD Gherkin scenarios, or decision matrices.".to_string(),
                schema_version: "2.0".to_string(),
                cartograph_version: "0.2.0".to_string(),
                content_hash: "sha256:extract_logic_v2_gate_hash_020".to_string(),
                triggers: vec![
                    "explain the business logic of X".to_string(),
                    "what are the validation rules for Y".to_string(),
                    "convert logic to gherkin".to_string(),
                    "did this PR change business rules".to_string(),
                ],
                cli_command: "cartograph extract-logic --symbol \"<symbol>\" --xml".to_string(),
                template_content: Self::TEMPLATE_EXTRACT_LOGIC,
            },
            SkillDefinition {
                id: "cartograph-diagram".to_string(),
                name: "diagram".to_string(),
                title: "Architectural & Call Graph Visualizer".to_string(),
                description: "Standards-compliant architectural diagrams, call graphs, ER database schemas, and PR impact visualizations in Mermaid.js, SVG, or spatial XML.".to_string(),
                schema_version: "2.0".to_string(),
                cartograph_version: "0.2.0".to_string(),
                content_hash: "sha256:diagram_v2_gate_hash_020".to_string(),
                triggers: vec![
                    "draw architecture diagram".to_string(),
                    "visualize call graph of X".to_string(),
                    "show ER diagram for models".to_string(),
                    "diagram this PR".to_string(),
                ],
                cli_command: "cartograph diagram --query \"<symbol>\" --type callgraph --format mermaid".to_string(),
                template_content: Self::TEMPLATE_DIAGRAM,
            },
        ]
    }

    pub fn get_skill(id_or_name: &str) -> Option<SkillDefinition> {
        let clean = id_or_name.trim_start_matches("cartograph-");
        Self::all_skills()
            .into_iter()
            .find(|s| s.id == id_or_name || s.name == clean || s.name == id_or_name)
    }

    pub fn skills_for_preset(preset: SkillPreset) -> Vec<SkillDefinition> {
        match preset {
            SkillPreset::Minimal => Self::all_skills()
                .into_iter()
                .filter(|s| {
                    s.name == "locate"
                        || s.name == "impact"
                        || s.name == "test-selector"
                        || s.name == "verify"
                })
                .collect(),
            SkillPreset::Standard | SkillPreset::Strict => Self::all_skills(),
        }
    }

    pub fn skills_for_names(names: &[String]) -> Vec<SkillDefinition> {
        let all = Self::all_skills();
        names
            .iter()
            .filter_map(|name| {
                let clean = name.trim_start_matches("cartograph-");
                all.iter()
                    .find(|s| s.id == *name || s.name == clean || s.name == *name)
                    .cloned()
            })
            .collect()
    }
}
