# 🗺️ Cartograph Phase 1.4: 1-Turn Subgraph Localization (`cartograph locate`)
## Comprehensive Technical Design & Architecture Specification (v1.0 Final)

---

## 1. Executive Summary & Architectural Vision

The primary mission of **Phase 1.4 (`cartograph locate`)** is to deliver an ultra-high-performance (**sub-3ms**), 1-turn hybrid AST code search, bi-directional subgraph neighborhood traversal, and context localization engine for developers and AI pair-programming agents.

In modern software engineering and AI-assisted workflows, **context discovery is the number one source of token waste, latency, and reasoning failure:**
- **The Text Grep Problem**: Standard search tools (`grep`, `ripgrep`, text FTS) return flat, disconnected snippets stripped of all structural meaning. An AI agent searching for a feature or symbol receives dozens of matching lines with no knowledge of what functions call them, what classes implement them, what HTTP routes map to them, or what tests verify them.
- **The Multi-Turn Token Burn**: To assemble sufficient context to edit or debug code, an autonomous agent (Claude, Cursor, Antigravity, or an internal PR bot) must execute 10 to 20 consecutive file-read turns, spending minutes and burning 50,000 to 100,000 tokens just to reconstruct the local dependency graph.
- **The Parameter Type-Contract Blindness**: Even when an agent finds the target function, it still lacks the schemas of the parameter structs and return types, forcing additional exploratory turns.

**Cartograph Locate completely eliminates this multi-turn context loop in a single turn:**
Given a human feature query (`"accrual period update"`), a qualified symbol (`AuthService::verify_token`), an inline directive query (`"verify_token kind:fn path:src/auth"`), or a canonical URI, `cartograph locate`:
1. Classifies query intent and executes a **Centrality- & Git-Recency-Aware Hybrid Search** combining exact symbol lookups, scoped qualifier indexing, SQLite FTS5 trigram BM25 matching, and graph topological in-degree weighting.
2. Expands the top hit into an **N-Hop Bi-Directional Subgraph** containing direct callers, callees, route handlers (`MapsTo`), and unit/integration tests (`Tests`), with optional **End-to-End Call Chain Tracing (`--flow`)**.
3. Performs **Type-Contract Inlining (`--resolve-types`)**, automatically pulling parameter and return type schemas into the localized subgraph so the agent gets the complete data contract in one shot.
4. Filters edges by **Static vs. Dynamic Confidence (`--min-confidence 0.8`)**, ensuring agents are not misled by ambiguous dynamic candidate calls.
5. Extracts **Syntax-Padded Code Slices** across configurable **Context Density Tiers** (`--detail minimal | signatures | full`), governed by a strict **AI Token Budget Manager** (`--max-tokens`) that prevents context window overflow.
6. Renders the unified context into **ANSI Colorized Terminal Trees**, **Interactive Terminal TUI**, **GitHub-Flavored Markdown with Mermaid Flowcharts**, **MCP-Ready JSON**, or **Agent XML Prompt Blocks**.

---

### Core Architectural Pillars

1. **Multi-Modal Query Intent Classification & Inline Directives (`query_parser.rs`)**:
   - Parses diverse query structures:
     - **Canonical URIs**: `cartograph://crates/engine/src/linker.rs::::UniversalLinker#struct` ($O(1)$ direct ID lookup).
     - **Qualified Identifiers**: `OrderService::create`, `AuthService.verify_token`, `User#save`, `models.Order.status`.
     - **Exact Symbol Names**: `RateCard`, `verify_jwt`, `handle_webhook`.
     - **Filepath / Directory Scopes**: `src/billing/*.rs`, `crates/engine/`.
     - **Natural Language / Ticket Feature Queries**: `"accrual period calculation"`, `"payment webhook failure"`.
   - **Inline Query Directives (Power-User & Agent Syntax)**:
     Embeds filters directly in the search string without multiple CLI flags:
     ```bash
     cartograph locate "verify_token kind:function path:src/auth lang:rust min-conf:0.8"
     ```
   - CLI Directives: `--kind <kind>`, `--lang <lang>`, `--path <dir>`, `--depth <1..5>`, `--direction <both|downstream|upstream>`, `--edges <calls,mapsto,tests>`, `--flow`, `--resolve-types`, `--code`, `--context <lines>`, `--detail <minimal|signatures|full>`, `--max-tokens <N>`, `--format <terminal|markdown|json|agent-xml>`, `-i, --interactive`.

2. **Sub-3ms Centrality- & Churn-Aware Hybrid Search (`hybrid_search.rs`)**:
   - **Multi-Channel Candidate Generation**:
     - *Channel 1 (Exact Match)*: Direct lookup against `nodes(name)` and `nodes(id)` ($1.0$ confidence boost).
     - *Channel 2 (Scoped Qualifier Match)*: Matches `name = ? AND scope LIKE ?` ($0.9$ confidence boost).
     - *Channel 3 (SQLite FTS5 Trigram Search)*: Trigram full-text search with weighted column BM25 ranking (`name: 5.0x`, `scope: 3.0x`, `signature: 2.0x`, `docstring: 1.5x`, `filepath: 1.0x`).
     - *Channel 4 (Substring / Prefix Fuzzy Fallback)*: Substring and token prefix matching for short (<3 char) queries.
   - **Topological Centrality Weighting**:
     - Architectural landmarks (functions called by 30 other modules) are boosted over obscure private helpers using degree centrality:
       $$S_{\text{centrality}} = \ln(1 + d_{\text{in}} + 0.5 \cdot d_{\text{out}})$$
   - **Git Recency & Churn Boost (Bug Localization Accelerator)**:
     - In git repositories, recently modified or churn-heavy symbols receive a recency boost $B_{\text{churn}}$, making bug and regression localization significantly faster.
   - **Reciprocal Rank Fusion (RRF)**:
     - Fuses disparate channels into a calibrated, normalized ranking score without heuristic scale mismatches:
       $$S_{\text{final}} = \alpha \cdot \text{Norm}(S_{\text{text}}) + \beta \cdot \text{Norm}(S_{\text{centrality}}) + \gamma \cdot \text{Norm}(B_{\text{churn}})$$

