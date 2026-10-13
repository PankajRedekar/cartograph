---
name: cartograph-diagnose
description: Crash log and stack trace AST backward slicer that anchors frames to code nodes, isolates root causes, and scaffolds test reproducers.
schema_version: "2.0"
cartograph_version: "0.2.0"
content_hash: "sha256:diagnose_v2_gate_hash_020"
triggers:
  - "fix this panic"
  - "diagnose test failure"
  - "here is a stack trace"
  - "why did this error happen"
  - "reproduce this bug"
---

# Cartograph Bug Diagnosis & Backward Slicing

## Overview
Parses raw stack traces, test failure dumps, and Sentry crash logs. Anchors stack frames directly to AST graph nodes, performs def-use backward slicing to locate root-cause variable mutations, and scaffolds framework-native unit test reproducers.

## Command Invocation
```bash
# Diagnose a stack trace and scaffold reproducer test:
cartograph diagnose "<stack_trace_string>" --xml --scaffold-test

# Diagnose from a crash log file with git blame context:
cartograph diagnose --file crash.log --xml --git-blame
```

## Output Schema
Returns `<diagnostic_context>` containing:
- `<fault_frame>`: Exact file, line, and function where the panic or exception occurred.
- `<ast_root_cause>`: Symbol and statement that introduced the invalid state or null reference.
- `<backward_slice>`: Chain of variable assignments leading to the fault.
- `<reproducer_scaffold>`: Executable unit test replicating the crash scenario.

## Autonomous Decision Checklist
- [ ] Run `cartograph diagnose "<stack_trace>"` when encountering an exception or panic.
- [ ] Save the `<reproducer_scaffold>` to a new test file to confirm reproduction.
- [ ] Implement the fix at the `<ast_root_cause>` location.
- [ ] Re-run the reproducer test to verify that the bug is resolved.
