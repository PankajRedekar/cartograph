# 🌋 Cartograph Phase 2.5: God-Class & Complexity Heatmap (`cartograph complexity`)
## Comprehensive Technical Design & Architecture Specification (v2.5 Final)

---

## 1. Executive Summary & Architectural Vision

The primary mission of **Phase 2.5 (`cartograph complexity`)** is to deliver an ultra-fast (**sub-25ms** for full repository analysis on 50,000+ nodes, **<3ms** for incremental PR diffs), AST-grounded, multi-language **Code Complexity & Hotspot Heatmap Engine, God-Class & Monster-Method Detector, Graph Coupling Analyzer (Afferent $C_a$, Efferent $C_e$, Instability $I$), Complexity-to-Test Deficit (Danger Zone) Evaluator, AI Context Friction & Reasonability Analyzer, Git Churn & Complexity Trend Correlator, Maintainability Index & Debt Hours Estimator, LCOM4-Based Refactoring Decomposition Advisor, and CI/CD Quality Gate** for software engineers, platform architects, and autonomous AI coding agents.

```
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                               Developer / AI Agent / CI CLI Invocation                                 │
│        cartograph complexity [--top 15] [--granularity symbol|file|package]                            │
│        cartograph complexity --god-classes-only --suggest-refactoring                                  │
│        cartograph complexity --danger-zone                                                             │
│        cartograph complexity --prompt-refactor "BillingManager"                                        │
│        cartograph complexity --trend --since "90 days ago"                                             │
│        cartograph complexity --branch "origin/main...HEAD" --fail-on-god-class                         │
│        cartograph complexity --max-cyclomatic 15 --max-cognitive 20                                    │
│        cartograph complexity --record-baseline .cartograph-complexity-baseline.json                    │
│        cartograph complexity --baseline .cartograph-complexity-baseline.json --ratchet                 │
│        cartograph complexity -i   # Interactive split-pane TUI explorer                                │
│        cartograph complexity --format html > complexity_report.html                                     │
└───────────────────────────────────────────────────┬────────────────────────────────────────────────────┘
                                                    │
                                                    ▼
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                       Cartograph Complexity & Hotspot Engine (<25ms)                                   │
│                                                                                                        │
│   1. Config & Thresholds Resolver  ──► Loads cartograph.toml [complexity] & CLI override flags        │
│   2. Polyglot AST Complexity Engine──► Tree-sitter AST visitor (Cyclomatic, Cognitive, SLOC, Nesting)   │
│   3. Graph Coupling Aggregator     ──► Queries DB for In-degree (Ca), Out-degree (Ce), Instability (I) │
│   4. Test Deficit (CTD) Evaluator  ──► Correlates complexity with Phase 2.2 Test Graph (Tests count)   │
│   5. AI Context Friction Analyzer  ──► Computes Token Density, Context Complexity Ratio (CCR) & Risk   │
│   6. Object-Oriented Cohesion & WMC──► Computes WMC, LCOM4 connected components, TCC, ATFD, Methods   │
│   7. Git Churn & Trend Correlator  ──► Queries commit frequency, author dispersion, Volcanic/Cooling   │
│   8. Maintainability & Debt Estimator► Computes SEI Maintainability Index (0-100) & SQALE Debt Hours   │
│   9. 4-Quadrant Hotspot Classifier ──► Multi-factor Hotspot Score = Complexity * log2(2 + Churn)       │
│  10. God-Class & Smell Rule Engine ──► Evaluates God-Class Index (GCI), Monster Methods, Feature Envy │
│  11. LCOM4 Decomposition Advisor   ──► Partitions method-field graph into candidate cohesive classes   │
│  12. AI Refactoring Prompt Builder ──► Synthesizes ready-to-run prompts for Claude, GPT-4o, Antigravity│
│  13. Technical Debt Baseline Engine──► Suppresses legacy debt; auto-tightens on refactoring wins       │
│  14. PR Git Diff Delta Evaluator   ──► Pinpoints complexity introduced or increased in PR branch diff  │
└───────────────────────────────────────────────────┬────────────────────────────────────────────────────┘
                                                    │
                                                    ▼
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                             Multi-Engine Presentation Layer (<3ms)                                     │
│   • ANSI Terminal:  Color meters (🟢 🟡 🟠 🔴), Hotspot Table, Top-N ranking, Decomposition advice    │
│   • Interactive TUI:Split-pane quadrant matrix & drilldown inspector with sorting & filtering          │
│   • HTML Dashboard: Self-contained zero-dependency interactive dashboard with Treemap & Scatter Chart │
│   • GitHub Markdown:PR summary tables & auto-generated Mermaid Quadrant Scatter Charts (quadrantChart) │
│   • SARIF & JSON:   Standard SARIF 2.1.0 for GitHub Code Scanning alerts & rich machine JSON payload    │
│   • AI Agent XML:   Token-optimized <complexity_hotspot_report> context block for single-turn refactor │
│   • Refactor Prompt:Self-contained, copy-pasteable AI refactoring blueprint prompt                     │
└────────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

### The Problem: Invisible Code Rot, Complexity in a Vacuum & The Refactoring Dilemma

In modern enterprise codebases and AI-assisted pair-programming workflows, software architecture deteriorates through five critical phenomena:

1. **The God-Class & Monster-Method Anti-Pattern**:
   - Over time, central domain classes accumulate dozens of unrelated responsibilities (violating the Single Responsibility Principle).
   - In languages like Python, TypeScript, Rust, Go, and Ruby, classes or modules balloon to 1,000+ lines with 40+ methods and hundreds of transitive dependencies.
   - These God-Classes become cognitive bottlenecks: human developers are terrified of touching them, unit testing requires mocking the entire universe, and autonomous AI coding agents struggle because their methods exhaust context token windows and induce hallucinations.

2. **The "Complexity in a Vacuum" Dilemma (Alert Fatigue)**:
   - Traditional static analysis linters (e.g., `radon`, `lizard`, `SonarQube`, `cloc`) report every single function or class that exceeds an arbitrary line count or cyclomatic complexity threshold.
   - In a 500,000-line repository, this produces **thousands of warnings** on stable legacy code that has not been touched in five years and works reliably in production.
   - Developers and AI agents experience severe alert fatigue, ultimately disabling the linters or ignoring the reports altogether.
   - **The Solution (Adam Tornhill's Hotspot Principle)**: Complexity must be correlated with **Git Change Velocity (Churn & Author Dispersion)**. A 50-cyclomatic-complexity parser that was written 4 years ago and never touched is harmless debt. A 30-cyclomatic-complexity payment handler edited 80 times across 3 months by 12 different engineers is an active volcanic hotspot responsible for 80% of production outages.

3. **The Untested Complexity Danger Zone**:
   - High complexity is dangerous, but high complexity combined with high churn and **zero tests** is a production catastrophe waiting to happen.
   - Traditional linters have zero knowledge of tests; they cannot tell whether a complex class is battle-tested by 50 integration tests or completely untested.
   - Cartograph bridges this gap by cross-referencing its **Predictive Test Graph (Phase 2.2)** to highlight true "Danger Zone" symbols.

4. **The AI Context Friction & Token Reasonability Ceiling**:
   - As software engineering teams increasingly dispatch autonomous AI coding agents to modify code, complex functions with high cognitive complexity and large token footprints exhaust prompt budgets and trigger hallucinations.
   - There is currently no static analysis tool that quantifies whether a code structure is safe for single-turn AI reasoning or represents an extreme hallucination risk.

5. **The "Problem Without a Solution" Dilemma (Lack of Refactoring Guidance)**:
   - Linters that simply proclaim `"Class Foo is too big (LOC > 500)"` offer zero architectural remediation guidance.
   - Developers and AI agents need to know **how** to decompose the God-Class: which methods belong together, which fields they mutate, and what candidate classes should be extracted.

---

### The Solution: Cartograph Complexity 2.5

Cartograph Phase 2.5 delivers an **ultra-high-performance, polyglot-wide, deterministic Complexity & Hotspot Engine in <25ms**:

1. **Multi-Faceted Metric Suite**:
   - **Cyclomatic Complexity ($v(G)$)**: Measures mathematical execution branches across functions and methods.
   - **Cognitive Complexity**: Evaluates nested structural friction and mental comprehension load.
   - **Lines of Code (LOC / SLOC / CLOC)**: Distinguishes physical lines, actual executable code lines, and comment lines.
   - **Nesting Depth & Parameter Density**: Detects deeply nested callback pyramids and overloaded function signatures ($\ge 5$ arguments).
2. **Topological Graph Coupling Metrics**:
   - **Afferent Coupling ($C_a$ / In-Degree)**: Identifies how many external symbols depend on a component (blast radius).
   - **Efferent Coupling ($C_e$ / Out-Degree)**: Identifies how many external symbols a component consumes (brittleness).
   - **Instability Index ($I = \frac{C_e}{C_a + C_e}$)**: Evaluates whether components conform to the Stable Dependencies Principle.
3. **Complexity-to-Test Deficit (CTD / "Danger Zone")**:
   - Cross-references the Phase 2.2 Test Graph:
     $$\text{TestDeficit} = \frac{v(G)}{\max(1, \text{Associated Tests Count})}$$
   - Flags symbols with $v(G) \ge 15$ and 0 associated unit/integration tests as **Critical Danger Zone** items.
4. **AI Context Friction & Reasonability Score (`CCR`)**:
   - Computes **Context Complexity Ratio**:
     $$CCR = \frac{\text{Cognitive Complexity}}{\text{Token Count of Definition}}$$
   - Flags symbols exceeding the **AI Reasonability Ceiling** (e.g. Cognitive $> 30$ and Tokens $> 2,500$) where autonomous single-turn AI code generation error rates exceed 75%.
5. **Maintainability Index (MI) & Estimated Technical Debt Hours (SQALE)**:
   - Computes the normalized **SEI / Microsoft Maintainability Index (0–100)**:
     $$MI = \max\left(0, (171 - 5.2 \ln(V) - 0.23 \cdot v(G) - 16.2 \ln(\text{LOC})) \times \frac{100}{171}\right)$$
   - Estimates concrete **Technical Debt Hours**:
     $$\text{Debt Hours} = \max(0, CC - 15) \times 0.5\text{h} + \max(0, WMC - 50) \times 0.8\text{h} + (LCOM4 - 1) \times 3.0\text{h}$$
6. **Complexity Velocity & Trend Tracking**:
   - Mines historical Git commit snapshots (30, 60, 90 days ago) to assign trend badges:
     - 🌋 **Volcanic (Accelerating)**: $+25\%+$ complexity increase over the time window.
     - 🧊 **Cooling Down (Refactoring)**: $-15\%+$ complexity reduction.
     - ⚪ **Stable**: Minor or no drift.
7. **Object-Oriented Cohesion & God-Class Detection**:
   - **Weighted Methods per Class (WMC)**: Sum of cyclomatic complexities of member methods.
   - **Lack of Cohesion in Methods 4 (LCOM4)**: Graph of methods connected by shared instance fields or direct calls.
   - **Tight Class Cohesion (TCC)**: Fraction of directly connected method pairs.
   - **Access to Foreign Data (ATFD)**: Detects Feature Envy.
   - **God-Class Index (GCI)**: Unified 0–100 score classifying severity (Clean, Minor, Major, Critical).
8. **LCOM4 Graph Decomposition Advisor & AI Prompt Generator (`--prompt-refactor`)**:
   - When $LCOM4 > 1$, automatically partitions methods into candidate extraction clusters with synthesized names.
   - Running `--prompt-refactor <symbol>` emits a ready-to-run, token-optimized prompt for Claude, GPT-4o, or Antigravity.
9. **Adam Tornhill-Inspired Git Churn Hotspot Correlation**:
   - Synthesizes the **Composite Hotspot Score**:
     $$HotspotScore = Complexity \times \log_2(2 + Churn)$$
   - Classifies every symbol and file into Tornhill's **4-Quadrant Hotspot Matrix** (Critical Hotspots, Churn Hotspots, Stable Debt, Healthy).
10. **Git Diff & PR Quality Gating (`--branch`, `--staged`)**:
    - Evaluates delta complexity ($\Delta CC$) and blocks pull requests introducing new God-Classes or regressions beyond thresholds.
11. **Technical Debt Baseline & Auto-Ratchet Engine**:
    - Locks pre-existing complexity into `.cartograph-complexity-baseline.json` and automatically tightens the baseline on refactoring wins (`--ratchet`).
12. **Multi-Engine Presentation Layer (Including Interactive HTML Dashboard)**:
    - ANSI colorized terminal with gradient heat meters.
    - Interactive split-pane TUI explorer (`-i`).
    - **Self-Contained Standalone HTML/SVG Dashboard (`--format html`)**: Interactive zero-dependency dashboard featuring a 4-Quadrant Tornhill scatter chart, spatial treemap, and slide-out inspection drawer (as previewed in `sample_complexity_report.html`).
    - GitHub PR Markdown comments featuring auto-generated **Mermaid Quadrant Scatter Charts (`quadrantChart`)**.
    - SARIF 2.1.0 and machine JSON.
    - Structured AI Agent XML (`<complexity_hotspot_report>`).

---

## 2. Pipeline Architecture & Component Topology

```
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                   Developer, CI/CD, or Agent Input                                     │
│   cartograph complexity [--top 15] [--danger-zone] [--trend] [--prompt-refactor "Foo"] [--format html]  │
└───────────────────────────────────────────────────┬────────────────────────────────────────────────────┘
                                                    │
                                                    ▼
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                       1. Configuration & Threshold Resolver (<1ms)                                     │
│  • Reads cartograph.toml [complexity] section or CLI overrides                                          │
│  • Resolves weights for Cyclomatic, Cognitive, LOC, Coupling, and Git Churn                            │
│  • Configures thresholds for max function CC/Cognitive, max class WMC/LOC/Methods, and GCI             │
│  • Loads baseline fingerprints from .cartograph-complexity-baseline.json if specified                   │
└───────────────────────────────────────────────────┬────────────────────────────────────────────────────┘
                                                    │
                                                    ▼
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                       2. Topological Graph Coupling Extractor (<2ms)                                   │
│  • Executes single-pass batch SQL query against cartograph.db SQLite edges table                       │
│  • Computes Afferent Coupling (In-Degree / Ca) and Efferent Coupling (Out-Degree / Ce) for all symbols   │
│  • Computes Instability Index I = Ce / (Ca + Ce)                                                       │
└───────────────────────────────────────────────────┬────────────────────────────────────────────────────┘
                                                    │
                                                    ▼
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                       3. Polyglot AST Complexity & Token Friction Engine (<10ms)                       │
│  • Tree-sitter CST traversal across Rust, TypeScript, Python, Go, and Ruby                             │
│  • Computes McCabe Cyclomatic Complexity v(G) per function / method (decision points + 1)              │
│  • Computes SonarSource Cognitive Complexity per function / method (nesting penalties, break points)   │
│  • Calculates physical LOC, source SLOC (ignoring whitespace/comments), and max nesting depth          │
│  • Computes Token Footprint, Context Complexity Ratio (CCR), and AI Reasonability Ceiling flag         │
└───────────────────────────────────────────────────┬────────────────────────────────────────────────────┘
                                                    │
                                                    ▼
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                       4. Predictive Test Graph Correlation (<2ms)                                      │
│  • Cross-references Phase 2.2 Test Graph to count incoming Tests edges for each symbol                 │
│  • Computes Complexity-to-Test Deficit (CTD) = v(G) / max(1, TestCount)                                │
│  • Identifies "Danger Zone" components: high complexity + high churn + 0 tests                         │
└───────────────────────────────────────────────────┬────────────────────────────────────────────────────┘
                                                    │
                                                    ▼
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                       5. Class Cohesion & LCOM4 Decomposition Engine (<3ms)                            │
│  • Resolves class-level method and field membership (methods, fields, properties)                      │
│  • Computes Weighted Methods per Class (WMC = sum of method cyclomatic complexities)                   │
│  • Builds internal method-field sharing graph: edges between methods sharing fields or calling each other│
│  • Computes connected components (LCOM4) and Tight Class Cohesion (TCC)                                │
│  • Detects Access to Foreign Data (ATFD) to flag Feature Envy                                          │
│  • When LCOM4 > 1: partitions methods into Candidate Extraction Clusters with synthesized names        │
└───────────────────────────────────────────────────┬────────────────────────────────────────────────────┘
                                                    │
                                                    ▼
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                       6. Git Churn Velocity & Complexity Trend Analyzer (<5ms)                         │
│  • Mines local git log across the configured time window (default: 90 days)                             │
│  • Aggregates commit counts, distinct authors, and recency decay factors per file                      │
│  • Computes Normalized Churn Velocity Score (0.0 to 1.0)                                               │
│  • Compares against historical commit snapshots to compute Complexity Trend (🌋 Volcanic, 🧊 Cooling) │
└───────────────────────────────────────────────────┬────────────────────────────────────────────────────┘
                                                    │
                                                    ▼
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                       7. Maintainability Index & SQALE Debt Estimation (<1ms)                          │
│  • Computes SEI Maintainability Index (0–100 scale) based on Halstead Volume, CC, and LOC               │
│  • Estimates SQALE remediation effort in developer debt hours                                          │
└───────────────────────────────────────────────────┬────────────────────────────────────────────────────┘
                                                    │
                                                    ▼
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                       8. 4-Quadrant Hotspot Classifier & God-Class Evaluator (<2ms)                    │
│  • Synthesizes Composite Hotspot Score = (Complexity Vector) * log2(2 + Churn Velocity)                │
│  • Classifies symbols into Tornhill 4-Quadrant Matrix (Q1 Critical, Q2 Churn, Q3 Healthy, Q4 Debt)    │
│  • Computes God-Class Index (GCI, 0-100) and classifies severity (Clean, Minor, Major, Critical)     │
└───────────────────────────────────────────────────┬────────────────────────────────────────────────────┘
                                                    │
                                                    ▼
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                       9. Baseline Ratchet & PR Delta Evaluator (<1ms)                                  │
│  • Hashes violation signatures: SHA-256(rule, symbol_id, filepath, metric_kind)                        │
│  • Suppresses pre-existing baseline debt (reported as Baseline Technical Debt)                         │
│  • Evaluates git diff lines (--branch / --staged) to detect newly introduced complexity or delta CC    │
│  • If --ratchet: permanently removes resolved complexity items from baseline file                      │
└───────────────────────────────────────────────────┬────────────────────────────────────────────────────┘
                                                    │
                                                    ▼
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                      10. Multi-Engine Presentation Layer (<2ms)                                        │
│  ┌───────────────────────┐ ┌──────────────────────┐ ┌───────────────────┐ ┌──────────────────────────┐ │
│  │ ANSI Terminal & TUI   │ │ HTML Dashboard       │ │ SARIF & JSON      │ │ AI XML & Refactor Prompt │ │
│  │ (Meters, table, advice│ │ (Interactive Treemap)│ │ (GitHub Scanning) │ │ (<complexity_hotspot>)   │ │
│  └───────────────────────┘ └──────────────────────┘ └───────────────────┘ └──────────────────────────┘ │
│  • CI Exit Code Evaluation: Exit 1 if new violations exceed threshold or fail-on policy violated       │
└────────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Data Models & Interface Contracts

All complexity data structures are strongly typed, serializable via Serde, and positioned in `cartograph-core::model` and `cartograph-engine::complexity`.

### 3.1 Configuration Models (`cartograph-core::config`)

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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

impl std::str::FromStr for ComplexityGranularity {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "symbol" | "fn" | "class" => Ok(ComplexityGranularity::Symbol),
            "file" => Ok(ComplexityGranularity::File),
            "module" | "dir" => Ok(ComplexityGranularity::Module),
            "package" | "crate" => Ok(ComplexityGranularity::Package),
            other => Err(format!("Unknown complexity granularity: {other}. Expected symbol, file, module, or package")),
        }
    }
}