3. **Bi-Directional Subgraphs, Call Chains & Type-Contract Inlining (`subgraph.rs`)**:
   - **Radial Subgraph Expansion**: Explores callers (upstream) and callees (downstream) up to depth $N$ (default: 1, range: 0..5).
   - **Call-Chain / Flow Mode (`--flow`)**: Traces linear execution flow paths (`Route -> Controller -> Service -> Repository -> Model`) instead of expanding a spherical ball of neighbors.
   - **Type-Contract Inlining (`--resolve-types`)**:
     - Automatically parses parameter and return types (e.g. `order: Order, ctx: &SecurityContext -> Result<Receipt>`) and pulls the struct/interface schemas of `Order`, `SecurityContext`, and `Receipt` directly into the subgraph!
   - **Static vs. Dynamic Edge Confidence (`--min-confidence`)**:
     - Evaluates edge confidence scores (`edges.confidence REAL DEFAULT 1.0`). Filters out uncertain dynamic candidate calls when high precision is demanded.
   - **Cross-Domain Specialized Edges**: Backend HTTP endpoints mapped via `MapsTo`, and unit/integration tests connected via `Tests`.
   - **Full Structural Edge Retention**: Returns both nodes AND connecting edges (`LocalizedSubgraph { root_nodes, neighbor_nodes, edges }`), enabling graph visualization and topological dependency analysis.
   - **Disambiguation Clustering**: When a query matches multiple strong candidate clusters across different subsystems (e.g. `web/` vs `server/`), groups results by subsystem cluster.
   - **Fanout Guard & Cycle Prevention**: BFS visited set prevents circular recursion; fanout ceiling (`max_fanout = 15` per node) prevents runaway context explosion on massive dispatchers.

4. **AST Code Slicing & AI Token Budgeting (`slicer.rs` & `budget.rs`)**:
   - Zero-copy line slicing using Tree-sitter AST `Span` (`start_line..end_line`, `start_col..end_col`).
   - Configurable context padding (`--context 3` lines before and after definition).
   - **Context Density Tiers (`--detail`)**:
     - `minimal`: Topology only (node names, kinds, file:lines, edge connections) — ~150 tokens.
     - `signatures`: AST signatures, parameter contracts, docstrings, and graph topology — ~500 tokens.
     - `full` (default with `--code`): Full syntax-padded code bodies bounded by `--max-tokens`.
   - **Hierarchical Token Budget Degradation (`TokenBudgetManager`)**:
     - Enforces hard token ceilings (`--max-tokens 4000`) for LLM context windows.
     - Priority tiers: (1) Root signatures & docstrings -> (2) Root source code -> (3) Inlined type contracts -> (4) 1-hop neighbor signatures -> (5) 1-hop neighbor code bodies -> (6) 2-hop signatures -> (7) Test signatures.
     - Emits non-destructive budget truncation indicators (`[+N neighbor code bodies omitted to fit 4,000 token budget]`).

