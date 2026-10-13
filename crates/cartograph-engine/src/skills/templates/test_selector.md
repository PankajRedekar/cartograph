---
name: cartograph-test-selector
description: Predictive test selection mapping changed symbols and files to minimal sufficient test suites, saving up to 98% in test runtime.
schema_version: "2.0"
cartograph_version: "0.2.0"
content_hash: "sha256:test_selector_v2_gate_hash_020"
triggers:
  - "run tests for my changes"
  - "which tests cover this function"
  - "verify my edits"
  - "run affected tests"
---

# Cartograph Predictive Test Selection

## Overview
Maps changed symbols and files to their minimal, sufficient test suite. Prevents running entire 20-minute test suites, generating executable runner commands (`cargo test`, `pytest`, `npm test`, `go test`) targeting only affected test cases.

## Command Invocation
```bash
# Generate shell command targeting only affected tests:
cartograph test-selector --staged --format script

# Run affected tests directly:
cartograph test-selector --staged --run

# Analyze affected tests in XML format:
cartograph test-selector --staged --format xml --fail-on-untested
```

## Output Schema
Returns `<test_selection>` containing:
- `<affected_symbols>`: Changed symbols in staged diff or commit range.
- `<selected_tests>`: Minimal covering test cases with file and line locations.
- `<runner_command>`: Fully formed framework-native shell command ready for execution.

## Autonomous Decision Checklist
- [ ] After modifying code, run `cartograph test-selector --staged --run`.
- [ ] If untested symbols are detected, add new unit tests covering the modified logic.
- [ ] Ensure all selected tests pass before proceeding to pre-commit verification.
