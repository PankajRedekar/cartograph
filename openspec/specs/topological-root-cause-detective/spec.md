# Topological Root-Cause Detective (`cartograph diagnose`) Specification

## Purpose
Provides polyglot stack trace parsing, AST graph frame anchoring, backward variable def-use provenance slicing, topological fault localization, automated regression test reproducer synthesis, and executable git patch generation.

## Requirements

### Requirement: Polyglot Crash Log Ingestion
The system SHALL ingest and parse stack traces from Python, TypeScript/JavaScript, Rust, Go, Java, and Sentry/Datadog APM JSON formats, handling container path prefixes and source maps.

#### Scenario: Diagnosing Python traceback
- **WHEN** `cartograph diagnose "Traceback (most recent call last):\n  File \"app/service.py\", line 42, in process\n    val = obj.field\nAttributeError: 'NoneType' object has no attribute 'field'"` is executed
- **THEN** cartograph extracts the frames, normalizes the file path, and anchors each frame to an AST node.

#### Scenario: Ingesting piped test runner output
- **WHEN** crash output is piped into `cartograph diagnose -`
- **THEN** cartograph streams stdin, detects the stack trace pattern, and initiates diagnosis.

#### Scenario: Ingesting Sentry JSON event
- **WHEN** `cartograph diagnose --sentry event.json` is executed
- **THEN** cartograph extracts the exception type, value, and stack frames from the JSON payload.

### Requirement: Topological Fault Localization & Suspicion Scoring
The system SHALL compute a mathematical suspicion score (0.0 to 1.0) for every frame in the call path, distinguishing the true root-cause from innocent intermediate propagators and the leaf panic site.

#### Scenario: Pinpointing origin of None/Nil value
- **WHEN** a function returns an unexpected null value that crashes several frames deeper
- **THEN** cartograph traces backward def-use provenance, scores the upstream generator as the primary Root Cause (suspicion score > 0.8), and marks the leaf as the Panic Site.

#### Scenario: Correlating with Git recency and branch churn
- **WHEN** `cartograph diagnose --git-blame --branch origin/main` is executed
- **THEN** cartograph weights lines modified in the current branch or recent commits higher in suspicion ranking.

### Requirement: Semantic Fault Classification
The system SHALL classify the defect into standardized archetypes: `NullDeref`, `OutOfBounds`, `TypeDrift`, `MissingGuard`, `StateInvariantViolation`, `ResourceLeak`, and `ConcurrencyPanic`.

### Requirement: Automated Regression Test & Patch Generation
The system SHALL synthesize runnable framework-native regression tests reproducing the defect and generate executable unified git diff patches.

#### Scenario: Scaffolding a regression test reproducer
- **WHEN** `cartograph diagnose "<trace>" --scaffold-test` is executed
- **THEN** cartograph outputs runnable test code matching the repository's test framework.

#### Scenario: Exporting and applying an executable git patch
- **WHEN** `cartograph diagnose "<trace>" --export-patch fix.patch` is executed
- **THEN** cartograph generates a valid git patch and can apply it cleanly when `--apply-patch` is supplied.

### Requirement: Universal Output Layer & Interactive TUI
The system SHALL format diagnosis results in ANSI terminal (with visual heatbars), Markdown (with Mermaid sequence diagrams), OASIS SARIF 2.1.0, JSON, AI agent context XML, and provide an interactive split-pane TUI (`-i`).
