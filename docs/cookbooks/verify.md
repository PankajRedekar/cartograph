# ✅ `cartograph verify` Cookbook

The `cartograph verify` command is the **atomic pre-commit closed-loop verification gate**, validating AST syntax, architectural boundaries, SemVer breaking changes, and running predictive tests in `<200ms`.

## Common Workflows

### 1. Pre-Commit Verification Gate

Run atomic checks on uncommitted Git staged modifications:

::::{tab-set}
:::{tab-item} CLI Command
:sync: cli

```bash
cartograph verify --staged --fail-fast
```
```

:::{tab-item} Python SDK
:sync: python

```python
import cartograph

# cartograph verify is designed for high-speed pre-commit git hooks
# Run in bash: cartograph verify --staged
```
```

:::{tab-item} AI Agent Skill
:sync: agent

```markdown
<!-- Agent Directive: Mandatory pre-commit verification -->
Invoke: `cartograph verify --staged`
Gate: If status="FAILED", fix reported issues before creating git commit.
```
```

:::{tab-item} Structured XML Context
:sync: xml

```xml
<verification_gate status="PASSED" duration_ms="184">
  <check name="ast_syntax" status="OK"/>
  <check name="architectural_boundaries" status="OK"/>
  <check name="breaking_changes" status="OK"/>
  <check name="predictive_tests" passed="4" failed="0"/>
</verification_gate>
```
```
````

### 2. Base Branch Range Verification

Verify an entire feature branch against `main`:

```bash
cartograph verify --branch "origin/main...HEAD" --fail-fast
```