/// Project-level configuration for cartograph complexity in cartograph.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
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

fn default_weight_cyclomatic() -> f64 { 0.35 }
fn default_weight_cognitive() -> f64 { 0.25 }
fn default_weight_loc() -> f64 { 0.15 }
fn default_weight_coupling() -> f64 { 0.15 }
fn default_weight_churn() -> f64 { 0.10 }

fn default_max_function_cyclomatic() -> u32 { 15 }
fn default_max_function_cognitive() -> u32 { 20 }
fn default_max_function_loc() -> u32 { 100 }
fn default_max_nesting_depth() -> u32 { 4 }
fn default_max_parameters() -> u32 { 5 }

fn default_max_class_wmc() -> u32 { 50 }
fn default_max_class_loc() -> u32 { 500 }
fn default_max_class_methods() -> usize { 20 }
fn default_min_lcom4() -> usize { 2 }
fn default_max_god_class_index() -> f64 { 75.0 }

fn default_max_ai_tokens() -> usize { 2500 }
fn default_max_ai_cognitive() -> u32 { 30 }

fn default_churn_window_days() -> u32 { 90 }
fn default_churn_max_commits() -> usize { 100 }
fn default_top_n() -> usize { 15 }

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
```

### 3.2 Core Metric Data Models (`cartograph-core::model`)

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Primary AST-level complexity metrics for a function or method
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HotspotQuadrant {
    /// Q1: High Complexity & High Churn (Top Refactoring Priority) 🚨
    CriticalHotspot,
    /// Q2: Low Complexity & High Churn (High Volatility, Needs Good Tests) ⚠️
    ChurnHotspot,
    /// Q3: Low Complexity & Low Churn (Healthy Code) ✅
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GodClassSeverity {
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
```