5. **Quad-Engine Presentation Layer (`formatters/`)**:
   - **Terminal (`terminal.rs`)**:
     - Rich ANSI colorized interface using `colored`.
     - Symbol kind badges (`[FUNCTION]`, `[STRUCT]`, `[ROUTE]`), clickable file:line links, ASCII dependency trees showing incoming/outgoing calls, confidence indicators, and syntax-highlighted code blocks.
   - **Interactive Terminal TUI (`-i, --interactive`)**:
     - Live fuzzy search and split-pane subgraph and code preview in the developer's terminal.
   - **Markdown with Mermaid (`markdown.rs`)**:
     - Formatted for direct injection into LLM prompts and PR reviews.
     - Automatically generates Mermaid.js flowcharts (`graph LR`) depicting the localized subgraph.
     - Fenced code blocks with language syntax specifiers (```rust, ```python, ```typescript).
   - **MCP-Ready JSON (`json.rs`)**:
     - Strongly typed machine-readable JSON schema.
     - 100% compatible with the upcoming Phase 4.1 Model Context Protocol (MCP) server tool specifications.
   - **Agent XML Prompt Preset (`agent_xml.rs` via `--format agent-xml`)**:
     - Structured XML tags (`<code_context>`, `<root_symbol>`, `<type_contracts>`, `<dependencies>`) tailored for Anthropic Claude, OpenAI, and Cursor system prompt injection.

6. **Storage Backend Enhancements & Schema Upgrades (`cartograph-db`)**:
   - Adds `confidence REAL DEFAULT 1.0` and `weight REAL DEFAULT 1.0` columns to the `edges` table.
   - Extends `GraphStoreBackend` with:
     - `get_edges_for_nodes(&self, node_ids: &[String], min_confidence: f64) -> Result<Vec<Edge>>`.
     - `get_flow_chain(&self, start_id: &str, max_depth: usize) -> Result<Vec<Edge>>`.
   - Utilizes SQLite indexes `idx_edges_source` and `idx_edges_target` to resolve all connecting edges in **<0.2ms**.
   - Read-only connection pools ensure `cartograph locate` never blocks on background indexing or real-time watcher file writes.

7. **Project Configuration Management (`cartograph.toml`)**:
   - First-class repository configuration file supporting defaults for `[locate]`, `[indexer]`, and `[watcher]`.

---

## 2. Pipeline Architecture & Component Topology

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                              Developer & AI Agent Query Input                          │
│   cartograph locate "AuthService::verify_token kind:fn" --resolve-types --max-tokens 4k │
└───────────────────────────────────────────┬────────────────────────────────────────────┘
                                            │
                                            ▼
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                        1. Query Parser & Intent Classifier                             │
│       • Disambiguates: Canonical URI vs Qualified vs Exact vs Natural Language         │
│       • Extracts inline directives: kind:fn path:src/ lang:rs min-conf:0.8             │
└───────────────────────────────────────────┬────────────────────────────────────────────┘
                                            │
                                            ▼
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                     2. Centrality- & Churn-Aware Hybrid Search Engine                  │
│  ┌───────────────────────┐ ┌──────────────────────┐ ┌────────────────────────────────┐ │
│  │ Channel 1: Exact Name │ │ Channel 2: Qualified │ │ Channel 3: FTS5 Trigram BM25   │ │
│  │  Score: 1.0           │ │  Score: 0.9          │ │  Weighted: name, scope, sig... │ │
│  └───────────┬───────────┘ └──────────┬───────────┘ └────────────────┬───────────────┘ │
│              └────────────────────────┼──────────────────────────────┘                 │
│                                       ▼                                                │
│                 Reciprocal Rank Fusion (RRF) & Multi-Channel Merge                     │
│                                       │                                                │
│                                       ▼                                                │
│        Topological Centrality: log(1 + in_degree + 0.5 * out_degree)                   │
│        + Git Recency / Churn Boost: recent_churn_score                                 │
│                                       │                                                │
│                                       ▼                                                │
│            Disambiguation Clustering & Top-K Root Node Selection                       │
└───────────────────────────────────────┬────────────────────────────────────────────────┘
                                        │
                                        ▼
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                   3. Subgraph Neighborhood, Flow & Type Resolution                     │
│                                                                                        │
│             ┌───────────────────┐               ┌───────────────────┐                  │
│             │ Upstream Callers  │ ◄──────────── │     ROOT NODE     │                  │
│             │ & Dependents      │   (Incoming)  │  (Top Search Hit) │                  │
│             └───────────────────┘               └─────────┬─────────┘                  │
│                                                           │                            │
│                       ┌───────────────────────────────────┼────────────────────┐       │
│                       ▼ (Outgoing)                        ▼                    ▼       │
│             ┌───────────────────┐               ┌───────────────────┐ ┌──────────────┐ │
│             │ Downstream Callees│               │ Route Handlers    │ │ Unit / Integ │ │
│             │ & Dependencies    │               │ (MapsTo)          │ │ Tests (Tests)│ │
│             └─────────┬─────────┘               └───────────────────┘ └──────────────┘ │
│                       │                                                                │
│                       ▼ (Type Inlining)                                                │
│             ┌───────────────────┐                                                      │
│             │ Parameter/Return  │                                                      │
│             │ Type Contracts    │                                                      │
│             └───────────────────┘                                                      │
│                                                                                        │
│  • Radial BFS or Linear --flow Chain   • --resolve-types   • Edge Confidence Filtering │
└───────────────────────────────────────┬────────────────────────────────────────────────┘
                                        │
                                        ▼
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                   4. AST Code Slicing & AI Token Budget Manager                        │
│       • Zero-Copy Source Line Extraction (start_line..end_line + context padding)      │
│       • Context Density Tiers: --detail minimal | signatures | full                    │
│       • Priority-Tier Budget Allocation (Root Sig -> Root Code -> Type Contracts...)   │
│       • Strict Token Ceiling Enforcement (<max_tokens) with Truncation Indicators      │
└───────────────────────────────────────┬────────────────────────────────────────────────┘
                                        │
                                        ▼
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                        5. Quad-Engine Presentation Layer                               │
│                                                                                        │
│  ┌────────────────────────┐ ┌────────────────────────┐ ┌────────────────────────┐      │
│  │   Terminal Formatter   │ │   Markdown Formatter   │ │     JSON Formatter     │      │
│  │ • ANSI Badges & Tree   │ │ • GitHub-Flavored MD   │ │ • Strongly Typed Schema│      │
│  │ • Interactive TUI (-i) │ │ • Mermaid graph LR     │ │ • MCP Tool Compatible  │      │
│  └────────────────────────┘ └────────────────────────┘ └────────────────────────┘      │
│                                       │                                                │
│                                       ▼                                                │
│                          ┌────────────────────────┐                                    │
│                          │    Agent XML Prompt    │                                    │
│                          │ • Claude/OpenAI Ready  │                                    │
│                          │ • Structured XML Tags  │                                    │
│                          └────────────────────────┘                                    │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Core Rust Abstractions & Data Structures (`cartograph-engine::locate`)

### 3.1 Query Configuration, Directives & Intent Classification (`query.rs`)

```rust
use std::path::PathBuf;
use cartograph_core::{NodeKind, EdgeKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryIntent {
    /// Exact canonical URI lookup: cartograph://path::scope::name#kind
    CanonicalUri(String),
    /// Qualified symbol: scope + name (e.g. AuthService::verify_token, Order.save)
    QualifiedSymbol { scope: String, name: String },
    /// Exact unqualified symbol identifier: RateCard, calculate_tax
    ExactSymbol(String),
    /// Scoped path prefix: src/billing/*.rs, crates/engine/
    PathFilter(String),
    /// Natural language feature or ticket query: "accrual period update"
    SemanticIntent(Vec<String>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraversalMode {
    /// Radial BFS neighborhood in all directions
    Radial,
    /// End-to-end linear execution flow path: Route -> Controller -> Service -> Model
    FlowChain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraversalDirection {
    Both,
    DownstreamOnly,
    UpstreamOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextDetail {
    /// Topology only: node identifiers, kinds, file:line, and edges (~150 tokens)
    Minimal,
    /// Signatures only: AST signatures, parameter contracts, docstrings, and topology (~500 tokens)
    Signatures,
    /// Full: Syntax-padded code slices bounded by token budget
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Terminal,
    InteractiveTui,
    Markdown,
    Json,
    AgentXml,
}

#[derive(Debug, Clone)]
pub struct LocateQuery {
    pub raw_query: String,
    pub clean_query: String,
    pub intent: QueryIntent,
    pub kind_filter: Option<Vec<NodeKind>>,
    pub path_filter: Option<String>,
    pub language_filter: Option<String>,
    pub depth: usize,
    pub mode: TraversalMode,
    pub direction: TraversalDirection,
    pub edge_kinds: Option<Vec<EdgeKind>>,
    pub min_confidence: f64,
    pub resolve_types: bool,
    pub detail: ContextDetail,
    pub include_code: bool,
    pub context_lines: usize,
    pub max_tokens: Option<usize>,
    pub limit: usize,
    pub format: OutputFormat,
}

impl Default for LocateQuery {
    fn default() -> Self {
        Self {
            raw_query: String::new(),
            clean_query: String::new(),
            intent: QueryIntent::ExactSymbol(String::new()),
            kind_filter: None,
            path_filter: None,
            language_filter: None,
            depth: 1,
            mode: TraversalMode::Radial,
            direction: TraversalDirection::Both,
            edge_kinds: None,
            min_confidence: 0.0,
            resolve_types: false,
            detail: ContextDetail::Signatures,
            include_code: false,
            context_lines: 3,
            max_tokens: None,
            limit: 5,
            format: OutputFormat::Terminal,
        }
    }
}

pub struct QueryParser;

impl QueryParser {
    /// Parse raw query string and CLI parameters into a validated LocateQuery,
    /// extracting inline directives (e.g. `kind:fn path:src/ min-conf:0.8`)
    pub fn parse(raw_query: &str, cli_options: LocateCliOptions) -> LocateQuery;

    /// Disambiguate intent: URI vs Qualified vs Exact vs Natural Language
    pub fn classify_intent(query: &str) -> QueryIntent;

    /// Extract inline directives and return clean query term
    pub fn extract_inline_directives(input: &str) -> (String, InlineDirectives);

    /// Tokenize and split compound identifiers (CamelCase and snake_case)
    pub fn tokenize_terms(query: &str) -> Vec<String>;
}

#[derive(Debug, Default, Clone)]
pub struct InlineDirectives {
    pub kind: Option<String>,
    pub path: Option<String>,
    pub lang: Option<String>,
    pub min_confidence: Option<f64>,
    pub depth: Option<usize>,
}
```

---

### 3.2 Centrality- & Churn-Aware Hybrid Search Engine (`search.rs`)

```rust
use cartograph_core::{Node, SearchResult, GraphStoreBackend};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct ScoredNode {
    pub node: Node,
    pub score: f64,
    pub text_score: f64,
    pub centrality_score: f64,
    pub churn_score: f64,
    pub in_degree: usize,
    pub out_degree: usize,
    pub matched_channel: SearchChannel,
    pub matched_field: Option<String>,
    pub cluster_domain: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchChannel {
    CanonicalUri,
    ExactSymbol,
    QualifiedSymbol,
    Fts5Trigram,
    SubstringFallback,
}

pub struct HybridSearchEngine {
    backend: Arc<dyn GraphStoreBackend>,
    alpha: f64, // Weight for text relevance (default: 0.70)
    beta: f64,  // Weight for structural centrality (default: 0.20)
    gamma: f64, // Weight for git recency / churn (default: 0.10)
}

impl HybridSearchEngine {
    pub fn new(backend: Arc<dyn GraphStoreBackend>) -> Self {
        Self {
            backend,
            alpha: 0.70,
            beta: 0.20,
            gamma: 0.10,
        }
    }

    /// Execute multi-channel search and return top-K ranked root nodes
    pub async fn search_roots(&self, query: &LocateQuery) -> cartograph_core::Result<Vec<ScoredNode>>;

    /// Group candidate nodes into distinct domain clusters (e.g. "web/api", "core/engine")
    pub fn cluster_candidates(&self, candidates: Vec<ScoredNode>) -> Vec<CandidateCluster>;
}

#[derive(Debug, Clone)]
pub struct CandidateCluster {
    pub domain: String,
    pub nodes: Vec<ScoredNode>,
}
```

---

### 3.3 Subgraph Neighborhood, Flow & Type Resolution (`subgraph.rs`)

```rust
use cartograph_core::{Edge, EdgeKind, Node};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeDirection {
    Outgoing,    // Downstream (Calls, Imports, Extends, References)
    Incoming,    // Upstream (CalledBy, ImportedBy, ExtendedBy)
    CrossDomain, // Route mappings (MapsTo) or Test links (Tests)
    TypeContract,// Parameter or return type definition (TypeContract)
}

#[derive(Debug, Clone)]
pub struct LocalizedEdge {
    pub source_id: String,
    pub target_id: String,
    pub kind: EdgeKind,
    pub direction: EdgeDirection,
    pub confidence: f64,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct LocalizedSubgraph {
    pub root_nodes: Vec<ScoredNode>,
    pub neighbor_nodes: HashMap<String, Node>,
    pub inlined_type_contracts: HashMap<String, Node>,
    pub edges: Vec<LocalizedEdge>,
    pub depth: usize,
    pub total_nodes: usize,
    pub total_edges: usize,
}

pub struct SubgraphTraverser {
    max_fanout_per_node: usize, // Default: 15
}

impl SubgraphTraverser {
    pub fn new(max_fanout: usize) -> Self {
        Self { max_fanout_per_node: max_fanout }
    }

    /// Traverses the bi-directional neighborhood or execution flow for the given root nodes
    pub async fn extract_subgraph(
        &self,
        backend: &dyn cartograph_core::GraphStoreBackend,
        roots: Vec<ScoredNode>,
        query: &LocateQuery,
    ) -> cartograph_core::Result<LocalizedSubgraph>;

    /// Inlines parameter and return type definition schemas referenced in root signatures
    pub async fn resolve_type_contracts(
        &self,
        backend: &dyn cartograph_core::GraphStoreBackend,
        subgraph: &mut LocalizedSubgraph,
    ) -> cartograph_core::Result<()>;
}
```

---

### 3.4 AST Code Slicing & AI Token Budgeting (`slicer.rs` & `budget.rs`)

```rust
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct CodeSlice {
    pub node_id: String,
    pub filepath: String,
    pub start_line: u32,
    pub end_line: u32,
    pub context_start_line: u32,
    pub context_end_line: u32,
    pub content: String,
    pub estimated_tokens: usize,
    pub is_truncated: bool,
}

pub struct CodeSlicer {
    repo_root: PathBuf,
    context_lines: usize,
    file_cache: HashMap<String, Vec<String>>,
}

impl CodeSlicer {
    pub fn new(repo_root: PathBuf, context_lines: usize) -> Self;

    /// Extract code slice for a single AST node including context padding lines
    pub fn slice_node(&mut self, node: &cartograph_core::Node) -> std::io::Result<CodeSlice>;

    /// Extract signature, docstring, and type annotations only without method body
    pub fn slice_signature_only(node: &cartograph_core::Node) -> CodeSlice;
}

#[derive(Debug, Clone)]
pub struct BudgetedSubgraphContext {
    pub root_slices: Vec<CodeSlice>,
    pub type_contract_slices: Vec<CodeSlice>,
    pub neighbor_slices: Vec<CodeSlice>,
    pub total_estimated_tokens: usize,
    pub budget_ceiling: Option<usize>,
    pub omitted_neighbor_bodies_count: usize,
}

pub struct TokenBudgetManager;

impl TokenBudgetManager {
    /// Heuristic estimation: ~3.8 characters per token for source code
    pub fn estimate_tokens(text: &str) -> usize {
        (text.len() as f64 / 3.8).ceil() as usize
    }

    /// Hierarchically fits subgraph code slices into the requested token budget
    pub fn fit_to_budget(
        root_slices: Vec<CodeSlice>,
        type_contract_slices: Vec<CodeSlice>,
        neighbor_slices: Vec<CodeSlice>,
        max_tokens: Option<usize>,
    ) -> BudgetedSubgraphContext;
}
```

---

### 3.5 Quad-Engine Presentation Layer (`formatters/`)

```rust
pub trait OutputFormatter: Send + Sync {
    /// Render localized subgraph and code slices into formatted string
    fn format(
        &self,
        subgraph: &LocalizedSubgraph,
        context: Option<&BudgetedSubgraphContext>,
    ) -> String;
}

pub struct TerminalFormatter {
    pub colorize: bool,
}

pub struct MarkdownFormatter {
    pub include_mermaid: bool,
}

pub struct JsonFormatter {
    pub pretty: bool,
}

pub struct AgentXmlFormatter {
    pub include_system_prompt_tags: bool,
}
```

---

## 4. Deep-Dive Design Sections

### 4.1 Multi-Modal Query Intent Classification & Inline Directives

The `QueryParser` combines natural language intent recognition with GitHub/ripgrep-style inline filter parsing:

```bash
cartograph locate "verify_token kind:function path:src/auth lang:rust min-conf:0.8"
```

1. **Inline Filter Extraction**:
   Regex pattern `(?i)\b(kind|path|lang|min-conf|depth):(\S+)` extracts directives into an `InlineDirectives` struct and yields the sanitized search term `"verify_token"`.
2. **Intent Cascade**:
   - `cartograph://...` $\to$ `CanonicalUri` ($O(1)$ database direct hit).
   - Contains `::`, `.`, or `#` $\to$ `QualifiedSymbol { scope, name }`.
   - Single identifier $\to$ `ExactSymbol`.
   - Contains `/` or `*` $\to$ `PathFilter`.
   - Multi-word text $\to$ `SemanticIntent`.
3. **Compound Identifier Splitting**:
   Splits CamelCase and snake_case so queries like `"accrual period"` match `calculate_accrual_period()`.

---

### 4.2 Centrality-, Churn- & Git-Recency-Aware Hybrid Ranking

Standard text search engines are blind to software architecture and project momentum. Cartograph balances three complementary signals:

#### 1. Multi-Channel Lexical Relevance ($S_{\text{text}}$)
- **Channel 1 (Exact Symbol Match)**: Direct match on `nodes(name)`. Base score: $100.0$.
- **Channel 2 (Scoped Qualifier Match)**: Matches both `scope` and `name`. Base score: $85.0$.
- **Channel 3 (SQLite FTS5 BM25 Trigram Search)**:
  Weighted column score: `name (5.0x)`, `scope (3.0x)`, `signature (2.0x)`, `docstring (1.5x)`, `filepath (1.0x)`.
- **Channel 4 (Substring Fallback)**: For queries $<3$ characters or when FTS yields zero hits.

#### 2. Topological Centrality Bonus ($S_{\text{centrality}}$)
Foundational architectural abstractions have high connectivity:
$$S_{\text{centrality}} = \ln(1 + d_{\text{in}} + 0.5 \cdot d_{\text{out}})$$

#### 3. Git Recency & Churn Velocity ($B_{\text{churn}}$)
When searching during active debugging or incident triage, recently touched code is 5x more likely to be relevant:
$$B_{\text{churn}} = \frac{1}{1 + \text{days\_since\_last\_commit}} \cdot \ln(1 + \text{commit\_count\_last\_30d})$$

#### 4. Combined Score
$$S_{\text{final}} = \alpha \cdot \text{Norm}(S_{\text{text}}) + \beta \cdot \text{Norm}(S_{\text{centrality}}) + \gamma \cdot \text{Norm}(B_{\text{churn}})$$
*(Default configuration: $\alpha = 0.70$, $\beta = 0.20$, $\gamma = 0.10$)*.

#### 5. Multi-Root Subsystem Disambiguation Clustering
If a query like `"render"` matches across multiple distinct domains, `cluster_candidates()` groups hits by root directory:
- Cluster 1: `frontend/ui/` (`Button::render`, `Modal::render`)
- Cluster 2: `backend/template/` (`TemplateEngine::render`)
- Cluster 3: `cli/` (`TerminalRenderer::render`)

---

### 4.3 Subgraph Neighborhood, Flow Chains & Type-Contract Inlining

#### 1. Radial vs. End-to-End Flow Chain (`--flow`)
- **Radial Mode (Default)**: BFS expanding outgoing dependencies and incoming callers in a sphere up to depth $N$.
- **Flow Chain Mode (`--flow`)**: Traces linear execution pipelines from entrypoint to persistence:
  ```
  POST /api/v1/checkout ──► CheckoutController::pay ──► PaymentService::charge ──► StripeClient::call ──► OrderModel
  ```

#### 2. Type-Contract Inlining (`--resolve-types`)
When an AI agent examines:
```rust
pub fn process_order(order: &Order, gateway: &PaymentGateway) -> Result<PaymentReceipt>
```
Without type resolution, the agent must spend 3 extra turns querying `Order`, `PaymentGateway`, and `PaymentReceipt`.
With `--resolve-types`, Cartograph:
1. Parses parameter type names and return types from `Node.signature`.
2. Queries the symbol table for their definitions.
3. Automatically inlines their struct/interface field definitions into the localized subgraph payload under `inlined_type_contracts`!

#### 3. Edge Confidence Filtering (`--min-confidence 0.8`)
- Static calls (compiled, typed, or explicit imports) have `confidence = 1.0`.
- Dynamic candidates (untyped JS/Python receiver calls) have `confidence = 0.5..0.7`.
- Filtering by `--min-confidence` guarantees agents receive noise-free architectural graphs.

---

### 4.4 AST Code Slicing, Context Tiers & Token Budgeting

#### Context Density Tiers (`--detail`)
| Tier | Description | Target Token Range | Ideal Consumer |
| :--- | :--- | :--- | :--- |
| **`minimal`** | Node IDs, kinds, file:lines, directed edges | ~100–250 tokens | High-level architectural map |
| **`signatures`** | AST signatures, docstrings, inlined types, graph tree | ~300–800 tokens | Interface contract awareness |
| **`full`** | Syntax-padded source code slices + signatures | ~1,500–5,000 tokens | Detailed code generation / debugging |

#### Hierarchical Degradation under `--max-tokens`
```
Requested Budget: --max-tokens 3000

Priority Degradation Tiers:
┌────────────────────────────────────────────────────────┐
│ Tier 1: Root Node Signatures & Docstrings              │  100% Guaranteed
├────────────────────────────────────────────────────────┤
│ Tier 2: Root Node Full Source Code Body                │  100% Guaranteed
├────────────────────────────────────────────────────────┤
│ Tier 3: Inlined Type Contracts (Param/Return Structs)  │  100% Guaranteed
├────────────────────────────────────────────────────────┤
│ Tier 4: 1-Hop Neighbor Signatures & Docstrings         │  100% Guaranteed
├────────────────────────────────────────────────────────┤
│ Tier 5: 1-Hop Neighbor Code Bodies                     │  Adaptive (Dropped if Budget Tight)
├────────────────────────────────────────────────────────┤
│ Tier 6: 2-Hop Neighbor & Test Function Signatures      │  Signature Only
└────────────────────────────────────────────────────────┘
```
If budget is exhausted during Tier 5, neighbor bodies are omitted, but their signatures, line spans, and graph edges are 100% retained with clear metadata:
`[NOTE: 3 neighbor function bodies omitted to satisfy --max-tokens 3000 budget. Graph topology and signatures fully preserved.]`

---

### 4.5 Quad-Engine Presentation Layer

#### 1. Colorized Terminal Mode (`--format terminal`)
- ANSI color badges (`[FUNCTION]`, `[STRUCT]`, `[ROUTE]`, `[TEST]`).
- Clickable file:line links (`src/billing/service.rs:42:10`).
- ASCII tree branch visualization:
  ```
  ┌─ [ROOT] pub fn process_payment(order: &Order) -> Result<PaymentReceipt>
  │  📍 src/billing/service.rs:42-88 (47 lines)
  │  📖 Processes credit card transaction and persists payment receipt.
  │
  ├──► Inlined Type Contracts:
  │    ├── [STRUCT] Order (src/models/order.rs:10) { pub id: Uuid, pub total: Decimal, ... }
  │    └── [STRUCT] PaymentReceipt (src/models/receipt.rs:5) { pub txn_id: String, ... }
  │
  ├──► Calls (Downstream) [conf: 1.0]:
  │    ├── [METHOD] CardGateway::charge (src/gateways/stripe.rs:105)
  │    └── [FUNCTION] audit_log (src/audit/mod.rs:18)
  │
  ◄─── Called By (Upstream) [conf: 1.0]:
  │    └── [ROUTE] POST /api/v1/checkout/pay (src/api/routes.rs:65)
  │
  🧪── Tested By:
       └── [TEST] test_process_payment_insufficient_funds (tests/test_billing.rs:210)
  ```

#### 2. Interactive Terminal TUI (`cartograph locate -i`)
- Instant fuzzy query input with live update.
- Left pane: Ranked symbol hit list with domain clustering.
- Right top pane: Localized subgraph tree and connected edges.
- Right bottom pane: Syntax-highlighted source code preview with context padding.

#### 3. Markdown with Mermaid Mode (`--format markdown`)
- Auto-generates embedded Mermaid `graph LR` flowcharts:
  ```mermaid
  graph LR
    R["POST /api/v1/checkout/pay"] -->|MapsTo| ROOT["process_payment()"]
    ROOT -->|Calls| S["CardGateway::charge()"]
    ROOT -->|Calls| A["audit_log()"]
    ROOT -->|TypeContract| O["struct Order"]
    T["test_process_payment"] -->|Tests| ROOT
  ```

#### 4. MCP-Ready JSON Mode (`--format json`)
- Strongly typed JSON payload aligned with Model Context Protocol (MCP) tool response schemas.

#### 5. Agent XML Prompt Mode (`--format agent-xml`)
- Designed for Anthropic Claude, OpenAI, and Cursor system prompt injection:
  ```xml
  <code_context query="process_payment">
    <root_symbol id="cartograph://src/billing/service.rs::::process_payment#function" file="src/billing/service.rs" lines="42-88">
      <signature>pub fn process_payment(order: &Order) -> Result<PaymentReceipt></signature>
      <docstring>Processes credit card transaction and persists payment receipt.</docstring>
      <code>...</code>
    </root_symbol>
    <type_contracts>
      <type name="Order" file="src/models/order.rs">...</type>
    </type_contracts>
    <callers>
      <caller id="..." kind="route" path="POST /api/v1/checkout/pay" file="src/api/routes.rs" lines="65" />
    </callers>
    <dependencies>
      <callee id="..." kind="method" name="CardGateway::charge" file="src/gateways/stripe.rs" lines="105" confidence="1.0" />
    </dependencies>
  </code_context>
  ```

---

### 4.6 Project Configuration (`cartograph.toml`)

Cartograph loads repository configuration from `cartograph.toml`:

```toml
[project]
name = "cartograph"
version = "0.1.0"

[indexer]
exclude_patterns = ["target", "node_modules", "dist", ".git", ".cartograph"]
max_file_size_mb = 2
mmap_threshold_kb = 64
threads = 8

[locate]
default_max_tokens = 4000
default_context_lines = 3
default_detail = "signatures"
default_format = "terminal"
min_confidence = 0.5
resolve_types = true

[watcher]
debounce_ms = 25
notify_blast_radius = true
```

---

## 5. CLI Interface Specification (`cartograph locate`)

### 5.1 Command Line Usage

```bash
cartograph locate <QUERY> [OPTIONS]
```

#### Arguments
- `<QUERY>`: Symbol name, qualified identifier (`AuthService::verify`), canonical URI, natural language feature search, or query with inline directives (`kind:fn path:src/`).

#### Options
- `-p, --path <PATH>`: Root directory of repository or target code scope (default: `.`).
- `-d, --db <DB>`: Database profile or custom SQLite file path (default: `default`).
- `-k, --kind <KIND>`: Comma-separated filter of `NodeKind`s (e.g. `function,method,class,route`).
- `--depth <N>`: Subgraph traversal depth radius (default: `1`, range: `0..5`).
- `--flow`: Trace linear execution flow chain instead of radial BFS.
- `--direction <both|downstream|upstream>`: Graph traversal direction (default: `both`).
- `--edges <KINDS>`: Restrict traversal to specific edge kinds (e.g. `calls,imports,mapsto,tests`).
- `--min-confidence <FLOAT>`: Minimum edge confidence threshold (`0.0..1.0`, default: `0.0`).
- `--resolve-types`: Automatically inlines struct/interface schemas of parameter and return types.
- `--detail <minimal|signatures|full>`: Context density tier (default: `signatures`, or `full` if `--code` is passed).
- `-c, --code`: Shorthand for `--detail full` with source code body inlining.
- `--context <LINES>`: Number of surrounding context padding lines (default: `3`).
- `--max-tokens <TOKENS>`: Hard context token budget ceiling for AI agent prompts (e.g. `2000`, `4000`).
- `-l, --limit <N>`: Maximum number of root candidate nodes to return (default: `5`).
- `-f, --format <terminal|interactive|markdown|json|agent-xml>`: Output formatting style (default: `terminal`). Auto-switches to `json` when piped.
- `-i, --interactive`: Launch interactive terminal TUI with live search and split-pane preview.
- `-v, --verbose`: Display detailed scoring metrics and traversal timings.

---

### 5.2 Example CLI Invocations

```bash
# 1. Quick symbol lookup with 1-hop dependencies and inlined types
cartograph locate RateCard --resolve-types

# 2. Qualified symbol with full code body and caller/callee context
cartograph locate "AuthService::verify_token" --code

# 3. Inline query directives: search function in billing module with min-confidence
cartograph locate "process_payment kind:function path:src/billing min-conf:0.8" --code

# 4. End-to-end execution flow trace from endpoint to database model
cartograph locate "POST /api/v1/orders" --flow --code

# 5. Natural language feature search with token budget ceiling
cartograph locate "accrual period calculation" --depth 2 --code --max-tokens 3000

# 6. Export structured prompt context for Claude / OpenAI system prompt
cartograph locate "OrderService::create" --format agent-xml --max-tokens 4000 > prompt_context.xml

# 7. Export Markdown with Mermaid diagram for GitHub PR review
cartograph locate "OrderRouter" --depth 2 --format markdown

# 8. Interactive terminal TUI mode for live fuzzy exploration
cartograph locate -i
```

---

## 6. Performance Engineering & Latency Budgets

Cartograph enforces an uncompromising **<3ms end-to-end SLA** for 1-turn subgraph localization on a 50,000-node codebase:

| Pipeline Stage | Operation Description | Latency Budget |
| :--- | :--- | :--- |
| **1. Query Parsing** | Intent classification, inline directive extraction, tokenization | `< 0.05 ms` |
| **2. Hybrid Search** | Exact match scan + FTS5 BM25 trigram query + degree & churn scoring | `< 1.20 ms` |
| **3. Subgraph Traversal** | Bi-directional BFS / flow chain + edge confidence filter + type resolution | `< 0.80 ms` |
| **4. Code Slicing** | Memory-mapped line slicing with context padding | `< 0.50 ms` |
| **5. Token Budgeting** | Heuristic token counting and hierarchical priority packing | `< 0.10 ms` |
| **6. Output Formatting** | ANSI tree rendering / Markdown Mermaid / MCP JSON / Agent XML | `< 0.35 ms` |
| **TOTAL** | **Complete 1-Turn Subgraph Retrieval** | **`< 3.00 ms`** |

---

## 7. Phased Implementation Strategy & Task Breakdown

### Phase 1.4 Task Breakdown

- [ ] **Task 1.4.1: Storage Layer Schema Upgrades & Batch Edge Queries (`cartograph-core` & `cartograph-db`)**
  - Add `confidence: Option<f64>` and `weight: Option<f64>` to `Edge` struct in `cartograph-core`.
  - Update SQLite schema migration: add `confidence REAL DEFAULT 1.0` to `edges` table.
  - Implement `get_edges_for_nodes(&self, node_ids: &[String], min_confidence: f64) -> Result<Vec<Edge>>` in `SqliteBackend` and `MemoryBackend`.
  - Implement column-weighted FTS5 BM25 search in `SqliteBackend`.

- [ ] **Task 1.4.2: Query Intent Classifier & Inline Directive Parser (`cartograph-engine::locate::query`)**
  - Implement `QueryParser` with inline directive regex extraction (`kind:`, `path:`, `lang:`, `min-conf:`).
  - Add CamelCase, snake_case, and kebab-case token segmentation.
  - Implement `LocateQuery` struct with `TraversalMode`, `ContextDetail`, and `OutputFormat`.

- [ ] **Task 1.4.3: Centrality- & Churn-Aware Hybrid Search Engine (`cartograph-engine::locate::search`)**
  - Implement multi-channel candidate generation (Exact, Qualified, FTS5 Trigram, Substring).
  - Implement topological degree centrality calculation ($S_{\text{centrality}}$).
  - Implement git recency / churn scoring ($B_{\text{churn}}$) using commit logs.
  - Implement Reciprocal Rank Fusion (RRF) score normalization.
  - Implement multi-root subsystem candidate clustering (`cluster_candidates`).

- [ ] **Task 1.4.4: Subgraph Neighborhood, Flow Chain & Type Resolution (`cartograph-engine::locate::subgraph`)**
  - Implement bi-directional BFS neighborhood traversal supporting depth `0..5`.
  - Implement linear execution flow chain tracing (`--flow`).
  - Implement parameter and return type contract inlining (`--resolve-types`).
  - Add edge confidence threshold filtering (`--min-confidence`).
  - Enforce fanout guards (`max_fanout_per_node = 15`) and cycle detection.

- [ ] **Task 1.4.5: AST Code Slicer & AI Token Budget Manager (`cartograph-engine::locate::slicer` & `budget.rs`)**
  - Implement zero-copy / line-cached source file slicing using AST `Span`.
  - Implement context padding lines (`--context <N>`).
  - Implement context density tiers (`minimal`, `signatures`, `full`).
  - Implement `TokenBudgetManager` with hierarchical priority degradation.

- [ ] **Task 1.4.6: Quad-Engine Output Formatters & Interactive TUI (`cartograph-engine::locate::formatters`)**
  - Implement `TerminalFormatter` with ANSI color badges, clickable links, and ASCII trees.
  - Implement `MarkdownFormatter` with embedded Mermaid `graph LR` flowcharts and fenced code blocks.
  - Implement `JsonFormatter` producing strongly-typed MCP-compliant JSON payloads.
  - Implement `AgentXmlFormatter` producing structured `<code_context>` prompt tags.
  - Implement interactive terminal TUI (`-i, --interactive`) with live search and split-pane preview.

- [ ] **Task 1.4.7: Configuration File Manager (`cartograph.toml`)**
  - Implement `cartograph_core::config::ProjectConfig` parser for `cartograph.toml`.
  - Wire default settings into `LocateQuery`, `DiscoveryConfig`, and `WatcherSettings`.

- [ ] **Task 1.4.8: High-Level SubgraphLocator Coordinator & Public Engine API**
  - Wire all locate submodules into `SubgraphLocator` in `crates/cartograph-engine/src/locate/mod.rs`.
  - Expose clean asynchronous API: `locator.locate(query).await -> Result<LocateOutput>`.

- [ ] **Task 1.4.9: CLI Integration (`cartograph locate` in `cartograph-cli`)**
  - Add `Locate` subcommand and all CLI flag bindings in `crates/cartograph-cli/src/main.rs`.
  - Implement TTY vs pipe auto-detection (defaulting to colorized terminal for TTY and JSON for pipes).
  - Wire up interactive TUI flag (`-i`).

- [ ] **Task 1.4.10: Comprehensive Unit, Integration, and Benchmark Test Suite**
  - Polyglot repository integration tests in `crates/cartograph-engine/tests/test_locate.rs`.
  - Test exact symbol resolution, route mapping resolution, flow chain tracing, and type inlining.
  - Micro-benchmark asserting end-to-end execution completes within the `<3ms` SLA budget.
