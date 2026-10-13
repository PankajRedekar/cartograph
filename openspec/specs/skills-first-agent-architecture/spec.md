# skills-first-agent-architecture Specification

## Purpose
Provides a zero-token-overhead "Skills-First" architecture for autonomous AI coding agents, including multi-IDE skill bundle scaffolding, on-demand CLI execution, closed-loop pre-commit verification gating, skill drift detection, and token ROI analytics.
## Requirements
### Requirement: 1-Click Multi-IDE Skill Installation
The system SHALL provide a CLI command `cartograph skills install` (and top-level alias `cartograph install-skills`) that auto-detects the repository environment and scaffolds standardized `SKILL.md` bundles and IDE rule files across Claude Code, Cursor, Windsurf, Google Antigravity, Aider, Roo Code, and Cline in under 15ms without network calls.

#### Scenario: Installing all skills with auto-detection
- **WHEN** `cartograph skills install` is executed in a repository with detected IDE configurations
- **THEN** cartograph scaffolds `.agent/skills/` definitions and configures rule files (`CLAUDE.md`, `.cursorrules`, `.windsurfrules`, `.agents/skills/`) for all detected platforms.

#### Scenario: Installing tailored skills for a specific target IDE
- **WHEN** `cartograph skills install --target claude` is executed
- **THEN** cartograph writes `.agent/skills/` templates and configures `CLAUDE.md` with guidelines and custom slash commands.

#### Scenario: Non-interactive agent turn-1 self-installation
- **WHEN** `cartograph skills install --yes --preset standard` is executed non-interactively
- **THEN** cartograph accepts all defaults without prompting and generates standard skill bundles.

#### Scenario: Dry-run preview
- **WHEN** `cartograph skills install --dry-run` is executed
- **THEN** cartograph previews all file creations and modifications without altering the filesystem.

### Requirement: Closed-Loop Pre-Commit Verification Gate
The system SHALL provide an atomic command `cartograph verify` that verifies AST syntax, checks architectural boundaries and circular dependencies, detects breaking SemVer contract regressions, and predictively executes affected unit and integration tests in under 200ms.

#### Scenario: Successful pre-commit verification on staged changes
- **WHEN** `cartograph verify --staged` is executed on staged git modifications with valid architecture and passing tests
- **THEN** cartograph outputs `<verification_gate status="PASSED">` and exits with status code 0.

#### Scenario: Verification failure on breaking contract change
- **WHEN** `cartograph verify --staged` is executed on code introducing an unhandled return type or signature break
- **THEN** cartograph outputs `<verification_gate status="FAILED">` with specific `<issue>` details, remediation advice, and exits with status code 1.

### Requirement: Skill Drift Detection and Automated Upgrade
The system SHALL embed version and content checksum hashes into `SKILL.md` frontmatter, detect when installed skills have drifted from newer engine templates, and perform intelligent 3-way merge upgrades preserving user customizations.

#### Scenario: Detecting drifted skill templates
- **WHEN** `cartograph skills check` is executed in a repository with outdated skill definitions
- **THEN** cartograph reports the list of drifted skills and prompts to update.

#### Scenario: Performing smart 3-way update
- **WHEN** `cartograph skills update` is executed
- **THEN** cartograph upgrades parameter flags and checklists to match the running binary while preserving content inside user customization blocks.

#### Scenario: Auto-remediation of missing files
- **WHEN** `cartograph skills check --fix` is executed
- **THEN** cartograph repairs missing skill files, refreshes stale SQLite indexes, and restores rule bindings.

### Requirement: Token ROI and Productivity Analytics
The system SHALL track local query telemetry in SQLite and report prompt tokens saved, full-file reads avoided, and redundant test execution minutes saved.

#### Scenario: Querying agent efficiency metrics
- **WHEN** `cartograph skills stats` is executed
- **THEN** cartograph displays total prompt tokens saved, avoided file reads, and test minutes saved in formatted text or JSON.

#### Scenario: Exporting metrics as XML
- **WHEN** `cartograph skills stats --format xml` is executed
- **THEN** cartograph emits `<agent_roi_stats>` containing exact token savings and quality gate counters.

