use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

/// Complete semantic business rule extracted from source code AST
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BusinessRule {
    /// Deterministic SHA-256 fingerprint of normalized predicate + coordinates
    pub id: String,
    /// Classification category
    pub kind: RuleKind,
    /// Human-readable rule title or inferred intent
    pub title: String,
    /// Inferred or explicit domain concepts (e.g. ["pricing", "discount", "order"])
    pub domain_concepts: Vec<String>,
    /// Fully qualified symbol where rule was found (e.g. "OrderService::process_payment")
    pub enclosing_symbol: String,
    /// Path to file containing the rule
    pub file_path: PathBuf,
    /// Line and column span in source file
    pub span: RuleSpan,
    /// Normalized condition tree
    pub predicate: PredicateExpr,
    /// Resulting action or consequence
    pub consequence: Consequence,
    /// Inferred invariant direction (what must hold true for execution to succeed)
    pub invariant: InvariantCondition,
    /// Confidence score (0.0 to 1.0) distinguishing domain logic from technical noise
    pub confidence: f32,
    /// Inter-procedural depth (0 = direct in queried symbol, 1+ = called sub-function)
    pub call_depth: usize,
    /// Execution call path (e.g. ["checkout", "validate_cart", "check_inventory"])
    pub call_path: Vec<String>,
    /// Symbolic provenance & def-use dataflow origin map for variables in predicate
    pub variable_provenance: Vec<VariableProvenance>,
    /// Original raw source code snippet
    pub raw_snippet: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleKind {
    /// Input sanity, range boundary, format validation, null/type checks
    Validation,
    /// Mathematical calculations, monetary conversions, score/fee derivations
    Calculation,
    /// Finite State Machine lifecycle rules, allowed transitions
    StateTransition,
    /// Role-based, permission-based, or tenant access controls
    Authorization,
    /// Precondition that must hold prior to method execution
    Precondition,
    /// Postcondition guaranteed upon normal execution
    Postcondition,
    /// Struct, entity, or class-level integrity constraint
    Invariant,
}

impl RuleKind {
    pub fn as_str(&self) -> &str {
        match self {
            RuleKind::Validation => "validation",
            RuleKind::Calculation => "calculation",
            RuleKind::StateTransition => "state_transition",
            RuleKind::Authorization => "authorization",
            RuleKind::Precondition => "precondition",
            RuleKind::Postcondition => "postcondition",
            RuleKind::Invariant => "invariant",
        }
    }

    pub fn badge(&self) -> &str {
        match self {
            RuleKind::Validation => "🛡️ Validation",
            RuleKind::Calculation => "🧮 Calculation",
            RuleKind::StateTransition => "🔄 State Transition",
            RuleKind::Authorization => "🔐 Authorization",
            RuleKind::Precondition => "📥 Precondition",
            RuleKind::Postcondition => "📤 Postcondition",
            RuleKind::Invariant => "📋 Invariant",
        }
    }
}

impl fmt::Display for RuleKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum PredicateExpr {
    /// Binary comparison (e.g. age >= 18)
    Comparison {
        left: Box<Operand>,
        op: ComparisonOp,
        right: Box<Operand>,
    },
    /// Logical conjunction/disjunction (AND, OR, NOT)
    Logical {
        op: LogicalOp,
        operands: Vec<PredicateExpr>,
    },
    /// Membership check (e.g. role in ["ADMIN", "MANAGER"])
    Membership {
        item: Box<Operand>,
        container: Box<Operand>,
        negated: bool,
    },
    /// Method or function call predicate (e.g. email.is_valid(), text.matches(regex))
    Call {
        target: Box<Operand>,
        method: String,
        arguments: Vec<Operand>,
    },
    /// Unary check (e.g. !is_active, opt.is_none())
    Unary { op: UnaryOp, operand: Box<Operand> },
    /// Schema validation attribute (e.g. @Min(10), validates :price, presence: true)
    SchemaConstraint {
        field: String,
        constraint: String,
        parameters: Vec<(String, String)>,
    },
    /// Opaque predicate when expression is too complex to decompose
    Raw(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComparisonOp {
    Eq,      // ==
    NotEq,   // !=
    Lt,      // <
    Lte,     // <=
    Gt,      // >
    Gte,     // >=
    Matches, // =~
    StartsWith,
    EndsWith,
    Contains,
}

impl ComparisonOp {
    pub fn as_str(&self) -> &str {
        match self {
            ComparisonOp::Eq => "==",
            ComparisonOp::NotEq => "!=",
            ComparisonOp::Lt => "<",
            ComparisonOp::Lte => "<=",
            ComparisonOp::Gt => ">",
            ComparisonOp::Gte => ">=",
            ComparisonOp::Matches => "matches",
            ComparisonOp::StartsWith => "starts_with",
            ComparisonOp::EndsWith => "ends_with",
            ComparisonOp::Contains => "contains",
        }
    }

    /// Return the negated / inverted operator (e.g. < becomes >=)
    pub fn invert(&self) -> Self {
        match self {
            ComparisonOp::Eq => ComparisonOp::NotEq,
            ComparisonOp::NotEq => ComparisonOp::Eq,
            ComparisonOp::Lt => ComparisonOp::Gte,
            ComparisonOp::Lte => ComparisonOp::Gt,
            ComparisonOp::Gt => ComparisonOp::Lte,
            ComparisonOp::Gte => ComparisonOp::Lt,
            ComparisonOp::Matches => ComparisonOp::Matches,
            ComparisonOp::StartsWith => ComparisonOp::StartsWith,
            ComparisonOp::EndsWith => ComparisonOp::EndsWith,
            ComparisonOp::Contains => ComparisonOp::Contains,
        }
    }
}

impl fmt::Display for ComparisonOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LogicalOp {
    And,
    Or,
    Not,
    Imply,
}

impl LogicalOp {
    pub fn as_str(&self) -> &str {
        match self {
            LogicalOp::And => "AND",
            LogicalOp::Or => "OR",
            LogicalOp::Not => "NOT",
            LogicalOp::Imply => "IMPLIES",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnaryOp {
    Not,
    IsNull,
    IsNotNull,
    IsEmpty,
    IsNotEmpty,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum Operand {
    Identifier(String),
    PropertyAccess {
        object: Box<Operand>,
        property: String,
    },
    LiteralNumber(f64),
    LiteralString(String),
    LiteralBoolean(bool),
    Array(Vec<Operand>),
    Expression(String),
}

impl Operand {
    pub fn to_string_repr(&self) -> String {
        match self {
            Operand::Identifier(s) => s.clone(),
            Operand::PropertyAccess { object, property } => {
                format!("{}.{}", object.to_string_repr(), property)
            }
            Operand::LiteralNumber(n) => {
                if n.fract() == 0.0 {
                    format!("{:.0}", n)
                } else {
                    format!("{}", n)
                }
            }
            Operand::LiteralString(s) => format!("\"{}\"", s),
            Operand::LiteralBoolean(b) => b.to_string(),
            Operand::Array(items) => {
                let s: Vec<_> = items.iter().map(|i| i.to_string_repr()).collect();
                format!("[{}]", s.join(", "))
            }
            Operand::Expression(e) => e.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum Consequence {
    /// Execution aborted with an error/exception
    Reject {
        error_type: Option<String>,
        message: Option<String>,
        http_status: Option<u16>,
    },
    /// Early return with specific value
    Return { value: String },
    /// Variable mutated or assigned calculation
    Assign { variable: String, formula: String },
    /// State transitioned to new value
    Transition {
        from_state: Option<String>,
        to_state: String,
    },
    /// Execution allowed to proceed
    Allow,
}

/// Normalized positive invariant that MUST hold for successful execution
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InvariantCondition {
    /// Canonical verbalized description: e.g. "order.total must be >= 50.0"
    pub canonical_text: String,
    /// Formal mathematical representation: e.g. "total >= 50.0"
    pub formal_expr: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleSpan {
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
}

impl fmt::Display for RuleSpan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}-{}:{}",
            self.start_line, self.start_col, self.end_line, self.end_col
        )
    }
}

/// Symbolic provenance tracking where a variable in a rule predicate originates
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VariableProvenance {
    /// Variable name or identifier path (e.g. "overdraft_limit", "user.tier")
    pub variable: String,
    /// Origin classification
    pub source: ProvenanceSource,
    /// Inferred data type (if known from signature or AST, e.g. "f64", "Option<String>")
    pub data_type: Option<String>,
    /// Source coordinates where the variable was declared or assigned
    pub defined_at: Option<RuleSpan>,
    /// Constant or default literal value if statically resolvable
    pub resolved_value: Option<String>,
    /// Human-readable explanation of origin
    pub explanation: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "details")]
pub enum ProvenanceSource {
    /// Function or method parameter
    Parameter {
        index: usize,
        default_value: Option<String>,
    },
    /// Instance field on receiver (`self.field` / `this.field` / `obj.field`)
    FieldAccess {
        receiver: String,
        field_name: String,
    },
    /// In-file or imported constant / static definition
    Constant {
        declaration_file: Option<PathBuf>,
        value: Option<String>,
    },
    /// Local variable computed within enclosing function (backward def-use slice)
    ComputedLocal {
        assignment_expr: String,
        line: usize,
    },
    /// Return value of an external function or method invocation
    CallResult {
        call_expr: String,
        callee_symbol: Option<String>,
    },
    /// Environment variable or global configuration
    GlobalOrEnv { key: String },
    /// External unresolvable symbol
    External,
}

impl ProvenanceSource {
    pub fn as_str(&self) -> &str {
        match self {
            ProvenanceSource::Parameter { .. } => "parameter",
            ProvenanceSource::FieldAccess { .. } => "field_access",
            ProvenanceSource::Constant { .. } => "constant",
            ProvenanceSource::ComputedLocal { .. } => "computed_local",
            ProvenanceSource::CallResult { .. } => "call_result",
            ProvenanceSource::GlobalOrEnv { .. } => "global_or_env",
            ProvenanceSource::External => "external",
        }
    }
}

/// Structured decision table compiling multi-branch conditionals
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecisionTable {
    /// Title / target symbol name
    pub title: String,
    /// Condition column names (e.g. ["User Tier", "Cart Total", "Delivery Type"])
    pub condition_headers: Vec<String>,
    /// Outcome / action column names (e.g. ["Outcome", "Calculated Fee"])
    pub outcome_headers: Vec<String>,
    /// Matrix of decision rows
    pub rows: Vec<DecisionRow>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecisionRow {
    /// Row identifier (e.g. "R1", "R2")
    pub rule_id: String,
    /// Condition cell values matching condition_headers
    pub conditions: Vec<String>,
    /// Outcome cell values matching outcome_headers
    pub outcomes: Vec<String>,
    /// Source line number
    pub line: usize,
}

/// Comparison report of business rules between two Git revisions or branches
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuleDiff {
    /// Base revision (e.g. "origin/main")
    pub base_ref: String,
    /// Head revision (e.g. "HEAD")
    pub head_ref: String,
    /// Newly added business rules
    pub added: Vec<BusinessRule>,
    /// Deleted business rules
    pub removed: Vec<BusinessRule>,
    /// Modified business rules (e.g. relaxed bounds or updated formulas)
    pub modified: Vec<RuleModification>,
    /// Total count of rules in base vs head
    pub base_count: usize,
    pub head_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuleModification {
    /// Symbol enclosing the rule
    pub symbol: String,
    /// File path
    pub file_path: PathBuf,
    /// Previous rule in base
    pub base_rule: BusinessRule,
    /// New rule in head
    pub head_rule: BusinessRule,
    /// Classification of modification
    pub severity: DiffSeverity,
    /// Human-readable explanation of change
    pub explanation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DiffSeverity {
    /// Constraint was relaxed/weakened (e.g. min_length 12 -> 8)
    RelaxedConstraint,
    /// Existing business rule deleted completely
    RuleRemoved,
    /// Calculation formula altered
    CalculationChanged,
    /// Constraint was tightened (e.g. min_length 8 -> 12)
    TightenedConstraint,
    /// New rule added
    RuleAdded,
    /// Benign or non-semantic modification
    Informational,
}

impl DiffSeverity {
    pub fn is_risk(&self) -> bool {
        matches!(
            self,
            DiffSeverity::RelaxedConstraint
                | DiffSeverity::RuleRemoved
                | DiffSeverity::CalculationChanged
        )
    }

    pub fn badge(&self) -> &str {
        match self {
            DiffSeverity::RelaxedConstraint => "⚠️ RELAX",
            DiffSeverity::RuleRemoved => "❌ REMOVE",
            DiffSeverity::CalculationChanged => "🧮 CALC",
            DiffSeverity::TightenedConstraint => "🔒 TIGHT",
            DiffSeverity::RuleAdded => "➕ ADD",
            DiffSeverity::Informational => "ℹ️ INFO",
        }
    }
}

/// Summary statistics for logic extraction results
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LogicStats {
    pub total_rules: usize,
    pub validations: usize,
    pub calculations: usize,
    pub state_transitions: usize,
    pub authorizations: usize,
    pub preconditions: usize,
    pub postconditions: usize,
    pub invariants: usize,
    pub symbols_scanned: usize,
    pub elapsed_ms: u128,
}

impl LogicStats {
    pub fn record_rule(&mut self, kind: &RuleKind) {
        self.total_rules += 1;
        match kind {
            RuleKind::Validation => self.validations += 1,
            RuleKind::Calculation => self.calculations += 1,
            RuleKind::StateTransition => self.state_transitions += 1,
            RuleKind::Authorization => self.authorizations += 1,
            RuleKind::Precondition => self.preconditions += 1,
            RuleKind::Postcondition => self.postconditions += 1,
            RuleKind::Invariant => self.invariants += 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_business_rule_serialization_roundtrip() {
        let rule = BusinessRule {
            id: "sha256_rule_123".to_string(),
            kind: RuleKind::Validation,
            title: "Age limit".to_string(),
            domain_concepts: vec!["auth".to_string(), "kyc".to_string()],
            enclosing_symbol: "UserService::register".to_string(),
            file_path: PathBuf::from("src/user.rs"),
            span: RuleSpan {
                start_line: 10,
                start_col: 4,
                end_line: 12,
                end_col: 5,
            },
            predicate: PredicateExpr::Comparison {
                left: Box::new(Operand::Identifier("age".to_string())),
                op: ComparisonOp::Gte,
                right: Box::new(Operand::LiteralNumber(18.0)),
            },
            consequence: Consequence::Reject {
                error_type: Some("UnderageError".to_string()),
                message: Some("User must be 18 or older".to_string()),
                http_status: Some(400),
            },
            invariant: InvariantCondition {
                canonical_text: "age must be greater than or equal to 18".to_string(),
                formal_expr: "age >= 18".to_string(),
            },
            confidence: 0.95,
            call_depth: 0,
            call_path: vec!["register".to_string()],
            variable_provenance: vec![VariableProvenance {
                variable: "age".to_string(),
                source: ProvenanceSource::Parameter {
                    index: 1,
                    default_value: None,
                },
                data_type: Some("u32".to_string()),
                defined_at: None,
                resolved_value: None,
                explanation: "Function parameter #1".to_string(),
            }],
            raw_snippet: "if age < 18 { return Err(UnderageError); }".to_string(),
        };

        let json = serde_json::to_string(&rule).expect("Failed to serialize BusinessRule");
        let deserialized: BusinessRule =
            serde_json::from_str(&json).expect("Failed to deserialize BusinessRule");
        assert_eq!(rule, deserialized);
    }

    #[test]
    fn test_comparison_op_inversion() {
        assert_eq!(ComparisonOp::Lt.invert(), ComparisonOp::Gte);
        assert_eq!(ComparisonOp::Lte.invert(), ComparisonOp::Gt);
        assert_eq!(ComparisonOp::Gt.invert(), ComparisonOp::Lte);
        assert_eq!(ComparisonOp::Gte.invert(), ComparisonOp::Lt);
        assert_eq!(ComparisonOp::Eq.invert(), ComparisonOp::NotEq);
        assert_eq!(ComparisonOp::NotEq.invert(), ComparisonOp::Eq);
    }

    #[test]
    fn test_operand_to_string_repr() {
        let op = Operand::PropertyAccess {
            object: Box::new(Operand::Identifier("user".to_string())),
            property: "tier".to_string(),
        };
        assert_eq!(op.to_string_repr(), "user.tier");

        let num = Operand::LiteralNumber(100.0);
        assert_eq!(num.to_string_repr(), "100");
    }

    #[test]
    fn test_diff_severity_risk() {
        assert!(DiffSeverity::RelaxedConstraint.is_risk());
        assert!(DiffSeverity::RuleRemoved.is_risk());
        assert!(DiffSeverity::CalculationChanged.is_risk());
        assert!(!DiffSeverity::TightenedConstraint.is_risk());
        assert!(!DiffSeverity::RuleAdded.is_risk());
    }
}
