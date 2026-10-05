# 🤖 Cartograph Phase 3.2: Contract-Aware AI Reviewer (`cartograph ai-review`)
## Comprehensive Technical Design & Architecture Specification (v3.2 Final)

---

## 1. Executive Summary & Architectural Vision

The primary mission of **Phase 3.2 (`cartograph ai-review`)** is to deliver an ultra-fast (**sub-100ms** for offline deterministic graph review, **<2.5s** for AI-augmented synthesis), AST-grounded, polyglot **Contract-Aware Code Review Engine, Downstream Side-Effect Verifier, Breaking API & SemVer Guard, Topological PR Risk Scorer (0–100), Predictive Test Deficit Auditor, Cognitive Complexity Spike Budgeter, Semantic Changelog Synthesizer, Review Memory & Delta Tracker, Automated Git Patch Synthesizer, Git Notes Annotator, Universal Platform Publisher (GitHub, GitLab, Bitbucket, Azure DevOps, Gitea, Gerrit, Webhooks), SARIF 2.1.0 Security Exporter, and Interactive Terminal Review Playground** for software engineers, platform architects, and autonomous AI pair-programming agents.

```
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                               Developer / CI / AI Agent CLI Invocation                                 │
│        # Universal Git Diff Sources (Branch, Staged, Commit, Patch, Piped Diff)                        │
│        cartograph ai-review --branch "origin/main...HEAD"                                              │
│        cartograph ai-review --source <branch> --target <branch>                                        │
│        cartograph ai-review --staged              # Review staged changes prior to git commit          │
│        cartograph ai-review --commit <sha>        # Review a specific commit                           │
│        git diff | cartograph ai-review -          # Review diff piped from stdin / patch file          │
│                                                                                                        │
│        # Intelligence, Risk Scoring & Changelog Flags                                                  │
│        cartograph ai-review --summarize           # Synthesize AST-grounded PR description             │
│        cartograph ai-review --changelog           # Synthesize Keep-a-Changelog Markdown               │
│        cartograph ai-review --only-new            # Filter to net-new findings (Review Memory)         │
│        cartograph ai-review --apply-patch         # Safe single-shot git apply of suggested fixes      │
│                                                                                                        │
│        # Execution Modes: 100% Offline Deterministic vs. Multi-Provider AI                             │
│        cartograph ai-review --offline             # 100% local, air-gapped, zero-LLM (<50ms)           │
│        cartograph ai-review --provider anthropic --model claude-3-7-sonnet                             │
│        cartograph ai-review --provider openai --model gpt-4o                                           │
│        cartograph ai-review --provider gemini --model gemini-2.5-flash                                 │
│        cartograph ai-review --provider ollama --model qwen2.5-coder                                    │
│                                                                                                        │
│        # Pluggable Discussion & Publishing Layer (Vendor-Agnostic)                                     │
│        cartograph ai-review --export-patch review.patch      # Executable git patch with suggestions   │
│        cartograph ai-review --publish git-notes              # Native git notes attached to commit     │
│        cartograph ai-review --publish webhook --webhook-url https://...                                │
│        cartograph ai-review --publish github                 # Auto-detected in GitHub Actions         │
│        cartograph ai-review --publish gitlab                 # Auto-detected in GitLab CI              │
│        cartograph ai-review --format sarif > results.sarif   # OASIS SARIF 2.1.0 standard              │
│        cartograph ai-review -i                               # Interactive TUI review playground       │
└───────────────────────────────────────────────────┬────────────────────────────────────────────────────┘
                                                    │
                                                    ▼
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                   Cartograph Structural Context Slicing Pipeline (<50ms)                               │
│                                                                                                        │
│   1. Git Diff Source Ingestion   ──► Multi-Git polyrepo resolver, branch range, staged, or stdin diff  │
│   2. Diff-to-AST Mapper          ──► Maps changed hunks to exact AST nodes and symbol spans            │
│   3. SemVer & Public API Guard   ──► Detects breaking signatures, parameter changes, visibility drops  │
│   4. 1st & 2nd-Hop Caller Slicer ──► Traverses AST call graph; extracts caller call sites & snippets   │
│   5. Downstream Side-Effect Audit──► Checks for unhandled error variants, nullability & state drift    │
│   6. Business Invariant Delta    ──► Analyzes relaxed or deleted validation rules (Phase 3.1)          │
│   7. Predictive Test Deficit     ──► Flags modified logic lacking test coverage (Phase 2.2)            │
│   8. Arch & Deadcode Enforcer    ──► Catches boundary leaks (Phase 2.4) & zombie additions (Phase 2.6) │
│   9. Complexity Spike Detector   ──► Flags cognitive complexity spikes > 15 (Phase 2.5)                │
│  10. Topological PR Risk Score   ──► PageRank centrality & transitive blast radius score (0–100)       │
└───────────────────────────────────────────────────┬────────────────────────────────────────────────────┘
                                                    │
                                                    ▼
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                          Dual Synthesis Engine (Deterministic vs. AI)                                  │
│                                                                                                        │
│   ┌──────────────────────────────────────────────┐  ┌──────────────────────────────────────────────┐   │
│   │    Offline Deterministic Engine (<10ms)      │  │      AI-Augmented Synthesis Engine (<2s)     │   │
│   │  • Zero-LLM, 100% air-gapped & local         │  │  • Multi-Provider Client (Claude, GPT,       │   │
│   │  • Rules-based contract verification         │  │    Gemini, Ollama, LiteLLM)                  │   │
│   │  • Algorithmic side-effect classification    │  │  • Hyper-compact Graph Context (<1.2k tokens)│   │
│   │  • Template-driven fix synthesis             │  │  • Deep semantic reasoning & patch synthesis │   │
│   └──────────────────────────────────────────────┘  └──────────────────────────────────────────────┘   │
└───────────────────────────────────────────────────┬────────────────────────────────────────────────────┘
                                                    │
                                                    ▼
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                         Pluggable Discussion & Universal Publishing Layer                              │
│   • Executable Git Patch:  Generates `review.patch` applicable via `git apply review.patch`            │
│   • Native Git Notes:      Attaches review comments directly to commit objects in `refs/notes/reviews` │
│   • Interactive TUI:       Split-pane terminal navigator with 1-click patch acceptance & clipboard     │
│   • ANSI Terminal:         Colorized severity badges (🚨 Blocker, ⚠️ Warning, 🧪 Test Deficit), links    │
│   • Generic Webhook:       Dispatches structured JSON payloads to Slack, Discord, or internal APIs     │
│   • OASIS SARIF 2.1.0:     Standard ingested by GitHub Code Scanning, GitLab SAST, Azure DevOps, IDEs  │
│   • Platform Adapters:     Optional publishers for GitHub PRs, GitLab MRs, Bitbucket, Azure DevOps     │
└────────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Architectural Philosophy: De-coupling from Vendors (MR is Just a Branch + Discussions)

A core flaw of modern code review tools is **vendor lock-in**. Many tools treat a "Pull Request" or "Merge Request" as a proprietary construct tightly coupled to GitHub or GitLab REST APIs.

### The Ground Truth of Code Review:
1. **A Pull Request / Merge Request is simply a Git Branch Delta**:
   $$\Delta = \text{Git Diff}(\text{base}, \text{head})$$
   Whether changes live on GitHub, GitLab, Bitbucket, Azure DevOps, Gerrit, Gitea, or a local working tree, the diff is identical.
2. **Review Comments are Line-Anchored Annotations**:
   A review finding is simply a tuple:
   $$(\text{File}, \text{LineRange}, \text{DiffHunk}, \text{Severity}, \text{Description}, \text{SuggestedPatch})$$
3. **The Discussion Layer is Pluggable**:
   Where these comments are published should be an orthogonal decision:
   - To your terminal during pre-commit or pre-push.
   - To an executable Git patch (`review.patch`).
   - Directly into the Git repository via native `git notes`.
   - To a web UI (GitHub, GitLab, Bitbucket, Azure DevOps, Gerrit) via a lightweight publisher adapter.
   - To an automated CI security scanner via standard SARIF 2.1.0.

By establishing this decoupled architecture, Cartograph works identically everywhere: in air-gapped local terminals, corporate self-hosted servers, custom CI pipelines, and public cloud platforms.

```
┌───────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                          Traditional AI Reviewer: The Diff-Only Blind Spot                            │
│                                                                                                       │
│   PR Diff in service.rs:                                                                              │
│   @@ -42,7 +42,7 @@                                                                                   │
│   - pub fn calculate_discount(order: &Order) -> Option<f64>                                           │
│   + pub fn calculate_discount(order: &Order) -> Result<f64, DiscountError>                            │
│                                                                                                       │
│   ❌ What Diff-Only LLMs See:  A simple signature refactor. "Looks good! Great use of Result."         │
│                                                                                                       │
│   🚨 What is ACTUALLY Happening Across the Codebase:                                                  │
│   • checkout_controller.rs:L108: `let disc = calculate_discount(&order).unwrap_or(0.0);`             │
│     💥 RUNTIME PANIC: Method no longer returns Option; calls unwrap_or on Result expecting Option!    │
│   • invoice_generator.rs:L64: Calls `calculate_discount`; fails to match new `DiscountError` variants.│
│   • webhook_handler.rs:L212: Silent contract breach; drops error context.                             │
│   • ZERO unit tests exist for `DiscountError::ExpiredCoupon` branch.                                  │
│                                                                                                       │
│   Traditional AI Reviewers miss 100% of these downstream breakages because they cannot see callers!   │
└───────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Universal Git Diff Ingestion & Structural Context Slicing

