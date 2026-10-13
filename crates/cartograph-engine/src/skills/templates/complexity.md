---
name: cartograph-complexity
description: Cognitive complexity & Danger Zone refactoring advisor pinpointing God classes and generating modular decomposition blueprints.
schema_version: "2.0"
cartograph_version: "0.2.0"
content_hash: "sha256:complexity_v2_gate_hash_020"
triggers:
  - "find complex code"
  - "identify refactoring candidates"
  - "show god classes"
  - "find danger zones"
  - "how can I simplify this file"
---

# Cartograph Complexity Analysis

## Overview
Detects cognitive complexity hotspots, God classes, and "Danger Zones" (high complexity coupled with low test coverage). Generates LCOM4 modular decomposition blueprints for refactoring.

## Command Invocation
```bash
# Identify top Danger Zones in the repository:
cartograph complexity --danger-zone --format xml --top 5

# Analyze specific file and generate refactoring blueprint:
cartograph complexity --file "src/service.rs" --prompt-refactor --format xml
```

## Output Schema
Returns `<complexity_hotspots>` containing:
- `<hotspot>`: Symbol name, file, lines, and cyclomatic/cognitive scores.
- `<deficit_tier>`: Criticality rating combining complexity and test deficit.
- `<decomposition_plan>`: Suggested function splits and modular extractions.

## Autonomous Decision Checklist
- [ ] Before modifying legacy modules, check `cartograph complexity --file "<path>"`.
- [ ] Target functions with cognitive complexity > 15 for modular extraction.
- [ ] Follow `<decomposition_plan>` recommendations to isolate pure logic from side effects.
