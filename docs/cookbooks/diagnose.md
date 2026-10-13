# 🩺 `cartograph diagnose` Cookbook

The `cartograph diagnose` command parses **crash logs, stack traces, and Sentry payloads**, anchoring frames to AST coordinates and performing def-use backward slicing to scaffold reproducing tests.

## Common Workflows

### 1. Stack Trace Parsing & Root-Cause Slicing

Analyze a crash trace directly from stdin or file:

::::{tab-set}
:::{tab-item} CLI Command
:sync: cli

```bash
cartograph diagnose --file crash.log --deep --scaffold-test
```
```

:::{tab-item} Python SDK
:sync: python

```python
import cartograph

with cartograph.open() as cg:
    diag = cg.diagnose(log_file="crash.log", deep=True)
    print(f"Probable Root Cause: {diag.root_cause.symbol} at {diag.root_cause.file}:{diag.root_cause.line}")
```
```

:::{tab-item} AI Agent Skill
:sync: agent

```markdown
<!-- Agent Directive: Diagnose crash trace and scaffold reproducer -->
Invoke: `cartograph diagnose --file crash.log --format xml --scaffold-test`
Action: Implement minimal patch addressing the identified backward def-use slice.
```
```

:::{tab-item} Structured XML Context
:sync: xml

```xml
<diagnostic_report status="RESOLVED" confidence="0.94">
  <root_cause file="src/auth.rs" line="88" symbol="parse_header">
    <slice_provenance variable="auth_header" unhandled_variant="None"/>
  </root_cause>
</diagnostic_report>
```
```
````

### 2. Sentry APM JSON Integration

Ingest live production crash dumps from Sentry:

```bash
cartograph diagnose --sentry sentry_payload.json
```

### 3. Scaffolding Reproducer Unit Tests

Automatically generate a failing test reproducing the crash:

```bash
cartograph diagnose --file traceback.txt --scaffold-test
```