Rather than requiring proprietary API webhooks, `cartograph ai-review` accepts any standard Git diff input via `GitDiffSource`:

```rust
pub enum GitDiffSource {
    /// Branch comparison (e.g. origin/main...HEAD or feat...main)
    BranchRange { base: String, head: String },
    /// Local staged changes in git index (`--staged`)
    Staged,
    /// Unstaged + staged working tree changes vs HEAD
    WorkingTree,
    /// Specific commit SHA (`--commit <sha>`)
    Commit(String),
    /// Diff piped via stdin or patch file (`--patch <path>`)
    Patch(String),
}
```

```
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                              Cartograph Context Slicing Pipeline                                       │
├──────────────────────────┬─────────────────────────────────────────────────────────────────────────────┤
│ Stage                    │ Operation & Subsystem                                                       │
├──────────────────────────┼─────────────────────────────────────────────────────────────────────────────┤
│ 1. Git Diff Mapping      │ Resolves modified files and line ranges from `GitDiffSource`. Supports      │
│                          │ multi-git polyrepo workspaces via `GitWorkspaceResolver`.                   │
├──────────────────────────┼─────────────────────────────────────────────────────────────────────────────┤
│ 2. AST Target Binding    │ Maps diff line hunks to exact AST nodes (`Node`) in Head vs Base graphs.    │
├──────────────────────────┼─────────────────────────────────────────────────────────────────────────────┤
│ 3. SemVer Guard Audit    │ Checks `SemVerGuard`: Detects signature breaks, visibility reductions,      │
│                          │ parameter changes, and removed public exports.                              │
├──────────────────────────┼─────────────────────────────────────────────────────────────────────────────┤
│ 4. Caller Slicing (BFS)  │ Queries `cartograph-db` for 1st- and 2nd-hop callers. Locates exact         │
│                          │ call sites and extracts 5–8 lines of caller context around each invocation. │
├──────────────────────────┼─────────────────────────────────────────────────────────────────────────────┤
│ 5. Side-Effect Analyzer  │ Inspects caller call sites for unhandled errors, nullability mismatches,    │
│                          │ and precondition violations.                                                │
├──────────────────────────┼─────────────────────────────────────────────────────────────────────────────┤
│ 6. Invariant Delta Check │ Invokes `extract_logic::diff` (Phase 3.1) to flag relaxed or deleted        │
│                          │ validation rules and modified calculation formulas.                         │
├──────────────────────────┼─────────────────────────────────────────────────────────────────────────────┤
│ 7. Test Deficit Audit    │ Invokes `test_selector::deficit` (Phase 2.2) to identify modified symbols  │
│                          │ with zero test coverage and generate scaffolding.                           │
├──────────────────────────┼─────────────────────────────────────────────────────────────────────────────┤
│ 8. Arch & Deadcode Gate  │ Invokes `lint_arch` (Phase 2.4) and `deadcode` (Phase 2.6) to detect        │
│                          │ newly introduced boundary leaks or zombie functions.                        │
├──────────────────────────┼─────────────────────────────────────────────────────────────────────────────┤
│ 9. Complexity Spike Gate │ Invokes `complexity` (Phase 2.5) to detect cognitive complexity spikes      │
│                          │ (delta > +15 or function crossing threshold > 20).                          │
├──────────────────────────┼─────────────────────────────────────────────────────────────────────────────┤
│ 10. PR Risk Scoring      │ Invokes `impact::risk_engine` (Phase 2.1) to compute objective PR risk      │
│                          │ score (0–100) based on graph centrality, caller spread, and tier boundaries.│
└──────────────────────────┴─────────────────────────────────────────────────────────────────────────────┘
```