---

## 4. Detailed Component Design & Implementation

### 4.1 Polyglot Tree-Sitter AST Complexity Visitor (`cartograph-ast`)

The AST complexity visitor walks the Tree-Sitter concrete syntax tree, extracting **McCabe Cyclomatic Complexity**, **SonarSource Cognitive Complexity**, **SLOC**, and **Maximum Nesting Depth** in a single efficient pass.

```
Tree-Sitter CST Node
       │
       ├──► Is Function/Method? ──► Initialize local counters (CC = 1, Cog = 0, Nesting = 0)
       │         │
       │         ├──► Is Decision Branch? (if, for, while, catch, match_arm, &&, ||, ?)
       │         │         ├──► CC += 1
       │         │         └──► Cognitive += (1 + NestingLevel)
       │         │
       │         ├──► Increases Nesting Level? (nested blocks, lambdas, inner functions)
       │         │         └──► NestingLevel += 1
       │         │
       │         └──► Leaves Scope? ──► Record function metrics
       │
       └──► Is Class/Struct? ──► Aggregate method WMC and construct field access map
```

#### Decision Points by Language Grammar

| Language | Cyclomatic Decision Points ($+1$ to $v(G)$) | Cognitive Complexity Increments ($\Delta = 1 + \text{nesting}$) |
|---|---|---|
| **Rust** | `if_expression`, `while_expression`, `for_expression`, `loop_expression`, `match_arm` (non-wildcard), `try_expression` (`?`), binary `&&`, `||` | `if_expression`, `while_expression`, `for_expression`, `loop_expression`, `match_arm`, closures, nested control structures |
| **TypeScript / JS** | `if_statement`, `while_statement`, `do_statement`, `for_statement`, `for_in_statement`, `for_of_statement`, `switch_case`, `catch_clause`, `ternary_expression`, binary `&&`, `||`, `??` | Control flow breaks + nesting penalty; flat switch gets flat $+1$; arrow functions within methods increment nesting |
| **Python** | `if_statement`, `elif_clause`, `for_statement`, `while_statement`, `except_clause`, `conditional_expression`, `match_case`, boolean `and`, `or` | Control flow breaks + nesting penalty; list comprehensions with `if` add $+1$; generators add $+1$ |
| **Go** | `if_statement`, `for_statement`, `expression_case_clause`, `type_case_clause`, `select_statement`, `communication_case`, binary `&&`, `||` | Control flow breaks + nesting penalty; `select` cases get flat $+1$ |
| **Ruby** | `if`, `elsif`, `unless`, `while`, `until`, `for`, `rescue`, `when`, `conditional`, `&&`, `||`, `and`, `or` | Control flow breaks + nesting penalty; block invocations (`do ... end`) increment nesting if conditional |

