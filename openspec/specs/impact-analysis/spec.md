## Purpose
Performs blast-radius impact analysis across symbols, files, git branches, or pull requests to map downstream dependencies, classify semantic AST diffs, detect architectural coupling smells, enforce SemVer 2.0 release policies, and flag potential breaking changes.

## Requirements

### Requirement: Symbol and File Blast-Radius Mapping
The system SHALL traverse incoming and outgoing graph edges from a modified symbol or file to list all affected downstream components, views, models, and tests.

#### Scenario: Calculating impact for a modified database model
- **WHEN** `cartograph impact "AccrualPeriod"` is run
- **THEN** cartograph outputs all views, serializers, background tasks, and unit tests that consume `AccrualPeriod`.

### Requirement: Git Diff Branch Impact Analysis
The system SHALL analyze modified files and symbols in a git branch diff against the target branch to map full-stack blast radius.

#### Scenario: Analyzing uncommitted changes or branch diff
- **WHEN** `cartograph impact --branch "origin/main...HEAD"` is executed
- **THEN** cartograph computes the collective blast radius of all changed symbols across the diff.

### Requirement: AST Semantic Change Classification
The system SHALL compare pre-edit and post-edit AST structures to differentiate docstring edits, body edits, visibility reductions, and signature breaks.

#### Scenario: Editing docstrings only
- **WHEN** a diff alters only comments or docstrings in a public function
- **THEN** cartograph classifies the change as `DocstringOnly` and sets the risk score to 0.

### Requirement: Automated SemVer Impact Verification
The system SHALL audit public exports across supported languages against `--verify-semver <major|minor|patch>`.

#### Scenario: Introducing a breaking public change on a minor release branch
- **WHEN** `cartograph impact --branch "main...HEAD" --verify-semver minor` is run on a branch modifying a public function signature
- **THEN** cartograph identifies the breaking change, reports the SemVer violation, and exits with code 1.

### Requirement: Exact Call-Site Pinpointing and AI Remediation
The system SHALL extract exact file:line:column coordinates of invocations within callers and synthesize remediation hints for signature breaks.

#### Scenario: Pinpointing call-site coordinates in callers
- **WHEN** a function signature adds a mandatory argument
- **THEN** cartograph outputs the caller's invocation line and specifies the required fix.

### Requirement: Architectural Coupling Smell Detection
The system SHALL detect high in-degree symbols fanning out across multiple disparate packages and emit Interface Segregation Principle refactoring advice.

#### Scenario: Detecting an architectural God-Object
- **WHEN** a modified symbol has in-degree >= 10 across >= 4 subsystems
- **THEN** cartograph outputs an Architectural Coupling Insight box with interface decoupling suggestions.