---

## 4. Canonical Review Data Models & Intermediate Representation (IR)

All models in `cartograph-core::review` are strictly platform-agnostic, anchoring issues to canonical Git coordinates:

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use cartograph_core::model::RiskLevel;

/// Complete Review Report
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReviewReport {
    /// Identifier of the review target (branch range, commit, or patch)
    pub target_identifier: String,
    /// Base revision reference
    pub base_ref: Option<String>,
    /// Head revision reference
    pub head_ref: Option<String>,
    /// Overall review verdict
    pub verdict: ReviewVerdict,
    /// High-level executive summary of the changeset
    pub summary: String,
    /// Quantitative metrics of the review
    pub metrics: ReviewMetrics,
    /// Topological PR risk score and blast radius assessment
    pub risk_assessment: PrRiskAssessment,
    /// Optional Keep-a-Changelog Markdown and semantic summary
    pub changelog_summary: Option<PrChangelogSummary>,
    /// Actionable findings tied to canonical Git diff coordinates
    pub findings: Vec<ReviewFinding>,
    /// Cognitive complexity deltas on modified symbols
    pub complexity_deltas: Vec<ComplexityDeltaFinding>,
    /// Mode used for the review
    pub execution_mode: ReviewExecutionMode,
    /// Analysis duration in milliseconds
    pub analysis_duration_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewVerdict {
    /// Changes are safe, contracts preserved, tests present
    Approve,
    /// Non-blocking suggestions or informational notes
    Comment,
    /// Breaking contracts, unhandled side-effects, or test deficit
    RequestChanges,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ReviewMetrics {
    pub files_changed: usize,
    pub symbols_modified: usize,
    pub breaking_contract_count: usize,
    pub unhandled_side_effect_count: usize,
    pub untested_symbol_count: usize,
    pub arch_violation_count: usize,
    pub invariant_relaxation_count: usize,
    pub deadcode_count: usize,
    pub complexity_spike_count: usize,
    pub total_blockers: usize,
    pub total_warnings: usize,
    pub total_infos: usize,
}

/// Topological PR Risk Score and Blast Radius Assessment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrRiskAssessment {
    /// Composite risk score from 0.0 (negligible) to 100.0 (critical system impact)
    pub score: f64,
    /// Categorical risk tier
    pub level: RiskLevel,
    /// Graph centrality metric (0.0 to 1.0) of modified symbols (PageRank / Degree)
    pub graph_centrality_percentile: f64,
    /// Total number of downstream transitive symbols impacted
    pub total_impacted_symbols: usize,
    /// Number of architectural boundaries crossed by the blast radius
    pub cross_boundary_impact_count: usize,
    /// High-risk hotspot symbols modified
    pub hotspot_symbols: Vec<String>,
}

/// Semantic PR Description & Keep-a-Changelog Block
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrChangelogSummary {
    pub title: String,
    pub pr_overview: String,
    pub api_surface_changes: Vec<String>,
    pub business_rule_deltas: Vec<String>,
    pub changelog: KeepAChangelog,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct KeepAChangelog {
    pub added: Vec<String>,
    pub changed: Vec<String>,
    pub deprecated: Vec<String>,
    pub removed: Vec<String>,
    pub fixed: Vec<String>,
    pub security: Vec<String>,
}

/// Cognitive Complexity Delta on modified symbols
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComplexityDeltaFinding {
    pub symbol_name: String,
    pub file_path: PathBuf,
    pub line: usize,
    pub base_complexity: u32,
    pub head_complexity: u32,
    pub delta: i32,
    pub exceeds_threshold: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReviewFinding {
    /// Deterministic SHA-256 fingerprint of file, line, category, and issue
    pub id: String,
    /// Category of the issue
    pub category: FindingCategory,
    /// Severity level
    pub severity: FindingSeverity,
    /// Target file path relative to workspace root
    pub file_path: PathBuf,
    /// Exact line span in the file (1-indexed)
    pub line_span: FindingSpan,
    /// Git Diff side: Right (Head/New) or Left (Base/Old)
    pub diff_side: DiffSide,
    /// Optional commit SHA associated with the hunk
    pub commit_sha: Option<String>,
    /// Concise finding headline
    pub title: String,
    /// Detailed Markdown explanation of the risk and root cause
    pub description: String,
    /// Structural evidence extracted from the AST graph
    pub structural_evidence: StructuralEvidence,
    /// AI-generated narrative reasoning (if LLM mode enabled)
    pub ai_reasoning: Option<String>,
    /// Concrete suggested fix with code replacement block
    pub suggested_fix: Option<SuggestedFix>,
    /// Whether this finding was previously acknowledged via review memory or code pragma
    pub is_acknowledged: bool,
    /// Optional reason for suppression/acknowledgement
    pub ack_reason: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingCategory {
    /// Breaking public API contract or SemVer violation
    BreakingContract,
    /// Downstream caller failing to handle altered callee behavior
    UnhandledSideEffect,
    /// Modified business logic or public symbol lacking tests
    TestDeficit,
    /// Layer boundary breach or cyclic dependency introduced
    ArchViolation,
    /// Business validation rule relaxed or deleted
    InvariantRelaxation,
    /// Newly introduced dead code or zombie components
    DeadCode,
    /// Extreme cyclomatic/cognitive complexity hotspot introduced
    ComplexityHotspot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingSeverity {
    Info,
    Warning,
    Blocker,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiffSide {
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FindingSpan {
    pub start_line: usize,
    pub end_line: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SuggestedFix {
    /// Replacement code snippet
    pub replacement_code: String,
    /// Unified diff format representation ready for `git apply`
    pub unified_diff_patch: String,
    /// Explanation of why this fix resolves the hazard
    pub explanation: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StructuralEvidence {
    ContractBreak {
        symbol_name: String,
        is_public: bool,
        old_signature: Option<String>,
        new_signature: Option<String>,
        break_reason: String,
    },
    DownstreamHazard {
        callee_symbol: String,
        caller_symbol: String,
        caller_file: PathBuf,
        caller_line: usize,
        caller_snippet: String,
        hazard_type: DownstreamHazardType,
    },
    TestDeficit {
        untested_symbol: String,
        suggested_test_file: PathBuf,
        suggested_test_scaffold: String,
    },
    ArchViolation {
        from_layer: String,
        to_layer: String,
        rule_name: String,
    },
    RuleRelaxation {
        rule_title: String,
        old_predicate: String,
        new_predicate: String,
    },
    DeadCode {
        symbol_name: String,
        symbol_kind: String,
    },
    ComplexitySpike {
        symbol_name: String,
        base_complexity: u32,
        head_complexity: u32,
        delta: i32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DownstreamHazardType {
    UnhandledErrorVariant { variant: String },
    NullabilityDrift,
    PreconditionViolation { condition: String },
    CallSignatureMismatch,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ReviewExecutionMode {
    OfflineDeterministic,
    AiAugmented {
        provider: String,
        model: String,
        prompt_tokens: usize,
        completion_tokens: usize,
    },
}
```

---

## 5. Dual Review Engine: Offline Deterministic vs. AI-Augmented

Cartograph provides two execution engines:

```
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                   Dual Review Engine Architecture                                      │
├───────────────────────────────────┬────────────────────────────────────────────────────────────────────┤
│ Dimension                         │ Mode A: Offline Deterministic Engine   │ Mode B: AI-Augmented Engine   │
├───────────────────────────────────┼────────────────────────────────────────┼───────────────────────────┤
│ Execution Speed                   │ **Sub-50ms** across entire diffs       │ **1.5s - 3.5s** (API bound) │
│ Cost                              │ **$0.00** (Free, Zero tokens)          │ Minimal (~1.2k tokens/diff│
│ Network / Air-Gap                 │ **100% Local**, Zero network traffic    │ HTTPS to Provider API     │
│ Verification Method               │ AST graph rules, SemVer checks,       │ LLM deep reasoning over   │
│                                   │ call-site regex/AST flow verification  │ structured AST context    │
│ Suggested Fixes                   │ Algorithmic templates & patches        │ Contextual code patches   │
│ LLM Hallucinations                │ **Zero (Mathematically impossible)**   │ Suppressed via AST facts  │
│ Recommended For                   │ Local pre-commit, pre-push, air-gapped │ Pull Request discussions, │
│                                   │ CI quality gates, corporate compliance │ comprehensive code review │
└───────────────────────────────────┴────────────────────────────────────────┴───────────────────────────┘
```

### 5.1 Mode A: Offline Deterministic Engine (`--offline` / `--no-llm`)
- Operates locally with **zero external network requests** in sub-50ms.
- Evaluates SemVer breaks on public exports, call-site unhandled `Result`/`Option` hazards, test deficits, and architectural layer violations.
- Computes topological PR risk score (0–100) using PageRank centrality and caller counts.
- Flags cognitive complexity spikes exceeding configured thresholds.
- Produces deterministic finding descriptions, unified diff suggestions, and test scaffolds.

### 5.2 Mode B: AI-Augmented Synthesis Engine (`--provider <name>`)
- Compiles a hyper-compact AST graph slice into XML (<1,500 tokens).
- Sends to configured provider (Anthropic, OpenAI, Gemini, Ollama, LiteLLM).
- Generates natural language reasoning, contextual unified patches, and Keep-a-Changelog Markdown while strictly anchored to AST graph facts.

---

## 6. Review Memory & Multi-Commit Delta Tracking

On multi-commit feature branches, developers push incremental changes. Traditional bots spam repeat notifications on findings the developer already knows about or intentionally accepted.

### 6.1 Deterministic Finding Fingerprinting
Each finding is fingerprinted via SHA-256:
$$\text{FindingHash} = \text{SHA256}(\text{Category}, \text{SymbolName}, \text{FilePath}, \text{HazardDetails})$$

- Stored locally in `.cartograph/review_history.json` or `.cartograph/cartograph.db`.
- Findings resolved in subsequent commits are marked `Resolved`.
- Existing unaddressed findings are marked `Acknowledged` and silenced when `--only-new` is passed.

### 6.2 Code Pragmas & In-Source Suppressions
Developers can silence specific findings directly in code with an explicit reason:
```rust
// cartograph:ignore(breaking-contract) Reason: Major v2 refactor approved in RFC-40
pub fn process_payment(req: PaymentRequest) -> Result<Receipt, Error> { ... }
```
```python
# cartograph:ignore(untested) Reason: Covered by end-to-end integration test
def calculate_vat_rate(country_code: str) -> float: ...
```

### 6.3 Review Iteration Delta & Lifecycle Tracking (`--verify-fixes`)
On multi-commit PRs, when authors push new commits in response to review comments, Cartograph tracks iteration progress:
$$\Delta_{\text{iteration}} = \text{Compare}(\text{Findings}_{N-1}, \text{Findings}_N)$$
- **Resolved Findings**: Active in round $N-1$, but resolved in round $N$.
- **Unresolved Findings**: Still present in round $N$.
- **New Regressions**: Introduced in the latest push, not present in round $N-1$.

Exposed in CLI ANSI tables, Markdown summaries, and terminal output:
```text
  ─── Review Iteration Delta (vs `commit-abc`) ──────────────────────────────
  ✔ 2 findings resolved by author
  ⚠️ 1 finding unresolved
  ✔ 0 new regressions introduced
```

---

## 7. Pluggable Discussion, Git Notes & Universal Publishing Layer

Publishing discussions and comments is completely decoupled from the analysis engine behind a clean `ReviewPublisher` trait:

```rust
pub trait ReviewPublisher: Send + Sync {
    /// Publish or export the review findings to the destination
    fn publish(&self, report: &ReviewReport) -> Result<PublishReceipt, String>;
}
```

```
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                              Universal Publisher Ecosystem                                             │
├──────────────────────────┬─────────────────────────────────────────────────────────────────────────────┤
│ Publisher                │ Mechanism & Description                                                     │
├──────────────────────────┼─────────────────────────────────────────────────────────────────────────────┤
│ 1. Executable Git Patch  │ Exports `review.patch` containing all suggested fixes as standard unified   │
│    (`--export-patch`)    │ diffs. Anyone can inspect with `git diff` and apply via `git apply`.        │
├──────────────────────────┼─────────────────────────────────────────────────────────────────────────────┤
│ 2. Native Git Notes      │ Attaches review comments directly to the commit object via native           │
│    (`--publish git-notes`) `refs/notes/cartograph-review`. Comments travel with the repo, 100% offline.│
├──────────────────────────┼─────────────────────────────────────────────────────────────────────────────┤
│ 3. Interactive TUI       │ Split-pane terminal navigator with 1-click patch acceptance & clipboard.    │
├──────────────────────────┼─────────────────────────────────────────────────────────────────────────────┤
│ 4. Standard Markdown     │ Platform-neutral Markdown suitable for pasting anywhere (Jira, email, PR).  │
├──────────────────────────┼─────────────────────────────────────────────────────────────────────────────┤
│ 5. OASIS SARIF 2.1.0     │ Standard JSON for security tabs in GitHub, GitLab, Azure DevOps, SonarQube. │
├──────────────────────────┼─────────────────────────────────────────────────────────────────────────────┤
│ 6. Generic Webhook       │ POSTs structured JSON report to any webhook URL (Slack, Discord, internal). │
├──────────────────────────┼─────────────────────────────────────────────────────────────────────────────┤
│ 7. Platform Adapters     │ Lightweight HTTP adapters for GitHub PRs, GitLab MRs, Bitbucket, Gitea.     │
└──────────────────────────┴─────────────────────────────────────────────────────────────────────────────┘
```

### 7.1 Safe One-Step Patch Application (`--apply-patch`)
To prevent infinite or recursive auto-repair loops, Cartograph provides **safe, deterministic single-shot patch application**:
```bash
# Reviews changes and applies the generated unified diff patch cleanly in one step
cartograph ai-review --apply-patch
```
- Performs a dry-run first (`git apply --check`).
- If conflicts exist, it aborts cleanly without mutating the working directory and leaves `review.patch` for manual inspection.
- In the interactive TUI (`-i`), pressing `[A]` applies the patch for the highlighted finding with instant visual confirmation.

### 7.2 Native 1-Click Multi-Line GitHub & GitLab Suggestions
Rather than requiring authors to copy/paste diff snippets, Cartograph formats suggested remediations using native GitHub/GitLab suggestion markdown:
````markdown
```suggestion
let discount = calculate_discount(&order)?;
```
````
In the GitHub and GitLab PR conversation UI, authors can accept Cartograph's suggested fix with **a single button click** ("Commit suggestion").

### 7.3 Fast-Path Pre-Commit Hook Mode (`--fast` / `<10ms`)
For pre-commit and pre-push Git hooks, `--fast` executes purely deterministic 1st-hop contract verification (SemVer breaking changes, removed exports, visibility reductions) while bypassing LLM synthesis and deep transitive graph sweeps.
```bash
# Recommended .git/hooks/pre-commit
cartograph ai-review --staged --fast --fail-on-breaking
```

---

## 8. Interactive TUI Review Playground (`cartograph ai-review -i`)

For developers reviewing changes locally (working tree, staged, or branch) before pushing:

```
┌── Cartograph AI Review: feat/checkout-v2 (PR Risk Score: 74/100 - High Blast Radius) ─────────────────┐
│ Findings (3 Blockers, 2 Warnings)         │ Detail View: [🚨 BLOCKER] Unhandled Side-Effect            │
│───────────────────────────────────────────│───────────────────────────────────────────────────────────│
│ ▼ src/billing/pricing.rs                  │ Symbol: PricingService::calculate_discount               │
│   🚨 [Contract] Signature changed to Resul│ Location: src/controllers/order_controller.rs:88          │
│   🧪 [Test] calculate_discount untested   │ Blast Radius: 14 downstream callers across 3 modules      │
│   ⚠️ [Complexity] Spike: +18 (now 32)      │                                                           │
│ ▼ src/controllers/order_controller.rs     │ Hazard: Callee now returns Result<f64, DiscountError>.    │
│   🚨 [Side-Effect] Unhandled Result at L88│ Caller invokes .unwrap_or(0.0) expecting Option.          │
│   ⚠️ [Arch] Direct import of DataAccess   │                                                           │
│ ▼ src/models/order.rs                     │ Downstream Call-Site Code:                                │
│   ⚠️ [Invariant] Discount clamp removed   │  86:   pub async fn submit(&self, order: Order) {         │
│                                           │  87:       let user = self.auth.current_user()?;          │
│                                           │ >88:       let disc = pricing.calculate_discount(&order)  │
│                                           │  89:                          .unwrap_or(0.0);            │
│                                           │  90:       order.apply_discount(disc);                    │
│                                           │                                                           │
│                                           │ Suggested Patch:                                          │
│                                           │ - let disc = pricing.calculate_discount(&order).unwrap... │
│                                           │ + let disc = pricing.calculate_discount(&order)?          │
│                                           │                                                           │
│                                           │ [A] Apply Patch  [C] Copy Patch  [O] Open in Editor        │
└───────────────────────────────────────────┴───────────────────────────────────────────────────────────┘
 [Tab] Switch Pane  [↑/↓] Navigate  [Enter] Expand  [O] Open File  [A] Apply Patch  [Q] Quit
```

---

## 9. Configuration Specification (`cartograph.toml`)

All parameters are configured in the `[ai_review]` section of `cartograph.toml`:

```toml
[ai_review]
# Default execution mode: "offline" or "ai"
mode = "ai"

# AI Provider: "anthropic", "openai", "gemini", "ollama", "custom"
provider = "anthropic"

# Model name
model = "claude-3-7-sonnet"

# Temperature for review generation
temperature = 0.1

# Fallback to offline deterministic engine if API fails or times out
fallback_to_offline = true

# Default git comparison target if omitted
default_target = "origin/main"

# CI/CD Quality Gating: fail with exit code 1 if any of these occur
fail_on = ["blocker"]
fail_on_breaking = true
fail_on_side_effects = true
fail_on_untested = false
min_test_coverage = 0.80

# Maximum tolerable PR Risk Score (0-100) before blocking
max_risk_score = 80.0

# Cognitive complexity delta threshold (+15 spike)
max_complexity_delta = 15

# Review memory: whether to track historical findings across commits
review_memory = true
history_path = ".cartograph/review_history.json"
fast = false
verify_fixes = false

# Pluggable publishers configuration
[ai_review.publishers]
git_notes = false
webhook_url = ""

[ai_review.publishers.github]
auto_detect = true
inline_comments = true
step_summary = true

[ai_review.publishers.gitlab]
auto_detect = true
inline_comments = true
```

---

## 10. CLI Command Specification

```bash
# Review current working tree against default branch (auto-detects origin/main or master)
cartograph ai-review

# Review staged changes prior to running git commit
cartograph ai-review --staged
cartograph ai-review --staged --fast                 # Ultra-fast (<10ms) pre-commit hook mode

# Review explicit branch comparison
cartograph ai-review --branch "origin/main...HEAD"
cartograph ai-review --source feat/checkout --target main

# Review a specific commit
cartograph ai-review --commit 4a3f91b

# Review a diff piped from stdin or patch file
git diff | cartograph ai-review -
cartograph ai-review --patch my_changes.patch

# Intelligence, Risk Scoring, Iteration Tracking & Changelog Flags
cartograph ai-review --summarize                     # Synthesize AST PR description
cartograph ai-review --changelog                     # Synthesize Keep-a-Changelog Markdown
cartograph ai-review --only-new                      # Filter to net-new findings
cartograph ai-review --verify-fixes                  # Iteration delta: resolved findings vs new regressions
cartograph ai-review --apply-patch                   # Safe single-shot git apply

# 100% Offline review (Zero LLM, sub-50ms, air-gapped)
cartograph ai-review --offline

# Specify AI Provider and Model
cartograph ai-review --provider anthropic --model claude-3-7-sonnet
cartograph ai-review --provider openai --model gpt-4o
cartograph ai-review --provider gemini --model gemini-2.5-flash
cartograph ai-review --provider ollama --model qwen2.5-coder

# CI/CD Quality Gates
cartograph ai-review --fail-on blocker
cartograph ai-review --fail-on-breaking --fail-on-side-effects
cartograph ai-review --fail-on-risk 80
cartograph ai-review --fail-on-complexity-spike 15
cartograph ai-review --min-coverage 0.80

# Pluggable Publishing & Exporting
cartograph ai-review --export-patch review.patch      # Generate executable git patch
cartograph ai-review --publish git-notes              # Record review in git notes
cartograph ai-review --publish webhook --webhook-url https://api.company.com/review
cartograph ai-review --publish github                 # Post to GitHub PR
cartograph ai-review --publish gitlab                 # Post to GitLab MR

# Output Formats
cartograph ai-review --format terminal   # Default colorized ANSI with links
cartograph ai-review --format markdown   # Universal Markdown
cartograph ai-review --format json       # Machine JSON
cartograph ai-review --format sarif      # OASIS SARIF 2.1.0

# Interactive TUI Review Playground
cartograph ai-review -i
```

---

## 11. Implementation Roadmap & Milestones

The implementation of Phase 3.2 is structured into 5 cohesive milestones:

### Milestone 1: Review IR, Models & Offline Contract Analyzer (`cartograph-core` & `cartograph-engine`)
- Define `ReviewReport`, `ReviewFinding`, `FindingCategory`, `FindingSeverity`, `SuggestedFix`, `GitDiffSource`, `ReviewVerdict`, `PrRiskAssessment`, `PrChangelogSummary`, `ComplexityDeltaFinding` in `cartograph-core::review`.
- Add `AiReviewConfig` to `cartograph-core::config`.
- Implement `OfflineContractAnalyzer` in `cartograph-engine::ai_review::contract_analyzer`:
  - Integrates `SemVerGuard` for public API signature breaking detection.
  - Implements AST signature difference extraction (parameter additions, return changes, visibility).
  - Algorithmic generation of `SuggestedFix` unified diff patches.
- Unit tests validating deterministic SemVer and contract break detection across Rust, TS, Python, Go.

### Milestone 2: Downstream Side-Effect, PR Risk & Complexity Pipeline (`cartograph-engine`)
- Implement `DownstreamSideEffectAnalyzer` in `cartograph-engine::ai_review::side_effect_analyzer`:
  - Traverses `cartograph-db` for 1st- and 2nd-hop callers of modified symbols.
  - Analyzes caller call sites for unhandled `Result`/`Option` changes, arity drift, and precondition breaks.
  - Extracts caller source snippets ($\pm 4$ lines around call site).
- Connect `impact::risk_engine` for PR Risk Score (0–100) and blast radius assessment.
- Connect `complexity` for cognitive complexity spike detection ($\Delta CC$).
- Connect `test_selector::deficit` for automated test gap identification.
- Connect `lint_arch` for layer boundary check integration.
- Connect `extract_logic::diff` for business rule relaxation detection.
- Implement Review Memory cache (`.cartograph/review_history.json`) and pragma suppression parsing (`// cartograph:ignore(...)`).

### Milestone 3: Polyglot LLM Client, Context Compiler & Changelog Synthesizer (`cartograph-engine`)
- Implement `ReviewContextCompiler` generating compact `<review_context>` XML blocks (<1,500 tokens).
- Implement async `LlmClient` in `cartograph-engine::ai_review::llm`:
  - Supports Anthropic, OpenAI, Google Gemini, Ollama, and OpenAI-compatible endpoints.
  - Handles rate-limiting, retries, and network timeout handling.
  - Enforces JSON output format with schema validation and automatic fallback to offline engine.
- Implement `ChangelogSynthesizer` generating Keep-a-Changelog Markdown and PR description (`--summarize`, `--changelog`).
- Implement safe one-step patch applier (`--apply-patch`) with conflict pre-check.

### Milestone 4: Pluggable Discussion, Git Notes & Universal Publishers (`cartograph-engine`)
- Implement `ReviewPublisher` trait with decoupled publishers:
  - `PatchExporter`: Writes `review.patch` applicable via `git apply`.
  - `GitNotesPublisher`: Commits review findings into `refs/notes/cartograph-review`.
  - `TerminalFormatter`: Colorized ANSI badges, clickable OSC-8 links.
  - `MarkdownFormatter`: Universal Markdown report.
  - `SarifFormatter`: OASIS SARIF 2.1.0 JSON.
  - `WebhookPublisher`: POSTs structured JSON payloads.
  - `PlatformAdapters`: Lightweight adapters for GitHub, GitLab, Bitbucket.
- Wire up CI gating flags (`--fail-on-breaking`, `--fail-on-side-effects`, `--fail-on-risk`, `--fail-on-complexity-spike`, `--fail-on-untested`, `--fail-on blocker`).

### Milestone 5: CLI Subcommand, Interactive TUI & End-to-End Test Suite (`cartograph-cli`)
- Register `ai-review` subcommand in `crates/cartograph-cli/src/main.rs` and `commands.rs`.
- Build interactive split-pane TUI review playground (`cartograph ai-review -i`) with `ratatui`.
- Write end-to-end integration tests (`tests/test_ai_review.rs`) testing offline mode, staged diffs, PR risk scoring, git notes, patch generation, and mocked LLM modes.
- Update `ROADMAP.md` and documentation.

---

## 12. Quality Assurance & Performance Benchmarks

### Performance SLAs
| Operation | Target Dataset | SLA Target | Expected Achievement |
| :--- | :--- | :--- | :--- |
| **Git Diff Ingestion & AST Mapping** | 20 files, 500 LOC changed | **<10ms** | ~3ms |
| **Downstream Caller Slicing (2-hop)** | 15 modified symbols, 50 callers | **<25ms** | ~8ms |
| **Complete Offline Review Mode** | Entire Diff (Diff + Callers + Rules + Risk) | **<100ms** | ~40ms |
| **Context XML Compilation** | Full graph slice | **<5ms** | ~1ms |
| **AI-Augmented Synthesis (LLM)** | Anthropic / OpenAI API call | **<3.0s** | ~1.8s (network bound) |
| **Patch Export / Git Notes Write** | Full report | **<5ms** | ~2ms |

### Test Suite Structure
1. **`crates/cartograph-engine/src/ai_review/tests/`**:
   - `test_contract_analyzer.rs`: Breaking public functions, removed enum variants, altered visibility across Rust, TS, Python, Go.
   - `test_side_effect_analyzer.rs`: Callers failing to handle new error variants, null dereferences, arity mismatches.
   - `test_risk_scoring.rs`: Verification of PageRank centrality and blast radius computation.
   - `test_complexity_spikes.rs`: Detection of cognitive complexity spikes on modified functions.
   - `test_review_memory.rs`: Fingerprint calculation, delta suppression, and code pragma parsing.
   - `test_changelog_synthesizer.rs`: Generation of Keep-a-Changelog Markdown from AST diffs.
   - `test_context_compiler.rs`: Verification of XML token budget and accuracy.
   - `test_offline_engine.rs`: End-to-end deterministic review execution without network.
   - `test_llm_parser.rs`: JSON response parsing, error repair, fallback logic.
2. **`crates/cartograph-engine/src/ai_review/publishers/tests/`**:
   - `test_patch_exporter.rs`: Verifies that exported `.patch` files are valid unified diffs applicable with `git apply --check`.
   - `test_git_notes.rs`: Validates reading and writing to `refs/notes/cartograph-review`.
   - `test_sarif_formatter.rs`: Validates output against official OASIS SARIF 2.1.0 JSON schema.
3. **`tests/test_ai_review.rs`**:
   - End-to-end CLI integration tests verifying `--staged`, `--offline`, `--export-patch`, `--apply-patch`, `--publish git-notes`, `--fail-on-breaking`, and CI exit codes.