---

### 4.2 Complexity-to-Test Deficit (CTD) & "Danger Zone" Detection

Cartograph connects the AST complexity engine with its **Predictive Test Graph** (established in Phase 2.2):

1. **Test Graph Cross-Reference**:
   Queries incoming `Tests` edges for the symbol $S$, resolving direct and transitive tests:
   $$T(S) = \{ t \mid t \xrightarrow{\text{Tests}} S \}$$
2. **Deficit Ratio**:
   $$\text{TestDeficit}(S) = \frac{v(G)}{\max(1, |T(S)|)}$$
3. **Danger Zone Trigger**:
   A symbol is flagged as a **Danger Zone Item** when:
   $$v(G) \ge 15 \quad \land \quad |T(S)| = 0 \quad \land \quad V_{\text{churn}} \ge 0.40$$
   This highlights complex, actively changing code operating with zero test safety net!

---

### 4.3 AI Context Friction & Reasonability Score (`CCR`)

To guard against AI agent hallucinations during automated refactoring turns:

1. **Token Estimation**:
   Computes token count $T_{\text{tokens}}$ of the symbol's full signature and body (approx. 3.6 chars/token or exact BPE estimate).
2. **Context Complexity Ratio**:
   $$CCR = \frac{\text{Cognitive Complexity}}{T_{\text{tokens}}}$$
