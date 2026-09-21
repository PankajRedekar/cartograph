# code-complexity-heatmap Specification

## Purpose
Computes multi-dimensional software complexity metrics (McCabe Cyclomatic, SonarSource Cognitive, Lines of Code, Nesting Depth), topological graph coupling metrics (Afferent $C_a$, Efferent $C_e$, Instability $I$), evaluates Complexity-to-Test Deficit (Danger Zone) using the Test Graph, computes AI Context Friction & Reasonability Scores (CCR), tracks historical Complexity Velocity & Trends (Volcanic vs Cooling), calculates SEI Maintainability Index & SQALE Debt Hours, evaluates object-oriented cohesion (WMC, LCOM4, TCC, ATFD), detects God-Classes with an automated LCOM4-based refactoring decomposition advisor and `--prompt-refactor` generator, manages technical debt baselines with auto-ratcheting, and provides rich multi-engine reports (terminal heatmaps, interactive TUI, standalone interactive HTML/SVG dashboard, GitHub PR Markdown with Mermaid quadrant charts, SARIF 2.1.0, JSON, AI XML).

## Requirements

### Requirement: Polyglot AST Complexity Extraction
The system SHALL compute McCabe Cyclomatic Complexity ($v(G)$), SonarSource Cognitive Complexity, physical Lines of Code (LOC), Source Lines of Code (SLOC), and maximum nesting depth across Rust, TypeScript/JavaScript, Python, Go, and Ruby using Tree-Sitter concrete syntax trees.

#### Scenario: Computing cyclomatic and cognitive complexity for a branching function
- **WHEN** a function contains nested conditionals, loops, and boolean logic
- **THEN** cartograph computes McCabe Cyclomatic Complexity based on decision points + 1
- **AND** computes SonarSource Cognitive Complexity penalizing nested control structures exponentially while treating flat switch/match statements without nesting penalties.

---

### Requirement: Topological Graph Coupling Metrics
The system SHALL compute Afferent Coupling ($C_a$ / in-degree), Efferent Coupling ($C_e$ / out-degree), and the Instability Index ($I = \frac{C_e}{C_a + C_e}$) for all symbols, files, modules, and packages via single-pass batch SQL queries against the graph database.

#### Scenario: Evaluating component stability
- **WHEN** a central domain entity is imported by 25 external services and imports only 1 external utility
- **THEN** cartograph computes $C_a = 25$, $C_e = 1$, and Instability $I = 0.038$ (highly stable core).

---

### Requirement: Complexity-to-Test Deficit (Danger Zone) Evaluation
The system SHALL cross-reference the Phase 2.2 Test Graph to count incoming `Tests` edges for each symbol, compute the test deficit ratio $\text{TestDeficit} = \frac{v(G)}{\max(1, \text{TestCount})}$, and flag components with high complexity ($v(G) \ge 15$) and 0 tests as Critical Danger Zone items.

#### Scenario: Flagging untested complex code
- **WHEN** a modified function has Cyclomatic Complexity 24, Git churn 0.75, and 0 associated tests
- **AND** `cartograph complexity --danger-zone` is executed
- **THEN** cartograph flags the function with a Danger Zone warning badge and provides targeted test generation guidance.

---

### Requirement: AI Context Friction & Reasonability Analysis
The system SHALL compute estimated prompt tokens and the Context Complexity Ratio ($CCR = \frac{\text{Cognitive Complexity}}{\text{Tokens}}$), flagging symbols that exceed single-turn AI reasoning ceilings (Cognitive $> 30$ and Tokens $> 2,500$) as high hallucination risks.

#### Scenario: Warning about excessive AI reasoning load
- **WHEN** a God-Class method requires 3,500 tokens of context and has Cognitive Complexity 48
- **THEN** cartograph flags the symbol with high AI Context Friction and alerts autonomous agents to decompose it before attempting edits.

---

### Requirement: Maintainability Index & SQALE Technical Debt Estimation
The system SHALL calculate the normalized SEI Maintainability Index (0.0 to 100.0) based on Halstead Volume, Cyclomatic Complexity, and LOC, and estimate remediation effort in developer debt hours via SQALE-inspired formulas.

#### Scenario: Computing technical debt hours
- **WHEN** analyzing a God-Class with WMC 78 and LCOM4 3
- **THEN** cartograph reports the Maintainability Index score and estimates remediation effort (e.g. ~14.5 hours).

---

### Requirement: Complexity Velocity & Trend Tracking
The system SHALL compare current symbol and file complexity against historical Git commit trees (30, 60, 90 days ago), classifying components into 🌋 Volcanic (accelerating complexity $\ge +25\%$), 🧊 Cooling Down (improving complexity $\le -15\%$), or ⚪ Stable.

#### Scenario: Tracking complexity drift over time
- **WHEN** `cartograph complexity --trend` is executed
- **THEN** cartograph displays percentage complexity drift and tags rapidly deteriorating classes with Volcanic status badges.

---

