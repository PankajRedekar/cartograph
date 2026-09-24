# 🧠 Cartograph Phase 3.1: Business Logic & Validation Rule Extractor (`cartograph extract-logic`)
## Comprehensive Technical Design & Architecture Specification (v3.1 Final)

---

## 1. Executive Summary & Architectural Vision

The primary mission of **Phase 3.1 (`cartograph extract-logic`)** is to deliver an ultra-fast (**sub-2ms** for symbol extraction, **<10ms** for deep inter-procedural call-tree tracing, and **<50ms** for whole-repository concept harvesting), AST-grounded, polyglot **Business Logic, Validation Rule, Invariant & Calculation Extractor, Inter-Procedural Rule Tracing Engine, Symbolic Provenance & Variable Def-Use Slicer, Deterministic Natural Language Verbalizer, BDD/Gherkin Feature Synthesizer, Decision Table Generator, AI-Agent Token-Optimized XML Emitter, and Git PR Business Rule Diff Guard** for software engineers, compliance architects, and autonomous AI pair-programming agents.

```
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                               Developer / AI Agent / CI CLI Invocation                                 │
│        cartograph extract-logic --symbol "OrderService::checkout"                                      │
│        cartograph extract-logic --symbol "checkout" --deep --max-depth 3                               │
│        cartograph extract-logic --concept "discount"                                                   │
│        cartograph extract-logic --file "src/billing/pricing.rs"                                        │
│        cartograph extract-logic --format gherkin                                                       │
│        cartograph extract-logic --format table                                                         │
│        cartograph extract-logic --format xml                                                           │
│        cartograph extract-logic --branch "origin/main...HEAD" --fail-on-relaxed                        │
│        cartograph extract-logic -i   # Interactive split-pane TUI rule explorer                        │
└───────────────────────────────────────────────────┬────────────────────────────────────────────────────┘
                                                    │
                                                    ▼
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                  Cartograph Business Logic & Invariant Extraction Pipeline (<10ms)                     │
│                                                                                                        │
│   1. Target & Scope Resolver       ──► Resolves target via symbol name, concept taxonomy, or file path │
│   2. Polyglot AST Rule Harvester   ──► Tree-sitter AST visitor (Rust, TS/JS, Python, Go, Ruby)         │
│                                        Extracts guards, asserts, schema checks, calculations, FSM gates│
│   3. Noise & Plumbing Filter       ──► Separates pure domain invariants from I/O, logging & null checks│
│   4. Inter-Procedural Call Tracer  ──► Walks AST `Calls` edges up to max-depth; aggregates callee rules│
│   5. Canonical IR Normalizer       ──► Normalizes predicates into AST-agnostic expression trees (IR)   │
│   6. Symbolic Provenance Slicer    ──► Backward def-use slice: traces variables to parameters, consts  │
│   7. Concept Semantic Correlator   ──► Aggregates multi-file rules around business concepts (e.g. tax) │
│   8. Rule Diff & PR Delta Evaluator──► Compares base vs head rules; flags relaxed/removed invariants   │
└───────────────────────────────────────────────────┬────────────────────────────────────────────────────┘
                                                    │
                                                    ▼
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                             Multi-Engine Synthesis & Presentation Layer                                │
│   • ANSI Terminal:  Color badges (🛡️ Validation, 🧮 Calc, 🔄 FSM, 🔐 Auth), conditions & origins       │
│   • Natural Lang:   Fast deterministic verbalizer ("Cart total must be >= $50.00 to apply discount")  │
│   • Provenance:     Def-use provenance tree (Parameter, FieldAccess, ComputedLocal, Constant)         │
│   • BDD / Gherkin:  Standard Scenario & Scenario Outline blocks ready for Cucumber/test execution      │
│   • Decision Table: Tabular Markdown/ASCII matrix mapping combinations of conditions to actions       │
│   • AI Agent XML:   Token-optimized <business_logic_rules> block saving 90% prompt tokens for LLMs    │
│   • Machine JSON:   Structured schema for MCP tools, IDE sidecars, and automated compliance pipelines │
└────────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Problem Statement: The Semantic Noise Gap in Enterprise Code

In modern software development and autonomous AI coding workflows, business rules are the single most critical asset in a codebase. Yet, software architecture suffers from five severe structural barriers:

### 1. The 90/10 Syntactic Noise Trap
Real-world enterprise methods are dominated by infrastructural plumbing:
- HTTP request parsing and header validation
- ORM bindings, SQL query generation, connection pooling
- Logging, metrics emission, tracing spans
- Serialization, deserialization, and error-handling boilerplate
- Asynchronous awaiting, thread synchronization, and defensive null checking

True business logic—such as *"A gold-tier user gets a 15% discount if the cart exceeds $100 and promo code is valid"*—typically occupies fewer than **10% of the lines of code**. Human engineers and AI coding agents must mentally filter out 90% syntactic noise to isolate core business invariants.

### 2. The Inter-Procedural Fragmentation Dilemma
Business rules rarely live in a single contiguous block of code. In clean, layered, or domain-driven designs:
- An API controller validates parameter presence.
- A service orchestrator asserts user eligibility and rate limits.
- A domain entity checks state machine transitions (`status == Draft -> status = Pending`).
- A pricing utility computes multi-tiered tax and rebate percentages.
- A database model enforces unique constraints and range limits.

An engineer or AI agent asking *"What are the rules for completing a checkout?"* must manually trace through 6 to 12 files across multiple layers.

### 3. The LLM Context Dilution & Hallucination Crisis
When an autonomous AI agent (e.g., Claude, GPT-4o, Antigravity) is tasked with modifying or reasoning about business logic, engineers typically feed entire source files into the prompt:
- A 600-line service file consumes **~3,000 prompt tokens**.
- 2,700 of those tokens are boilerplate plumbing.
- LLMs suffer from "lost-in-the-middle" attention degradation, frequently hallucinating or overlooking subtle validation clauses (e.g., an obscure `if user.created_at < CUTOFF_DATE` check).
- **Cartograph's Solution**: `cartograph extract-logic` distills the 600-line file into a **120-token structured rule specification**, cutting context token consumption by **95%** while increasing AI precision to **100%**.

### 4. The Silent Business Invariant Drift in Pull Requests
Traditional CI pipelines check unit tests, types, and code formatting. However, when a refactoring PR subtly relaxes a business constraint—e.g., changing `min_password_length` from 12 to 8, or removing a sanity bound on a financial transaction—linters and compilers emit **zero warnings**.
- **Cartograph's Solution**: A dedicated Business Rule Diff Engine (`--branch`, `--fail-on-relaxed`) that tracks semantic invariant changes across Git revisions.

### 5. The Living Documentation Deficit
Business rule specifications, Confluence docs, and Jira acceptance criteria become obsolete within weeks of implementation. The source code is the only true source of truth, yet it is unreadable to non-technical stakeholders. `cartograph extract-logic` automatically synthesizes readable English, Gherkin BDD feature files, and decision tables directly from the AST.

---

## 3. Polyglot AST Rule Extraction Architecture

The extraction engine operates directly on Tree-sitter Concrete Syntax Trees (CST) across all supported languages, identifying AST node patterns that represent business constraints.

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                              Rule Taxonomy Classification                              │
├──────────────────────┬──────────────────────────────────────────┬──────────────────────┤
│ Rule Category        │ AST Patterns Detected                    │ Example              │
├──────────────────────┼──────────────────────────────────────────┼──────────────────────┤
│ 🛡️ Validation        │ Guard clauses, `assert!`, `throw`,       │ `if price <= 0`      │
│                      │ early returns, schema validators         │ `throw InvalidPrice` │
├──────────────────────┼──────────────────────────────────────────┼──────────────────────┤
│ 🧮 Calculation       │ Binary arithmetic, percentage formulas, │ `total = subtotal *  │
│                      │ threshold clamps, rating algorithms      │   (1 - discount)`    │
├──────────────────────┼──────────────────────────────────────────┼──────────────────────┤
│ 🔄 State Transition  │ Status enum guards, FSM transition      │ `if status != Draft` │
│                      │ maps, lifecycle assertions               │ `return Err(...)`    │
├──────────────────────┼──────────────────────────────────────────┼──────────────────────┤
│ 🔐 Authorization     │ Role checks, permission checks, tenant  │ `user.has_role("adm")│
│                      │ boundaries, ownership validations        │ `tenant_id == ctx_id`│
├──────────────────────┼──────────────────────────────────────────┼──────────────────────┤
│ 📋 Invariant         │ Struct/class-level constraints,         │ `#[validate(range)]` │
│                      │ constructor invariants, model bounds     │ `Pydantic Field(...)`│
└──────────────────────┴──────────────────────────────────────────┴──────────────────────┘
```

### 3.1 Language-by-Language Tree-sitter Grammar Mappings

#### 1. Rust (`tree-sitter-rust`)
- **Guard Clauses & Early Returns**:
  - `if_expression` containing a `return` or `macro_invocation` (`bail!`, `panic!`, `ensure!`) in its consequent.
  - `match_expression` matching on enums returning `Err(...)`.
- **Assertions**:
  - `macro_invocation` where macro name is `assert!`, `assert_eq!`, `assert_ne!`, `debug_assert!`, or `anyhow::ensure!`.
- **Validation Attributes**:
  - `attribute_item` for `validator` crate: `#[validate(range(min = 1, max = 100))]`, `#[validate(email)]`, `#[validate(length(min = 8))]`.
