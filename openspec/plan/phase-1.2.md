# 🗺️ Cartograph Phase 1.2: Multi-Language Tree-Sitter AST Engine (`cartograph-ast`)
## Comprehensive Technical Design & Architecture Specification

---

## 1. Executive Summary & Architectural Vision

The core mission of **Phase 1.2 (`cartograph-ast`)** is to build Cartograph's high-throughput, multi-language structural parsing engine.

Modern codebases are inherently polyglot: a web application often combines a **Python/Django** or **Ruby on Rails** backend, a **TypeScript/React** frontend, **SQL** migrations, **Rust/Go** microservices, and **Markdown** documentation. To empower AI coding agents and developers with instant structural context, Cartograph transforms raw source code files across all these ecosystems into a unified, deterministic graph of **AST Nodes** (classes, methods, routes, models, components) and **Graph Edges** (imports, calls, inheritance, route-mappings, test coverage).

### Core Architectural Pillars
1. **Hybrid Extraction Engine (Declarative `.scm` Queries + Semantic Procedural Walkers)**:
   - Declarative Tree-sitter S-Expression queries (`queries/<lang>/tags.scm`) handle 85% of syntactic declarations, signatures, and trivia docstrings in high-speed native C loops.
   - Procedural Rust visitors handle framework-specific DSLs (Django `urlpatterns`, Rails `routes.rb`, ActiveRecord macros).
2. **Subtree Merkle Hashing (`node.checksum`)**:
   - Each AST node computes a deterministic hash of its normalized token subtree.
   - Unlocks **Differential SQLite Updates**: modifying 1 function in a 2,000-line file only mutates that single row in SQLite, slashing write I/O by >95% and isolating semantic code diffs for AI PR review bots.
3. **Type-Assisted Call Resolution**:
   - Analyzes type annotations (Python `def f(user: User)`, TypeScript `user: User`, Rust, Go) and RHS assignments to determine receiver types.
   - Upgrades dynamic candidate calls (`user.save()`) into deterministic static calls (`cartograph://...::User::save#method`), drastically reducing false positives in blast-radius impact analysis.
4. **Symbol Visibility & Export Boundary Detection**:
   - Extracts language-specific access levels (`public`, `private`, `protected`, `pub(crate)`, Go casing) into `Node.metadata.visibility`.
   - Distinguishes between internal implementation details and public API contracts, enabling architectural boundary linting (Phase 2.4) and dead-code detection (Phase 2.6).
5. **Multi-Language Embedded Code Slicing (Inline SQL & Polyglot Files)**:
   - Detects inline SQL string literals in Python/Ruby database queries (`cursor.execute("SELECT ... FROM orders")`) and passes them to `SqlParser` to establish `EdgeKind::References` directly to the `orders` model.
   - Handles Markdown code blocks and polyglot web templates.
6. **Generated Code & Vendor Filtering**:
   - Header inspection in `AstRegistry` detects protobufs (`*.pb.go`), minified files (`*.min.js`), and generated headers (`@generated`, `DO NOT EDIT`), tagging them with `is_generated: true` or skipping them via `ParseOptions.skip_generated`.
7. **Incremental Re-Parsing & CST Tree Caching**:
   - Exploits Tree-sitter's incremental parsing with `old_tree` and `InputEdit`. File save updates in the watcher run in **<50 microseconds**.
8. **Exact Token Byte Spans (`Span`)**:
   - Every node and reference tracks `start_byte..end_byte` alongside line/col, enabling zero-copy code slicing (`&source[start_byte..end_byte]`) for LLM context prompts.
9. **Deterministic Canonical Identifiers & Fault Tolerance**:
   - Entities resolve to `cartograph://<relative_filepath>::<scope_path>::<symbol_name>#<kind>`.
   - Strict Zero-Panic Contract retaining valid subtrees around `ERROR` and `MISSING` nodes.

---