3. **AI Reasonability Ceiling**:
   Flags functions or methods where:
   $$\text{Cognitive Complexity} > 30 \quad \lor \quad T_{\text{tokens}} > 2,500$$
   When flagged, Cartograph alerts autonomous agents:
   `⚠️ High AI Hallucination Risk: Symbol exceeds single-turn AI reasoning capacity. Decompose before editing.`

---

### 4.4 Maintainability Index & SQALE Technical Debt Hours

1. **Maintainability Index (MI)**:
   Computes Halstead Volume $V$, McCabe Cyclomatic $v(G)$, and LOC:
   $$MI = \max\left(0.0, \min\left(100.0, \left(171 - 5.2 \ln(V) - 0.23 \cdot v(G) - 16.2 \ln(\text{LOC})\right) \times \frac{100.0}{171.0}\right)\right)$$
   - 🟢 **$>50$**: Good maintainability.
   - 🟡 **$20-50$**: Moderate maintainability.
   - 🔴 **$<20$**: Unmaintainable / High technical debt.

2. **SQALE Debt Hours Estimation**:
   Quantifies remediation effort in developer hours:
   $$\text{Debt Hours} = \max(0, CC - 15) \times 0.5\text{h} + \max(0, WMC - 50) \times 0.8\text{h} + (LCOM4 - 1) \times 3.0\text{h}$$

