---
name: cartograph-verify
description: Closed-loop pre-commit verification gate combining syntax, architectural boundaries, SemVer contract impact, and predictive tests into a sub-200ms check.
schema_version: "2.0"
cartograph_version: "0.2.0"
content_hash: "sha256:verify_v2_gate_hash_020"
triggers:
  - "verify my changes"
  - "check if ready to commit"
  - "run pre-commit checks"
  - "did my edit break anything"
  - "run verification gate"
---

# Cartograph Pre-Commit Verification Gate

## Overview
The primary quality gate for autonomous AI coding agents. In a single atomic command (<200ms), verifies AST syntax, enforces architectural boundaries and circular dependency rules (`lint-arch`), checks transitive contract compatibility (`impact --verify-semver`), and predictively runs only affected tests (`test-selector`). Provides agents with an instant, closed-loop self-healing mechanism before marking any task as complete.

## Command Invocation
```bash
# Verify all staged changes (<200ms):
cartograph verify --staged --format xml

# Verify changes against base branch:
cartograph verify --branch "main...HEAD" --fail-fast
```

## Output Schema
Returns `<verification_gate status="PASSED|FAILED">` containing:
- `<check name="architecture_boundaries" status="PASSED|FAILED">`: Circular dependencies and layer violations.
- `<check name="contract_compatibility" status="PASSED|FAILED">`: Breaking changes to public signatures or return types.
- `<check name="predictive_tests" status="PASSED|FAILED|SKIPPED">`: Targeted test execution results for changed symbols.
- `<remediation_advice>`: Concrete instructions on what to fix if verification failed.

## Autonomous Decision Checklist
- [ ] Step 1: Run `cartograph verify --staged --format xml` before committing or finalizing code.
- [ ] Step 2: If status is `PASSED`, proceed to commit cleanly.
- [ ] Step 3: If status is `FAILED`, inspect `<issue>` items and follow `<remediation_advice>`.
- [ ] Step 4: Correct the contract regression or architecture breach, and re-run verification.