## 2. Pipeline Architecture & Component Topology

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                                 Source Code Input                                      │
│               (.py, .ts, .tsx, .js, .jsx, .rb, .rs, .go, .sql, .md)                    │
└───────────────────────────────────────────┬────────────────────────────────────────────┘
                                            │
                                            ▼
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                  Language Detector, Generated Filter & Parser Registry                 │
│       • Header Scanner: Detects @generated, DO NOT EDIT, minified code                 │
│       • Extension & Shebang Matcher (.py, .ts, #!/usr/bin/env ruby)                    │
│       • TS vs TSX Grammar Disambiguation                                               │
│       • Thread-Local Parser Cache (zero-allocation Tree-sitter instance reuse)         │
└───────────────────────────────────────────┬────────────────────────────────────────────┘
                                            │
                                            ▼
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                 Tree-Sitter Concrete Syntax Tree (CST) & Incremental Parser            │
│       • Optional old_tree + InputEdit -> Microsecond (<50µs) incremental re-parse      │
│       • Fault-tolerant error recovery (retains valid subtrees around ERROR nodes)      │
└───────────────────────────────────────────┬────────────────────────────────────────────┘
                                            │
                                            ▼
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                          Declarative S-Expression Query Engine                         │
│       • Pre-compiled .scm tag queries (compiled once at startup via LazyLock)          │
│       • High-speed QueryCursor match loop in native Tree-sitter C runtime              │
│       • Extracts: @definition.class, @definition.function, @reference.call, @docstring │
└───────────────────────────────────────────┬────────────────────────────────────────────┘
                                            │
                                            ▼
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                 Language-Specific Semantic Extractor & Framework Walkers               │
│   ┌────────────────────────┬─────────────────────────┬─────────────────────────────┐   │
│   │     Python / Django    │    TypeScript / React   │        Ruby / Rails         │   │
│   │  • Models & URLs       │  • JSX Components       │  • ActiveRecord Associations│   │
│   │  • Views & Tasks       │  • Hooks & Interfaces   │  • ActionControllers        │   │
│   │  • Pytest Functions    │  • ES Imports / Exports │  • routes.rb & RSpec        │   │
│   │  • Inline SQL Slicing  │  • Type-Assisted Calls  │  • Inline SQL Slicing       │   │
│   ├────────────────────────┼─────────────────────────┼─────────────────────────────┤   │
│   │        Rust / Go       │         SQL DDL         │      Markdown & Config      │   │
│   │  • Structs & Impls     │  • CREATE TABLE / VIEW  │  • Structural Headings      │   │
│   │  • Traits & Receivers  │  • Foreign Keys         │  • Code Fences              │   │
│   │  • Visibility (pub)    │  • Column Metadata      │  • Cross-Doc Links          │   │
│   └────────────────────────┴─────────────────────────┴─────────────────────────────┘   │
└───────────────────────────────────────────┬────────────────────────────────────────────┘
                                            │
                                            ▼
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                 Subtree Merkle Hashing & Canonical Entity Packaging                    │
│       • Computes node.checksum for each AST node (normalized token hashing)            │
│       • Tags visibility: public, private, protected, package-internal                  │
└───────────────────────────────────────────┬────────────────────────────────────────────┘
                                            │
                                            ▼
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                                  ParsedFile Result                                     │
│   1. Nodes: Canonical URIs, Exact Byte Spans, Signatures, Docstrings, Checksums, Kinds │
│   2. Intra-File Edges: Defines, Contains, Definite Calls, Extends, Implements          │
│   3. Unresolved References: Import specifiers & dynamic calls (for Phase 1.3 Linker)   │
│   4. Diagnostics: Syntax warnings, recovered error spans                               │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Core Rust Abstractions & Traits (`cartograph-ast`)

### 3.1 `LanguageParser` Trait

```rust
use std::path::Path;
use cartograph_core::{CartographError, Result};
use crate::model::{ParsedFile, ParseContext};

pub trait LanguageParser: Send + Sync {
    /// Unique language identifier (e.g., "python", "typescript", "ruby", "rust", "go", "sql", "markdown")
    fn language_id(&self) -> &'static str;

    /// File extensions handled by this parser (e.g., &["py", "pyi"])
    fn file_extensions(&self) -> &'static [&'static str];

    /// Special filenames handled (e.g., &["routes.rb", "Gemfile", "Dockerfile"])
    fn special_filenames(&self) -> &'static [&'static str] {
        &[]
    }

    /// Check if this parser handles a given file path
    fn can_parse(&self, filepath: &Path) -> bool {
        if let Some(filename) = filepath.file_name().and_then(|f| f.to_str()) {
            if self.special_filenames().contains(&filename) {
                return true;
            }
        }
        if let Some(ext) = filepath.extension().and_then(|e| e.to_str()) {
            self.file_extensions().contains(&ext.to_lowercase().as_str())
        } else {
            false
        }
    }

    /// Full parse from scratch
    fn parse(&self, ctx: &ParseContext) -> Result<ParsedFile>;

    /// Incremental re-parse: updates existing CST in microseconds
    fn parse_incremental(
        &self,
        ctx: &ParseContext,
        old_tree: &tree_sitter::Tree,
        edit: Option<&tree_sitter::InputEdit>,
    ) -> Result<(ParsedFile, tree_sitter::Tree)>;
}
```

### 3.2 Enhanced Data Structures

```rust
use cartograph_core::{Edge, EdgeKind, Node, NodeKind};
use serde::{Deserialize, Serialize};

/// Exact source code coordinates including byte offsets for zero-copy slicing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub start_byte: usize,
    pub end_byte: usize,
    pub start_line: u32,
    pub start_col: u32,
    pub end_line: u32,
    pub end_col: u32,
}

impl Span {
    pub fn slice<'a>(&self, source: &'a str) -> &'a str {
        &source[self.start_byte..self.end_byte]
    }
}

/// Symbol access control visibility
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Visibility {
    Public,
    Protected,
    Private,
    PackagePrivate,
}

/// Call resolution fidelity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CallFidelity {
    /// Definite call: same-file symbol, explicit import, or verified receiver type
    Definite,
    /// Dynamic candidate call: receiver invocation where type is unverified at single-file scope
    DynamicCandidate,
}

/// Unresolved external reference for Phase 1.3 Linker
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnresolvedReference {
    pub source_node_id: String,
    pub target_symbol: String,
    pub import_source: Option<String>,
    pub kind: EdgeKind,
    pub span: Span,
    pub call_fidelity: CallFidelity,
    pub receiver_type_hint: Option<String>,
    pub receiver_var: Option<String>,
    pub arg_count: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct ParseContext<'a> {
    pub filepath: &'a str,
    pub content: &'a str,
    pub options: ParseOptions,
}

#[derive(Debug, Clone, Default)]
pub struct ParseOptions {
    pub extract_docstrings: bool,
    pub extract_signatures: bool,
    pub extract_call_sites: bool,
    pub extract_inline_sql: bool,
    pub compute_node_checksums: bool,
    pub skip_generated: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ParsedFile {
    pub filepath: String,
    pub language: String,
    pub is_generated: bool,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub unresolved_references: Vec<UnresolvedReference>,
    pub diagnostics: Vec<AstDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AstDiagnostic {
    pub span: Span,
    pub message: String,
    pub severity: DiagnosticSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagnosticSeverity {
    Warning,
    Error,
}
```

---

## 4. Deep Dive on the 5 Selected Architectural Enhancements

---

### 4.1 Subtree Merkle Hashing (`node.checksum`) & Differential Updates

#### The Architecture
Instead of relying solely on file-level SHA-256 hashes, each extracted AST node computes a deterministic checksum over its normalized token stream:
```rust
pub fn compute_node_checksum(node_text: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    // Normalize whitespace while preserving token ordering
    for token in node_text.split_whitespace() {
        hasher.update(token.as_bytes());
        hasher.update(b" ");
    }
    format!("{:x}", hasher.finalize())
}
```

#### The Lifecycle & Benefits
1. **Differential SQLite Updates**:
   - In a 2,000-line file with 30 methods, modifying 1 method yields 29 matching hashes and 1 modified hash.
   - `cartograph-db` updates *only that single row* in `nodes` instead of executing a bulk drop-and-reinsert of the entire file's graph.
2. **Semantic Code Diffs for AI Review Bots**:
   - Purely cosmetic edits (re-indentation, adding trailing commas) do not alter the normalized token hash.
   - AI PR Reviewers (`cartograph ai-review`) instantly recognize which methods experienced **real semantic changes**.

---

### 4.2 Type-Assisted Call Resolution (Dynamic to Static Promotion)

#### The Problem
In dynamically typed code, calls like `user.save()` or `auth.login()` are ambiguous without runtime evaluation:

```python
# Ambiguous without type inspection
def process(user):
    user.save()  # Is this User.save, Model.save, or a mock?
```

#### The Solution
`cartograph-ast` inspects local type annotations, parameter signatures, and variable assignment expressions:

```python
# 1. Parameter type annotation
def process(user: User):
    user.save()  # Receiver type is 'User'! -> Definite Call

# 2. Local assignment RHS
def sync():
    invoice = Invoice.objects.create(...)
    invoice.calculate_tax()  # Receiver type is 'Invoice'! -> Definite Call
```

```typescript
// TypeScript interface binding
function sync(invoice: IInvoice) {
    invoice.calculateTax(); // Receiver type is 'IInvoice'! -> Definite Call
}
```

#### Classification Rule
- **`CallFidelity::Definite`**:
  - Receiver type is explicitly annotated or inferred locally.
  - Receiver is `self` / `this` (internal class method).
  - Function is standalone or directly imported (`from utils import sanitize; sanitize(x)`).
- **`CallFidelity::DynamicCandidate`**:
  - Receiver type cannot be resolved from local file scope.
  - Emits `receiver_var: Some("user")`, `arg_count: Some(1)` into `UnresolvedReference` for heuristic resolution in Phase 1.3.

---

### 4.3 Symbol Visibility & Export Boundary Detection

#### Extraction Rules Across Languages
| Language | Syntax / Convention | Extracted Visibility | Stored Metadata |
|---|---|---|---|
| **Python** | No underscore prefix (`def login`) | `Public` | `{"visibility": "public"}` |
| **Python** | Single underscore (`def _helper`) | `Private` | `{"visibility": "private"}` |
| **Python** | Double underscore (`def __mangled`) | `Private` | `{"visibility": "private", "mangled": true}` |
| **TypeScript**| `export function foo()`, `public method()`| `Public` | `{"visibility": "public", "is_export": true}` |
| **TypeScript**| `private method()`, unexported function | `Private` | `{"visibility": "private", "is_export": false}` |
| **TypeScript**| `protected method()` | `Protected` | `{"visibility": "protected"}` |
| **Rust** | `pub fn foo()` | `Public` | `{"visibility": "public"}` |
| **Rust** | `pub(crate) fn foo()` | `PackagePrivate` | `{"visibility": "package_private"}` |
| **Rust** | `fn foo()` (default) | `Private` | `{"visibility": "private"}` |
| **Ruby** | Methods under `public` | `Public` | `{"visibility": "public"}` |
| **Ruby** | Methods under `private` | `Private` | `{"visibility": "private"}` |
| **Go** | Capitalized (`func NewServer()`) | `Public` | `{"visibility": "public", "exported": true}` |
| **Go** | Lowercase (`func handle()`) | `PackagePrivate` | `{"visibility": "package_private", "exported": false}` |

#### Downstream Value
- **Architectural Boundary Guard (Phase 2.4)**: Automatically flags PRs where an external package accesses a `private` or `package_private` symbol.
- **High-Precision Dead Code Detection (Phase 2.6)**: Avoids false positives on unreferenced public exports while accurately surfacing unused private methods.

---

### 4.4 Multi-Language Embedded Code Slicing (Inline SQL)

#### The Problem
In Rails and Django, complex queries frequently use raw SQL:
```python
cursor.execute("SELECT id, total_amount FROM orders WHERE status = 'pending'")
```
If the AST parser ignores the SQL string, the relationship between this Python view and the `orders` database table is completely lost!

#### The Solution: Embedded Language Slicing
1. When the Python or Ruby visitor matches a database execution call:
   - Target functions: `execute`, `raw`, `find_by_sql`, `select_all`, `query`.
2. The visitor extracts the raw SQL string literal argument.
3. The string slice is passed to an embedded invocation of `SqlParser::parse_query_tables(sql)`.
4. `SqlParser` parses the SQL fragment and extracts referenced table names (`orders`).
5. Generates an `EdgeKind::References` edge directly from the containing Python function to the SQL table model:
   ```rust
   Edge {
       source_id: "cartograph://app/views.py::::get_pending_orders#function",
       target_id: "cartograph://db/schema.sql::::orders#model",
       kind: EdgeKind::References,
       metadata: Some(json!({ "via": "inline_sql", "query_type": "SELECT" })),
   }
   ```

---

### 4.5 Generated Code & Vendor Filtering

#### Header Inspection Mechanism
Before executing Tree-sitter parsing on a file, `AstRegistry::is_generated_content` checks the first 10 lines of the file for known generator markers:
```rust
const GENERATED_MARKERS: &[&str] = &[
    "@generated",
    "DO NOT EDIT",
    "Code generated by",
    "Automatically generated",
    "Autogenerated by",
    "This file is generated",
    "GENERATED BY PROTO-GEN",
];

pub fn is_generated(content: &str) -> bool {
    for line in content.lines().take(10) {
        let trimmed = line.trim();
        for marker in GENERATED_MARKERS {
            if trimmed.contains(marker) {
                return true;
            }
        }
    }
    false
}
```

#### Behavior & Filters
- **Extensions / Patterns Automatically Tagged**:
  - `*.pb.go`, `*_pb2.py`, `*.d.ts.map`, `*.min.js`, `*.min.css`, `bundle.js`.
- If `ParseOptions.skip_generated == true`, the file is skipped with zero AST overhead.
- If parsed, all extracted nodes are tagged with `metadata.is_generated = true`, allowing queries to filter out generated noise.

---

## 5. Declarative Tree-Sitter S-Expression (`.scm`) Query Architecture

### 5.1 Query Organization
Queries are organized cleanly under `crates/cartograph-ast/queries/<lang>/tags.scm` and embedded into the binary at compile time via `include_str!`.

```
crates/cartograph-ast/
├── queries/
│   ├── python/tags.scm         # Classes, functions, pytest, calls, docstrings
│   ├── typescript/tags.scm     # Interfaces, types, classes, hooks, JSX calls
│   ├── ruby/tags.scm           # Classes, modules, methods, calls, RSpec
│   ├── rust/tags.scm           # Structs, enums, traits, impls, doc comments
│   ├── go/tags.scm             # Structs, interfaces, receivers, imports
│   ├── sql/tags.scm            # Tables, columns, foreign keys, views
│   └── doc/tags.scm            # Markdown headings, code blocks
```

### 5.2 Pre-Compilation via `LazyLock`
```rust
use std::sync::LazyLock;
use tree_sitter::{Language, Query};

pub static PYTHON_TAGS_QUERY: LazyLock<Query> = LazyLock::new(|| {
    let language = tree_sitter_python::LANGUAGE.into();
    Query::new(&language, include_str!("../queries/python/tags.scm"))
        .expect("Failed to compile Python Tree-sitter query")
});
```

---

## 6. Performance Engineering & Memory Footprint

| Metric | Target | Rationale |
|---|---|---|
| **Python Parsing Speed** | **>35,000 LOC/s / core** | Pre-compiled S-expressions + thread-local parser |
| **TypeScript/TSX Parsing Speed** | **>30,000 LOC/s / core** | Fast TSX query cursor matching |
| **Ruby Parsing Speed** | **>35,000 LOC/s / core** | Ruby grammar AST traversal |
| **Rust / Go Parsing Speed** | **>40,000 LOC/s / core** | Pure grammar execution |
| **Incremental Re-Parse Latency**| **< 50 microseconds** | Tree-sitter subtree reuse via `old_tree` |
| **Memory Allocation per File** | **< 150 KB peak** | Direct string slicing from input buffer |
| **Subtree Hash Overhead** | **< 5% parse time** | Fast streaming hash over token slice |

---

## 7. Phased Implementation Strategy & Task Breakdown

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│ Milestone 1.2.1: AST Engine Core Framework & Abstractions                              │
│ • LanguageParser trait with incremental re-parse support.                              │
│ • Span, Visibility, CallFidelity, UnresolvedReference, and ParsedFile models.         │
│ • AstRegistry with extension, special filename, shebang, and generated code filter.    │
│ • LineIndex for sub-microsecond coordinate resolution.                                 │
├────────────────────────────────────────────────────────────────────────────────────────┤
│ Milestone 1.2.2: Declarative Query Engine, Hashing & Trivia Binder                     │
│ • queries/<lang>/tags.scm structure with compile-time include_str! and LazyLock.       │
│ • Query matching harness and standardized capture mapper.                              │
│ • Subtree Merkle Hashing (node.checksum computation).                                  │
│ • TriviaBinder docstring association & SignatureNormalizer.                            │
├────────────────────────────────────────────────────────────────────────────────────────┤
│ Milestone 1.2.3: Python / Django / FastAPI / Pytest Extractor                          │
│ • Python tags.scm + procedural Django URL and model field visitor.                     │
│ • Type-assisted call resolution (Python type hints).                                   │
│ • Visibility tagging (_private vs public).                                             │
│ • Inline SQL query slicing (tracing raw SQL to tables).                                │
│ • Celery tasks, DRF serializers, and Pytest coverage extraction.                       │
├────────────────────────────────────────────────────────────────────────────────────────┤
│ Milestone 1.2.4: TypeScript / JavaScript / React Extractor                             │
│ • Separate TS and TSX grammar paths with export/visibility detection.                  │
│ • React Component heuristics (JSX return analysis), Hooks, and JSX call trees.         │
│ • Type-assisted call resolution via TS interfaces and parameter types.                 │
├────────────────────────────────────────────────────────────────────────────────────────┤
│ Milestone 1.2.5: Ruby / Ruby on Rails / RSpec Extractor                                │
│ • Ruby tags.scm + ActiveRecord association & validation macro extractors.              │
│ • ActionControllers, routes.rb REST DSL parsing, and RSpec spec extraction.            │
│ • Inline SQL slicing (find_by_sql, execute).                                           │
│ • Visibility tracking (private/protected blocks).                                      │
├────────────────────────────────────────────────────────────────────────────────────────┤
│ Milestone 1.2.6: Rust, Go, SQL & Markdown Extractors                                   │
│ • Rust traits/impls, visibility (pub, pub(crate)), Go receivers and exported casing.   │
│ • SQL DDL tables, foreign keys, and views.                                             │
│ • Markdown structural headings and cross-document link resolution.                     │
├────────────────────────────────────────────────────────────────────────────────────────┤
│ Milestone 1.2.7: Integration, Benchmark Suite & Zero-Panic Verification                 │
│ • Polyglot test suite across real open-source fixtures.                                │
│ • Incremental re-parse benchmark (<50µs) and throughput validation (>30,000 LOC/s).     │
│ • Zero-panic verification against syntax-errored and generated test cases.             │
└────────────────────────────────────────────────────────────────────────────────────────┘
```