---

### 4.5 Complexity Velocity & Trend Tracking

By checking Git commit trees at $t_0$ (now), $t_{-30}$, and $t_{-90}$ days:
1. Computes $\Delta CC = CC(t_0) - CC(t_{-90})$.
2. Percentage Change:
   $$\% \Delta = \frac{CC(t_0) - CC(t_{-90})}{CC(t_{-90})} \times 100\%$$
3. Categorization:
   - 🌋 **Volcanic (Accelerating)**: $\% \Delta \ge +25\%$
   - 🧊 **Cooling Down (Refactoring)**: $\% \Delta \le -15\%$
   - ⚪ **Stable**: Between $-15\%$ and $+25\%$

---

### 4.6 Class Cohesion & LCOM4 Graph Decomposition Advisor (`cartograph-engine`)

The **LCOM4 (Lack of Cohesion in Methods)** metric is formulated as graph connectivity:

Given a class $C$ with member methods $M = \{m_1, m_2, \dots, m_k\}$ and fields $F = \{f_1, f_2, \dots, f_p\}$:
1. Extract for each method $m_i$:
   - $\text{FieldsAccessed}(m_i) \subseteq F$ (fields read or written by $m_i$)
   - $\text{MethodsCalled}(m_i) \subseteq M$ (other member methods of $C$ called by $m_i$)
2. Construct an undirected graph $G_C = (M, E_C)$, where an undirected edge $(m_i, m_j) \in E_C$ exists if and only if:
   $$\text{FieldsAccessed}(m_i) \cap \text{FieldsAccessed}(m_j) \ne \emptyset \quad \lor \quad m_i \in \text{MethodsCalled}(m_j) \quad \lor \quad m_j \in \text{MethodsCalled}(m_i)$$
3. Compute the **connected components** of $G_C$:
   $$LCOM4 = |\text{ConnectedComponents}(G_C)|$$

#### Automated AI Refactoring Blueprint Generator (`--prompt-refactor <symbol>`)
When an engineer or agent executes:
```bash
cartograph complexity --prompt-refactor "BillingManager"
```
Cartograph outputs a self-contained, token-optimized prompt containing:
1. Target class definition and coordinates.
2. Exact LCOM4 decomposition clusters (methods, fields to migrate).
3. Candidate class names (e.g. `PaymentGatewayService`, `SubscriptionPlanManager`).
4. Step-by-step extraction instructions preserving public API compatibility via composition delegation.