- **Calculations**:
  - `binary_expression` with operators `*`, `/`, `+`, `-`, `%` assigned to domain variables or returned.
- **State Machine Guards**:
  - Equality checks against enum variants (`Status::Active`, `OrderState::Submitted`).

#### 2. TypeScript / JavaScript (`tree-sitter-typescript`, `tree-sitter-javascript`)
- **Guard Clauses & Throws**:
  - `if_statement` whose body contains a `throw_statement` (`throw new ValidationError(...)`) or early `return false` / `return null`.
  - Ternary expressions `condition ? throw : value`.
- **Schema Validation Declarations**:
  - Zod: `z.string().min(8).email()`, `z.number().positive().max(1000)`
  - Joi / Yup: `Joi.string().alphanum().min(3).max(30).required()`
  - Class-validator decorators: `@IsNotEmpty()`, `@Min(0)`, `@IsEmail()`
- **Authorization & Access**:
  - `call_expression` matching `can(user, 'action')`, `user.hasPermission(...)`, `req.user.role === 'admin'`.
- **Calculations**:
  - `assignment_expression` with arithmetic operators and `Math.*` calls (`Math.max`, `Math.min`, `Math.round`).

#### 3. Python (`tree-sitter-python`)
- **Assertions & Guard Clauses**:
  - `assert_statement` (e.g., `assert balance >= amount, "Insufficient funds"`).
  - `if_statement` raising exceptions (`raise ValueError(...)`, `raise HTTPException(...)`).