### Requirement: Object-Oriented Cohesion & God-Class Detection
The system SHALL compute Weighted Methods per Class (WMC), Lack of Cohesion in Methods (LCOM4), Tight Class Cohesion (TCC), and Access to Foreign Data (ATFD) for structs, classes, and interfaces. It SHALL synthesize a composite God-Class Index (GCI, 0–100) and flag God-Classes exceeding configured thresholds.

#### Scenario: Detecting a God-Class with high WMC and disconnected LCOM4
- **WHEN** a struct has WMC $\ge 50$, LOC $\ge 500$, method count $\ge 20$, and LCOM4 $\ge 2$
- **THEN** cartograph flags the struct as a `Critical God-Class` with detailed heuristic triggers.

---

### Requirement: Automated LCOM4 Refactoring Decomposition Advisor & AI Prompt Builder
The system SHALL construct an undirected method-field sharing graph for each flagged God-Class, compute its connected components (LCOM4 clusters), and automatically synthesize candidate class extractions with concrete method groupings, field assignments, candidate service names, and ready-to-run AI prompts (`--prompt-refactor <symbol>`).

#### Scenario: Generating an AI refactoring prompt
- **WHEN** `cartograph complexity --prompt-refactor "BillingManager"` is executed
- **THEN** cartograph outputs a token-optimized markdown prompt with exact LCOM4 clusters and delegation rules for autonomous agents.

---

### Requirement: Git Churn Velocity Mining & 4-Quadrant Hotspot Classification
The system SHALL mine local Git commit history across a configurable time window (default: 90 days) to compute commit counts, distinct authors, and recency decay. It SHALL synthesize a Composite Hotspot Score ($Score = Complexity \times \log_2(2 + Churn)$) and classify components into a 4-Quadrant Hotspot Matrix:
- Q1: Critical Hotspots (High Complexity + High Churn) 🚨
- Q2: Churn Hotspots (Low Complexity + High Churn) ⚠️
- Q3: Healthy Code (Low Complexity + Low Churn) ✅
- Q4: Stable Technical Debt (High Complexity + Low Churn) ⏳

#### Scenario: Prioritizing refactoring on high-churn hotspots
- **WHEN** `cartograph complexity --top 10` is executed
- **THEN** components with high complexity and frequent commit churn are ranked at the top of the report, while complex legacy code with zero churn is classified as Q4 Stable Debt.

---

### Requirement: Git Diff & Pull Request Scoped Complexity Gating
The system SHALL support scoping complexity analysis to staged changes (`--staged`) or a Git branch diff (`--branch <range>`), tracking delta complexity ($\Delta CC, \Delta Cognitive$) and preventing pull requests from introducing new God-Classes or regressions beyond configured thresholds (`--fail-on-god-class`, `--max-cyclomatic <N>`).

#### Scenario: Gating pull requests in CI
- **WHEN** `cartograph complexity --branch "origin/main...HEAD" --max-cyclomatic 15 --fail-on-god-class` is executed in CI
- **THEN** cartograph fails with exit code 1 if and only if the PR diff introduced a new God-Class or modified a function whose cyclomatic complexity exceeds 15.

---

### Requirement: Technical Debt Baseline & Auto-Ratchet Engine
The system SHALL support recording all pre-existing complexity violations into a baseline file (`--record-baseline <file>`). When analyzing against a baseline (`--baseline <file>`), it SHALL suppress known violations and fail CI only on new violations. With `--ratchet`, it SHALL automatically prune resolved violations from the baseline file.

#### Scenario: Recording and checking against complexity baseline
- **WHEN** `cartograph complexity --record-baseline .cartograph-complexity-baseline.json` is run
- **THEN** existing hotspots and God-Classes are fingerprinted with SHA-256 signatures.
- **WHEN** `cartograph complexity --baseline .cartograph-complexity-baseline.json` is executed subsequently
- **THEN** pre-existing debt is suppressed and CI passes.

---

### Requirement: Multi-Engine Presentation Layer (Including Interactive HTML Dashboard)
The system SHALL format complexity and hotspot reports across:
1. ANSI Terminal with color heat meters (🟢 🟡 🟠 🔴) and ASCII hotspot tables.
2. Interactive split-pane TUI explorer (`-i`).
3. Self-contained standalone interactive HTML/SVG Dashboard (`--format html`) with 4-Quadrant scatter plot, spatial treemap, and inspection drawer.
4. GitHub PR Markdown comments with auto-generated Mermaid quadrant scatter charts (`quadrantChart`).
5. Standard SARIF 2.1.0 for GitHub Code Scanning alerts.
6. Machine JSON for MCP tools and automation.
7. AI Agent XML prompt context (`<complexity_hotspot_report>`).
8. Copyable AI Refactoring Blueprint prompts (`--prompt-refactor`).

#### Scenario: Multi-engine formatting and HTML dashboard generation
- **WHEN** `cartograph complexity --format html` is executed
- **THEN** a standalone HTML/SVG dashboard with interactive quadrant scatter plot is emitted.
- **WHEN** `cartograph complexity --format markdown` is executed
- **THEN** a GitHub-compatible markdown table and Mermaid `quadrantChart` are generated.