---

### 4.7 4-Quadrant Hotspot Matrix & Composite Scoring

Cartograph synthesizes structural complexity with Git churn velocity into a single normalized 0–100 **Hotspot Score**:

$$Score = \left( 0.35 \times \frac{v(G)}{15} + 0.25 \times \frac{\text{Cognitive}}{20} + 0.15 \times \frac{\text{LOC}}{100} + 0.15 \times \frac{C_a + C_e}{20} \right) \times \log_2(2 + 4 \times V_{\text{churn}}) \times 25.0$$
capped at $100.0$.

- **Q1: Critical Hotspots (🚨 Top Priority)**: Complex code undergoing frequent changes.
- **Q2: Churn Hotspots (⚠️ Volatile)**: Simple code that changes constantly.
- **Q3: Healthy Code (✅ Green)**: Modest complexity, low churn.
- **Q4: Stable Technical Debt (⏳ Defer)**: Complex code that has not been touched in months/years.

---

## 5. Storage Layer & Graph Traversal Query Enhancements (`cartograph-db`)

### 5.1 Batch Symbol Coupling Query

```sql
SELECT 
    n.id,
    COALESCE(incoming.in_degree, 0) AS in_degree,
    COALESCE(outgoing.out_degree, 0) AS out_degree
FROM nodes n
LEFT JOIN (
    SELECT target_id, COUNT(*) AS in_degree
    FROM edges
    WHERE kind IN ('calls', 'imports', 'references', 'extends', 'implements')
    GROUP BY target_id
) incoming ON n.id = incoming.target_id
LEFT JOIN (
    SELECT source_id, COUNT(*) AS out_degree
    FROM edges
    WHERE kind IN ('calls', 'imports', 'references', 'extends', 'implements')
    GROUP BY source_id
) outgoing ON n.id = outgoing.source_id;
```

### 5.2 Class Member & Internal Call Graph Query

```sql
SELECT 
    m.id, m.name, m.kind, m.start_line, m.end_line, m.metadata
FROM nodes m
WHERE m.scope = ?1 OR m.id IN (
    SELECT target_id FROM edges WHERE source_id = ?1 AND kind = 'contains'
);
```

---

## 6. Multi-Engine Presentation Layer

### 6.1 ANSI Terminal Presentation
Outputs color heat meters, ASCII hotspot tables, Danger Zone alerts, and LCOM4 refactoring advisor boxes.

### 6.2 Self-Contained Standalone HTML/SVG Dashboard (`--format html`)
Generates a zero-dependency, rich interactive HTML report (as implemented in `sample_complexity_report.html`):
- **4-Quadrant Tornhill Scatter Matrix**: Interactive bubble chart sized by LOC and color-coded by quadrant.
- **Spatial Treemap**: Hierarchical heatmap tiles sized by SLOC and colored by complexity heat.
- **Side Inspection Drawer**: Metric breakdown, Danger Zone alerts, AI Context Friction indicators, and LCOM4 decomposition clusters.
- **AI Prompt Modal**: Single-click copyable refactoring prompts.

### 6.3 GitHub PR Markdown with Auto-Generated Mermaid Quadrant Chart
Embeds `quadrantChart` Mermaid visualizations and PR comment tables directly into GitHub Actions.

### 6.4 SARIF 2.1.0, Machine JSON & AI Agent XML
- SARIF 2.1.0 for GitHub Code Scanning alerts.
- Machine JSON for automated pipelines.
- `<complexity_hotspot_report>` XML block for agent context injection.

---

## 7. Performance Engineering & Latency Budgets

| Operation | Budget / SLA | Algorithmic Complexity | Implementation Technique |
|---|---|---|---|
| **Batch Coupling Query** | **<2ms** | $O(E)$ | Single-pass SQLite aggregate `GROUP BY` on indexed `source_id` / `target_id` |
| **AST Tree-Sitter Walk** | **<10ms** | $O(N_{\text{nodes}})$ | Reuses cached tree-sitter parse trees from `cartograph-ast` |
| **LCOM4 Graph Connectivity** | **<3ms** | $O(M + E_M)$ | Breadth-First Search (BFS) / Union-Find on class method-field adjacency graph |
| **Git Churn Mining** | **<5ms** | $O(C)$ | Streaming `git log --pretty=format` pipe with single-pass hashmap accumulation |
| **Test Deficit Resolution**| **<2ms** | $O(T)$ | Fast query on SQLite `Tests` edges |
| **PR Branch Diff Check** | **<3ms** | $O(D_{\text{hunks}})$ | Filters to symbols intersecting `git diff -U0` hunk line ranges |
| **Total Full Run (50k nodes)**| **<25ms** | — | Parallel Rayon workers across independent files |

---

## 8. CLI Interface Specification (`cartograph complexity`)