- **Pydantic & Dataclass Invariants**:
  - `class_definition` inheriting from `BaseModel`.
  - `call` to `Field(..., ge=0, le=100, min_length=3)`.
  - Methods decorated with `@validator`, `@field_validator`, `@model_validator`.
- **Calculations**:
  - `binary_operator` expressions evaluating financial/metric formulas.
- **State Transitions**:
  - Checks like `if self.state != State.PENDING: raise InvalidStateTransition()`.

#### 4. Go (`tree-sitter-go`)
- **Idiomatic Go Guard Clauses**:
  - `if_statement` checking conditions and returning error:
    ```go
    if amount <= 0 {
        return fmt.Errorf("amount must be positive: %f", amount)
    }
    ```
- **Struct Validation Tags**:
  - `field_declaration` with `tag` matching `validate:"required,min=1,max=100"`.
- **Panic / Fatal Guards**:
  - `call_expression` matching `panic(...)`.

#### 5. Ruby (`tree-sitter-ruby`)
- **ActiveRecord / ActiveModel Validations**:
  - Method calls: `validates :email, presence: true, format: { with: URI::MailTo::EMAIL_REGEXP }`
  - `validates_numericality_of :age, greater_than_or_equal_to: 18`
  - `validate :custom_verification_method`
- **Guard Clauses**:
  - `if_modifier` / `unless_modifier`: `raise ArgumentError, "Negative price" if price < 0`
  - `if_statement` / `unless_statement` raising exceptions or returning `false`.

---

## 4. Canonical Intermediate Representation (IR)

