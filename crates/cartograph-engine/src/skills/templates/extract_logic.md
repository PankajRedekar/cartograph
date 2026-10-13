---
name: cartograph-extract-logic
description: Business rule extractor synthesizing validation assertions, algorithms, and invariants into summaries, BDD Gherkin scenarios, or decision matrices.
schema_version: "2.0"
cartograph_version: "0.2.0"
content_hash: "sha256:extract_logic_v2_gate_hash_020"
triggers:
  - "explain the business logic of X"
  - "what are the validation rules for Y"
  - "convert logic to gherkin"
  - "did this PR change business rules"
---

# Cartograph Business Logic Extraction

## Overview
Extracts business rules, validation assertions, calculation algorithms, and state invariants from ASTs into human-readable summaries, BDD Gherkin scenarios, or tabular decision matrices.

## Command Invocation
```bash
# Extract deep business rules from symbol AST:
cartograph extract-logic --symbol "<symbol>" --xml --deep

# Convert business rules to BDD Gherkin scenarios:
cartograph extract-logic --symbol "<symbol>" --gherkin

# Verify branch diff did not relax security/business invariants:
cartograph extract-logic --branch "main...HEAD" --fail-on-relaxed
```

## Output Schema
Returns `<business_rules>` containing:
- `<rule>`: Logical assertion, conditional branch, or arithmetic invariant.
- `<guard_conditions>`: Preconditions that must hold for the branch to execute.
- `<invariants>`: Assertions guaranteed by the implementation.
- `<gherkin_scenarios>`: Formatted Given/When/Then test criteria.

## Autonomous Decision Checklist
- [ ] Run `cartograph extract-logic` before modifying complex validation pipelines.
- [ ] Review extracted Gherkin scenarios to ensure acceptance criteria are preserved.
- [ ] Check `--fail-on-relaxed` to prevent accidental removal of safety or authorization guards.