```bash
# 1. Basic usage: Display top 15 complexity hotspots across repository
cartograph complexity

# 2. Focus on Danger Zone items (high complexity + high churn + 0 tests)
cartograph complexity --danger-zone

# 3. Focus on God-Classes and display decomposition refactoring suggestions
cartograph complexity --god-classes-only --suggest-refactoring

# 4. Generate self-contained AI refactoring prompt for a target God-Class
cartograph complexity --prompt-refactor "BillingManager"

# 5. Display complexity drift trend (Volcanic vs Cooling Down)
cartograph complexity --trend --since "90 days ago"

# 6. CI/CD Quality Gate: Fail build if any function CC > 15 or any God-Class exists
cartograph complexity --max-cyclomatic 15 --fail-on-god-class

# 7. Scope complexity check to Pull Request git branch diff
cartograph complexity --branch "origin/main...HEAD" --fail-on-god-class

# 8. Technical Debt Baseline: record existing debt and ratchet improvements
cartograph complexity --record-baseline .cartograph-complexity-baseline.json
cartograph complexity --baseline .cartograph-complexity-baseline.json --ratchet

# 9. Launch interactive split-pane terminal TUI explorer
cartograph complexity -i

# 10. Generate standalone interactive HTML/SVG dashboard
cartograph complexity --format html > complexity_report.html
```

---

## 9. Phased Implementation Strategy & Task Breakdown

### Phase 2.5A: Core Models, Configurations & AST Complexity Extractor
- [ ] **Task 2.5.1**: Define complexity, coupling, cohesion, churn, test deficit, AI friction, trend, and hotspot data structures in `cartograph-core::model`.
- [ ] **Task 2.5.2**: Add `ComplexityConfig` and threshold defaults in `cartograph-core::config`.
- [ ] **Task 2.5.3**: Implement polyglot Tree-Sitter AST complexity visitor in `cartograph-ast::complexity` (Cyclomatic, Cognitive, SLOC, Nesting Depth across Rust, TS, Python, Go, Ruby).
- [ ] **Task 2.5.4**: Add unit tests for AST complexity visitor validating exact scores against complex code fixtures.

### Phase 2.5B: Database Coupling Queries, Test Graph Deficit & LCOM4 Engine
- [ ] **Task 2.5.5**: Implement batch SQL coupling query (`get_symbol_coupling_metrics`) in `cartograph-db::sqlite::traversal`.
- [ ] **Task 2.5.6**: Implement class member & internal call graph query (`get_class_member_graph`) in `cartograph-db`.
- [ ] **Task 2.5.7**: Implement Test Deficit resolver linking to Phase 2.2 Test Graph edges.
- [ ] **Task 2.5.8**: Build LCOM4 method-field connectivity graph and connected components algorithm in `cartograph-engine::complexity::god_class`.
- [ ] **Task 2.5.9**: Implement LCOM4 automated decomposition advisor and `--prompt-refactor` blueprint generator.

### Phase 2.5C: Git Churn Analyzer, Trend Tracker & 4-Quadrant Hotspot Ranker
- [ ] **Task 2.5.10**: Implement zero-copy Git log streaming churn analyzer in `cartograph-engine::complexity::churn`.
- [ ] **Task 2.5.11**: Implement Complexity Trend evaluator comparing against historical Git commit trees (Volcanic/Cooling).
- [ ] **Task 2.5.12**: Implement 4-Quadrant Hotspot classifier, Maintainability Index, and SQALE debt hours estimation.
- [ ] **Task 2.5.13**: Implement multi-granularity metric aggregation (Symbol, File, Module, Package).

### Phase 2.5D: Git Diff Delta, Baseline Ratchet & Presentation Layer
- [ ] **Task 2.5.14**: Implement Git diff hunk intersection and delta complexity tracking (`cartograph-engine::complexity::diff`).
- [ ] **Task 2.5.15**: Implement technical debt baseline manager with SHA-256 fingerprinting and `--ratchet` pruning.
- [ ] **Task 2.5.16**: Implement ANSI terminal heatmap formatter with color meters and ASCII tables.
- [ ] **Task 2.5.17**: Implement interactive split-pane TUI explorer (`-i`).
- [ ] **Task 2.5.18**: Implement standalone self-contained HTML/SVG dashboard generator (`--format html`).
- [ ] **Task 2.5.19**: Implement GitHub PR Markdown generator with auto-generated Mermaid `quadrantChart`.
- [ ] **Task 2.5.20**: Implement SARIF 2.1.0, machine JSON, and AI Agent XML formatters.

### Phase 2.5E: CLI Integration, End-to-End Testing & Documentation
- [ ] **Task 2.5.21**: Register `Complexity` subcommand in `cartograph-cli::main.rs` with all CLI flags.
- [ ] **Task 2.5.22**: Write comprehensive integration tests in `crates/cartograph-engine/tests/test_complexity.rs`.
- [ ] **Task 2.5.23**: Update `ROADMAP.md` and `README.md` with Phase 2.5 documentation and usage guides.