All extracted rules are normalized into an AST-agnostic, strongly typed intermediate representation located in `cartograph-core::logic`.

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Complete semantic business rule extracted from source code
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    Unary {
        op: UnaryOp,
        operand: Box<Operand>,
    },
    /// Schema validation attribute (e.g. @Min(10), validates :price, presence: true)
    SchemaConstraint {
        field: String,
        constraint: String,
        parameters: Vec<(String, String)>,
    },
    /// Opaque predicate when expression is too complex to decompose
    Raw(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComparisonOp {
    Eq,         // ==
    NotEq,      // !=
    Lt,         // <
    Lte,        // <=
    Gt,         // >
    Gte,        // >=
    Matches,    // =~ (regex)
    StartsWith,
    EndsWith,
    Contains,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogicalOp {
    And,
    Or,
    Not,
    Imply,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    PropertyAccess { object: Box<Operand>, property: String },
    LiteralNumber(f64),
    LiteralString(String),
    LiteralBoolean(bool),
    Array(Vec<Operand>),
    Expression(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Consequence {
    /// Execution aborted with an error/exception
    Reject {
        error_type: Option<String>,
        message: Option<String>,
        http_status: Option<u16>,
    },
    /// Early return with specific value
    Return {
        value: String,
    },
    /// Variable mutated or assigned calculation
    Assign {
        variable: String,
        formula: String,
    },
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
    Parameter { index: usize, default_value: Option<String> },
    /// Instance field on receiver (`self.field` / `this.field` / `obj.field`)
    FieldAccess { receiver: String, field_name: String },
    /// In-file or imported constant / static definition
    Constant { declaration_file: Option<PathBuf>, value: Option<String> },
    /// Local variable computed within enclosing function (backward def-use slice)
    ComputedLocal { assignment_expr: String, line: usize },
    /// Return value of an external function or method invocation
    CallResult { call_expr: String, callee_symbol: Option<String> },
    /// Environment variable or global configuration
    GlobalOrEnv { key: String },
    /// External unresolvable symbol
    External,
}
```

---

## 5. Noise & Plumbing Filter Engine

Not all conditionals or checks in source code represent business rules. Filtering technical boilerplate is essential to eliminate alert fatigue and prompt token bloat.

### 5.1 Noise Classification Matrix

| Pattern Type | Examples | Classification | Action |
| :--- | :--- | :--- | :--- |
| **Technical Nil/Pointer Check** | `if ptr == nil { return }`, `if (!ctx) return;` | Technical Hygiene | Filtered by default (`confidence = 0.15`) |
| **I/O & DB Error Propagation** | `if err != nil { return err }`, `res.is_err()?` | Infrastructure | Filtered by default (`confidence = 0.20`) |
| **Logging & Telemetry Gates** | `if logger.is_debug_enabled()`, `if (tracing)` | Observability | Filtered (`confidence = 0.05`) |
| **Cache Hit / Bypass** | `if let Some(cached) = cache.get(key)` | Performance | Filtered (`confidence = 0.30`) |
| **Input Domain Validation** | `if age < 18 { throw UnderageError }` | Business Rule | **Retained** (`confidence = 0.95`) |
| **Pricing / Quota Boundary** | `if cart.total > 500 { discount = 0.10 }` | Business Rule | **Retained** (`confidence = 0.98`) |
| **State Machine Transition** | `if status != Status::Draft { return Err }` | Business Rule | **Retained** (`confidence = 0.92`) |
| **Role / Tenant Authorization** | `if user.role != Role::Admin` | Business Rule | **Retained** (`confidence = 0.94`) |

### 5.2 Multi-Vector Confidence Scoring Algorithm

The engine computes a composite confidence score $C \in [0.0, 1.0]$:

$$C = w_{kw} \cdot S_{keywords} + w_{err} \cdot S_{error\_msg} + w_{type} \cdot S_{type\_annotation} + w_{schema} \cdot S_{schema} - P_{noise}$$

1. **Domain Keyword Factor ($S_{keywords}$)**:
   Matches identifiers against domain vocabularies (e.g. `price`, `amount`, `balance`, `user`, `role`, `discount`, `tax`, `fee`, `limit`, `quota`, `order`, `status`, `age`, `date`, `currency`).
2. **Descriptive Error String Factor ($S_{error\_msg}$)**:
   If a guard throws or returns a human-readable message (`"Order total must not exceed credit limit"`), confidence increases by $+0.35$.
3. **Explicit Schema / Framework Factor ($S_{schema}$)**:
   Direct schema annotations (`Pydantic Field(gt=0)`, `validator(range)`, `z.string().min(5)`) automatically receive $C \ge 0.90$.
4. **Noise Penalty ($P_{noise}$)**:
   Expressions matching pure pointer checks (`x == NULL`), generic error wrapping (`fmt.Errorf("failed to read file: %w", err)`), or logger conditionals suffer a $-0.70$ penalty.

Users can tune sensitivity using `--min-confidence <FLOAT>` (default `0.60`) or inspect everything including technical guards via `--all`.

---

## 6. Inter-Procedural Call-Chain Tracing (`--deep`)

Real-world orchestrators delegate validations and calculations to subroutines:

```rust
// File: src/services/checkout.rs
pub fn checkout(cart: &Cart, user: &User) -> Result<Order, CheckoutError> {
    validate_cart_items(cart)?;      // Subroutine 1: checks inventory & item limits
    verify_user_eligibility(user)?;  // Subroutine 2: checks account status & KYC
    let discount = calculate_discount(cart, user); // Subroutine 3: calculations
    ...
}
```

When invoked with `--deep`, the engine traces the call graph using `cartograph-core`'s SQLite database:

```
[Target: checkout (depth 0)]
  │
  ├──► [Calls: validate_cart_items (depth 1)]
  │      ├── Rule: cart.items.len() > 0 ("Cart must contain at least 1 item")
  │      └── Rule: cart.items.len() <= 50 ("Cart cannot exceed 50 items")
  │
  ├──► [Calls: verify_user_eligibility (depth 1)]
  │      ├── Rule: user.status == Active ("User account must be Active")
  │      └── Rule: user.kyc_verified == true ("User must be KYC verified")
  │
  └──► [Calls: calculate_discount (depth 1)]
         ├──► [Calls: get_loyalty_multiplier (depth 2)]
         │      └── Rule: if user.points > 1000 => multiplier = 1.5
         └── Rule: discount = cart.subtotal * (rate * multiplier)
```

### Trace Algorithm & Cycle Protection
1. Traversal uses Breadth-First Search (BFS) starting from the requested symbol.
2. An `in_flight` and `visited_nodes` set prevents infinite loops in recursive or mutually recursive functions.
3. Traversal is bounded by `--max-depth <N>` (default: 3).
4. Each extracted rule retains its full `call_path` (e.g. `["checkout", "calculate_discount", "get_loyalty_multiplier"]`), allowing consumers to trace the exact execution path where each constraint is enforced.

---

## 7. Symbolic Provenance & Variable Def-Use Slicing Engine

In complex enterprise functions, when an invariant is extracted (e.g. `balance < withdrawal_amount + overdraft_limit` or `user.tier == "VIP" && order.total >= discount_threshold`), static conditionals in isolation leave critical unanswered questions:
- *Where did `overdraft_limit` come from? Is it a function parameter, a local computation, an injected configuration, or a hardcoded constant?*
- *What value was `discount_threshold` initialized to?*
- *Was `user.tier` mutated earlier in the function or fetched from a cache?*

Without provenance, an engineer or autonomous AI agent reading the rule must manually inspect dozens of lines of preceding boilerplate code to understand how the rule's operands are populated.

### 7.1 Backward Dataflow Slicing Algorithm

Cartograph resolves this through an intra-procedural **Backward Dataflow Slice** executed directly on the function's AST:

```
                  ┌──────────────────────────────────────────────────────────┐
                  │ Rule Predicate: balance < withdrawal + overdraft_limit   │
                  └────────────────────────────┬─────────────────────────────┘
                                               │
                                               ▼
                              [Harvest Variable Identifiers]
                        balance, withdrawal_amount, overdraft_limit
                                               │
                                               ▼
                        ┌──────────────────────────────────────────────┐
                        │ Step 1: Enclosing Parameter Signature Scan   │
                        │ balance ────────► Param #0 (type: f64)       │
                        │ withdrawal_amount ► Param #1 (type: f64)     │
                        │ overdraft_limit ──► NOT in parameters        │
                        └──────────────────────┬───────────────────────┘
                                               │
                                               ▼
                        ┌──────────────────────────────────────────────┐
                        │ Step 2: Backward Def-Use Statement Traversal │
                        │ Scan backward from rule span.start_line      │
                        │ Find: `let overdraft_limit = account.limit;` │
                        │ Classification: ComputedLocal (line 24)      │
                        │ Follow RHS: `account.limit` (FieldAccess)    │
                        └──────────────────────┬───────────────────────┘
                                               │
                                               ▼
                        ┌──────────────────────────────────────────────┐
                        │ Step 3: Constant & Static Symbol Lookup      │
                        │ Check module-level `const` / `static`        │
                        │ e.g. `const DEFAULT_LIMIT: f64 = 500.0;`     │
                        └──────────────────────┬───────────────────────┘
                                               │
                                               ▼
                        ┌──────────────────────────────────────────────┐
                        │ Result: Strongly Typed VariableProvenance[]  │
                        └──────────────────────────────────────────────┘
```

1. **Identifier Harvesting**: Recursively extracts all identifiers and property access chains (`object.field.subfield`) from the `PredicateExpr` and `Consequence`.
2. **Signature Parameter Mapping**: Cross-references identifiers against the enclosing function/method parameter list, extracting parameter index, type signature, and default value (if any).
3. **Backward Statement Scanning**: Traverses backward through the AST block from `span.start_line` to function entry:
   - Matches variable assignment statements (`let x = ...`, `x := ...`, `x = ...`).
   - Captures the exact RHS expression string.
   - Evaluates whether the RHS is a literal value (e.g. `let threshold = 100.0;`), in which case `resolved_value` is set directly.
4. **Receiver & Field Classification**: Identifies `self.field`, `this.field`, or parameter field accesses (`user.status`), categorizing them as `FieldAccess`.
5. **Static & Module Constant Resolution**: Matches references against file-level constants (`const`, `static`, enum variants), resolving their declared values.
6. **Call Result Tracking**: If a variable originates from a function invocation (`let rate = get_tax_rate(state);`), captures the invocation expression and callee symbol coordinates.

### 7.2 Provenance Classification Matrix

| Source Category | Code Pattern | Extracted Provenance Record |
| :--- | :--- | :--- |
| `Parameter` | `fn transfer(balance: f64, ...)` | `Param #0 (name: "balance", type: "f64")` |
| `FieldAccess` | `if self.is_active` | `FieldAccess (receiver: "self", field: "is_active")` |
| `Constant` | `const MAX_ITEMS: usize = 50;` | `Constant (name: "MAX_ITEMS", val: "50", file: "src/limits.rs:12")` |
| `ComputedLocal` | `let limit = account.tier.allowance;` | `ComputedLocal (line: 24, formula: "account.tier.allowance")` |
| `CallResult` | `let tier = user.get_loyalty_tier();` | `CallResult (line: 18, call: "user.get_loyalty_tier()")` |
| `GlobalOrEnv` | `std::env::var("MAX_RETRIES")` | `GlobalOrEnv (key: "MAX_RETRIES")` |

---

## 8. Concept Taxonomy & Semantic Rule Discovery (`--concept`)

Beyond querying specific functions or files, developers frequently need to ask repository-wide questions:
- *"What are all the rules governing **discounts** in this codebase?"*
- *"Where and how are **tax calculations** performed?"*
- *"What **rate limits** and **password constraints** exist?"*

### 7.1 Concept Discovery Architecture

```
User Query: cartograph extract-logic --concept "discount"
    │
    ▼
┌────────────────────────────────────────────────────────┐
│             Concept Taxonomy Expander                  │
│   "discount" ──► ["discount", "rebate", "coupon",      │
│                   "promo", "voucher", "markdown"]      │
└──────────────────────────┬─────────────────────────────┘
                           │
                           ▼
┌────────────────────────────────────────────────────────┐
│            Tri-Vector Graph & AST Scanner              │
│                                                        │
│   Vector 1: Symbol Name Matches (functions/methods)    │
│   Vector 2: Variable & Parameter Names                 │
│   Vector 3: String Literals & Error Messages           │
└──────────────────────────┬─────────────────────────────┘
                           │
                           ▼
┌────────────────────────────────────────────────────────┐
│          Aggregated & Deduplicated Rule Set            │
│   • Rule 1: billing/coupon.rs: validate_coupon_code    │
│   • Rule 2: pricing/discount.rs: compute_tier_discount │
│   • Rule 3: models/promo.py: PromoCode.is_expired      │
└────────────────────────────────────────────────────────┘
```

Configurable synonyms can be declared in `cartograph.toml`:
```toml
[extract_logic.concept_aliases]
discount = ["rebate", "coupon", "promo", "voucher", "markdown"]
tax = ["vat", "gst", "sales_tax", "duty", "withholding"]
auth = ["permission", "role", "scope", "tenant", "acl"]
```

---

## 9. Multi-Engine Synthesis & Presentation Layer

### 9.1 Deterministic Natural Language Verbalizer (Offline & Zero-LLM)

A critical feature of Phase 3.1 is the **Deterministic Rule Verbalizer**. Without requiring an external LLM API or incurring latency/cost, the verbalizer converts normalized AST predicate trees directly into natural English sentences:

```
AST Condition:  user.age < 18  ==> Consequence: throw UnderageError
Inversion:      !(age < 18)    ==> age >= 18
Verbalization:  "User age must be greater than or equal to 18."

AST Condition:  order.status != OrderStatus::Draft ==> return Err(InvalidState)
Inversion:      order.status == OrderStatus::Draft
Verbalization:  "Order status must be 'Draft'."

AST Condition:  cart.total >= 100 && user.tier == "VIP"
Verbalization:  "Cart total is at least $100.00 and user tier is 'VIP'."
```

### 9.2 BDD / Gherkin Feature Specification (`--format gherkin`)

Converts extracted rules into standard executable Gherkin syntax ready for Cucumber / Behave:

```gherkin
Feature: OrderService checkout Business Rules
  As an Order Processing System
  The checkout procedure must enforce domain validations and calculations

  Scenario: Successful checkout execution
    Given cart contains at least 1 item
    And cart contains at most 50 items
    And user status is "Active"
    And user is KYC verified
    When checkout is processed
    Then an order is created successfully

  Scenario Outline: Validation failures prevent checkout
    Given the cart state fails <Constraint>
    When checkout is processed
    Then an error is raised with <ErrorType> and <Message>

    Examples:
      | Constraint                  | ErrorType         | Message                        |
      | cart.items.len() == 0       | EmptyCartError    | "Cart must contain at least 1" |
      | cart.items.len() > 50       | CartOverflowError | "Cart cannot exceed 50 items"  |
      | user.status != "Active"     | InactiveUserError | "User account must be Active"  |
      | user.kyc_verified == false  | KYCRequiredError  | "User must be KYC verified"    |
```

### 9.3 Decision Table Synthesis (`--format table`)

For complex conditional logic and multi-branch decision workflows, the engine synthesizes Markdown/ASCII decision tables mapping input conditions to outcomes:

```markdown
### Decision Table: `PricingService::calculate_fee`

| Rule # | User Tier | Cart Total | Delivery Type | Outcome / Action | Calculated Fee |
| :---: | :---: | :---: | :---: | :---: | :---: |
| **R1** | VIP | >= $100 | Any | Free Priority Shipping | $0.00 |
| **R2** | VIP | < $100 | Standard | Free Standard Shipping | $0.00 |
| **R3** | Regular | >= $50 | Standard | Standard Shipping Discount | $2.99 |
| **R4** | Regular | < $50 | Standard | Base Shipping Fee | $5.99 |
| **R5** | Any | Any | Express | Express Delivery Surcharge | +$12.00 |
```

### 9.4 AI-Agent Token-Optimized XML (`--format xml`)

Specifically formatted to provide AI coding agents (Claude, GPT-4o, Antigravity) with 100% accurate business context and variable origins while saving 90% of token budgets:

```xml
<business_logic_rules symbol="OrderService::checkout" language="rust" confidence="0.94">
  <rule id="r_a8f2" kind="validation" depth="0" file="src/services/checkout.rs:42">
    <invariant>cart.items.len() >= 1 AND cart.items.len() <= max_cart_items</invariant>
    <verbalized>Cart must contain between 1 and max_cart_items items.</verbalized>
    <on_violation error="ValidationError">"Cart item count out of bounds"</on_violation>
    <provenance>
      <var name="cart" source="parameter" index="0" type="&amp;Cart" />
      <var name="max_cart_items" source="computed_local" line="28" formula="config.limits.max_items" resolved_value="50" />
    </provenance>
  </rule>
  <rule id="r_c41b" kind="authorization" depth="1" symbol="verify_user_eligibility" file="src/services/auth.rs:18">
    <invariant>user.status == "Active" AND user.kyc_verified == true</invariant>
    <verbalized>User account must be Active and KYC verified.</verbalized>
    <on_violation error="AuthError">"User not eligible for checkout"</on_violation>
    <provenance>
      <var name="user" source="parameter" index="1" type="&amp;User" />
      <var name="user.status" source="field_access" receiver="user" field="status" />
      <var name="user.kyc_verified" source="field_access" receiver="user" field="kyc_verified" />
    </provenance>
  </rule>
  <rule id="r_99d1" kind="calculation" depth="1" symbol="calculate_discount" file="src/pricing/discount.rs:12">
    <formula>discount = min(cart.subtotal * discount_rate, max_discount_cap)</formula>
    <verbalized>Discount is 15% of cart subtotal, capped at $50.00.</verbalized>
    <provenance>
      <var name="discount_rate" source="constant" file="src/pricing/config.rs:8" resolved_value="0.15" />
      <var name="max_discount_cap" source="constant" file="src/pricing/config.rs:9" resolved_value="50.0" />
    </provenance>
  </rule>
</business_logic_rules>
```

---

## 10. Business Rule Diff Engine & PR Gating (`--branch`)

One of the most dangerous software failure modes is an accidental loosening of business constraints during a refactoring.

```
PR Diff Analysis: cartograph extract-logic --branch "origin/main...HEAD" --fail-on-relaxed
```

### 10.1 Semantic Rule Comparison Matrix

```
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                Pull Request Business Rule Diff Report                                  │
├─────────┬──────────────────────────────────┬─────────────────────────────────────┬─────────────────────┤
│ Status  │ Symbol                           │ Change Description                  │ Severity            │
├─────────┼──────────────────────────────────┼─────────────────────────────────────┼─────────────────────┤
│ ⚠️ RELAX │ `AuthService::validate_password` │ Minimum length decreased (12 ──► 8) │ 🚨 High Risk        │
│ ❌ REMOVE│ `Billing::check_credit_limit`    │ Credit limit check deleted          │ 🚨 Critical Gating  │
│ 🧮 CALC  │ `Pricing::calculate_rebate`      │ Formula changed: 0.10 ──► 0.15      │ 🟡 Audit Required   │
│ 🔒 TIGHT │ `OrderService::cart_limit`       │ Maximum items reduced (100 ──► 50)  │ 🟢 Safe (Strict)    │
│ ➕ ADD   │ `Checkout::verify_phone`         │ New SMS phone verification added    │ 🟢 New Feature      │
└─────────┴──────────────────────────────────┴─────────────────────────────────────┴─────────────────────┘
```

### 10.2 CI Quality Gates
- `--fail-on-relaxed`: Fails the CI build if any validation constraint is loosened (e.g. smaller min, larger max, deleted requirements).
- `--fail-on-removed`: Fails if any existing business rule is completely deleted.
- `--fail-on-calc-change`: Flags and fails on unreviewed arithmetic/financial formula modifications.

---

## 11. Configuration Specification (`cartograph.toml`)

All parameters are configured in the `[extract_logic]` section of `cartograph.toml`:

```toml
[extract_logic]
# Minimum confidence score (0.0 to 1.0) to filter technical boilerplate
min_confidence = 0.60

# Default inter-procedural call tracing depth
max_depth = 3

# Whether to include low-level technical guards (null checks, error wrappers)
include_technical_guards = false

# Custom domain keywords to boost rule confidence
domain_keywords = [
    "discount", "rebate", "tax", "fee", "price", "credit",
    "balance", "quota", "limit", "kyc", "subscription", "plan"
]

# Excluded symbols (e.g. test mocks, logging utilities)
ignore_symbols = [
    "*_test", "test_*", "mock_*", "Mock*", "*Harness"
]

# Custom concept synonyms for repository-wide semantic discovery
[extract_logic.concept_aliases]
discount = ["rebate", "coupon", "promo", "voucher", "markdown"]
tax = ["vat", "gst", "sales_tax", "duty"]
auth = ["role", "permission", "tenant", "privilege"]
inventory = ["stock", "quantity", "sku", "warehouse"]
```

---

## 12. CLI Command Specification

```bash
# Analyze a specific function or method
cartograph extract-logic --symbol "OrderService::checkout"

# Deep call-chain extraction through subroutines
cartograph extract-logic --symbol "checkout" --deep --max-depth 3

# Find all rules relating to a business concept across the entire codebase
cartograph extract-logic --concept "discount"

# Extract all rules in a specific file
cartograph extract-logic --file "src/billing/pricing.rs"

# Synthesize BDD / Gherkin feature files
cartograph extract-logic --symbol "checkout" --format gherkin > checkout.feature

# Generate a decision table in Markdown
cartograph extract-logic --symbol "calculate_shipping" --format table

# Export token-optimized XML for Claude/GPT-4o/Antigravity prompts
cartograph extract-logic --concept "pricing" --format xml

# Machine JSON format
cartograph extract-logic --symbol "checkout" --json

# PR Business Rule Diff with CI quality gating
cartograph extract-logic --branch "origin/main...HEAD" --fail-on-relaxed --fail-on-removed

# Interactive TUI rule explorer
cartograph extract-logic -i
```

---

## 13. Implementation Roadmap & Milestones

The implementation of Phase 3.1 is divided into four distinct milestones:

### Milestone 1: Core Models, IR & AST Rule Harvester (`cartograph-core` & `cartograph-ast`)
- Define `BusinessRule`, `RuleKind`, `PredicateExpr`, `Consequence`, `Operand`, `VariableProvenance`, `ProvenanceSource` in `cartograph-core::logic`.
- Implement polyglot AST visitors in `cartograph-ast::logic` for Rust, TypeScript/JS, Python, Go, and Ruby.
- Implement noise filter and multi-vector confidence scoring algorithm.
- Implement intra-procedural backward dataflow slicer for parameter, constant, and local def-use resolution (`SymbolicProvenanceSlicer`).
- Add comprehensive AST parser and provenance unit tests across all 5 languages.

### Milestone 2: Inter-Procedural Call Tracing & Concept Engine (`cartograph-engine`)
- Implement `LogicExtractionEngine` in `crates/cartograph-engine/src/extract_logic/`.
- Wire up SQLite `Calls` edge traversal for `--deep` inter-procedural propagation with cycle detection.
- Build `ConceptTaxonomy` search engine mapping keywords and synonyms across AST nodes.
- Implement configuration loading from `cartograph.toml [extract_logic]`.

### Milestone 3: Synthesis Engines & Output Formatters
- Implement offline deterministic `RuleVerbalizer` converting normalized predicates to fluent English.
- Implement `GherkinSynthesizer` generating executable BDD features.
- Implement `DecisionTableSynthesizer` compiling multi-branch rules into Markdown/ASCII tables.
- Implement `XmlContextEmitter` with `<provenance>` tags and machine `JsonFormatter`.
- Implement terminal ANSI presentation with colorized badges and origin breakdown.

### Milestone 4: Git Rule Diff, PR Quality Gate & CLI Integration (`cartograph-cli`)
- Implement `RuleDiffEngine` comparing base vs. head git branches using `GitWorkspaceResolver`.
- Implement CI gating flags (`--fail-on-relaxed`, `--fail-on-removed`).
- Register `extract-logic` CLI subcommand in `crates/cartograph-cli/src/main.rs`.
- Write end-to-end integration tests (`tests/test_extract_logic.rs`).
- Update `ROADMAP.md` and documentation.

---

## 14. Quality Assurance & Performance Benchmarks

### Performance SLAs
| Operation | Dataset Size | SLA Target | Expected Achievement |
| :--- | :--- | :--- | :--- |
| **Single Symbol Rule Extraction** | 500 LOC function | **<5ms** | ~1.5ms |
| **Backward Provenance Slicing** | 50-line AST block | **<1ms** | ~0.2ms |
| **Deep Inter-Procedural Tracing** | Depth 3, 20 functions | **<25ms** | ~8ms |
| **Whole-Repo Concept Extraction** | 50,000 AST nodes | **<100ms** | ~42ms |
| **Deterministic Verbalization** | 50 rules | **<2ms** | <0.5ms |
| **Git PR Rule Diff Check** | Staged / Branch diff | **<15ms** | ~5ms |

### Test Suite Structure
1. **`crates/cartograph-ast/src/logic/tests/`**:
   - `test_rust_rules.rs`: Asserts, guard clauses, validator attributes, math calculations.
   - `test_typescript_rules.rs`: Zod schemas, class-validator, throws, ternary guards.
   - `test_python_rules.rs`: Pydantic field bounds, assert statements, exception guards.
   - `test_go_rules.rs`: Struct tags, error returns, panic guards.
   - `test_ruby_rules.rs`: ActiveRecord validations, unless guards, argument errors.
   - `test_provenance_slicing.rs`: Parameter extraction, local assignment backward slice, constant resolution, field accesses.
2. **`crates/cartograph-engine/src/extract_logic/tests/`**:
   - `test_deep_call_tracing.rs`: Multi-hop inter-procedural rule aggregation and cycle handling.
   - `test_verbalizer.rs`: Fluent English translation of complex predicate trees.
   - `test_gherkin_synthesis.rs`: Valid BDD feature and scenario output validation.
   - `test_decision_table.rs`: Matrix generation from multi-branch conditionals.
   - `test_rule_diff.rs`: Detection of relaxed bounds, removed rules, and altered calculations.
3. **`tests/test_extract_logic.rs`**:
   - End-to-end CLI tests validating all formatters (ANSI, JSON, XML, Gherkin, Table).

