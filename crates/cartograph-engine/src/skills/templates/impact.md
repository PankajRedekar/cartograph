---
name: cartograph-impact
description: Pre-edit blast radius & breaking change detector calculating downstream risk score, contract breaks, and missing test coverage.
schema_version: "2.0"
cartograph_version: "0.2.0"
content_hash: "sha256:impact_v2_gate_hash_020"
triggers:
  - "refactor function X"
  - "rename parameter in Y"
  - "delete method Z"
  - "what breaks if I change this"
  - "check blast radius of my changes"
---

# Cartograph Impact Analysis

## Overview
Computes the transitive blast-radius and downstream risk score before an agent edits or renames a symbol or commits a PR. Catches breaking contract changes and missing downstream test coverage before writing code.

## Command Invocation
```bash
# Query transitive impact of changing a symbol:
cartograph impact "<symbol>" --format xml --direction downstream --depth 3

# Verify staged changes for breaking SemVer contracts:
cartograph impact --staged --format xml --verify-semver major

# Analyze impact of branch changes against main:
cartograph impact --branch "main...HEAD" --format xml
```

## Output Schema
Returns `<blast_radius_analysis>` containing:
- `<root_target>`: Target name, file, line, and kind.
- `<impact_summary>`: Total impacted symbols, impacted files, downstream test count, and risk score.
- `<breaking_changes>`: Identified contract breaks, return type mismatches, or signature alterations.
- `<downstream_dependents>`: Transitive caller tree with call site snippets and test coverage indicators.

## Autonomous Decision Checklist
- [ ] Run `cartograph impact "<symbol>"` before modifying any public or shared symbol.
- [ ] If `risk_score` is HIGH or CRITICAL, plan updates for all downstream callers.
- [ ] Check `<tests_covering>` to verify which callers have test coverage.
- [ ] If breaking changes are required, verify SemVer impact with `--verify-semver`.
