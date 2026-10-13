# 🤖 `cartograph ai-review` Cookbook

The `cartograph ai-review` command executes **contract-aware code reviews**, detecting breaking changes and downstream side-effects, generating unified executable patches with 1-click application.

## Common Workflows

### 1. Offline Deterministic Code Review (<10ms)

Review staged Git modifications with zero LLM API dependency:

::::{tab-set}
:::{tab-item} CLI Command
:sync: cli

```bash
cartograph ai-review --staged --offline --fast
```
:::

:::{tab-item} Python SDK
:sync: python

```python
import cartograph

with cartograph.open() as cg:
    review = cg.review(staged=True, offline=True)
    for finding in review.findings:
        print(f"[{finding.severity}] {finding.file}:{finding.line} - {finding.message}")
```
:::

:::{tab-item} AI Agent Skill
:sync: agent

```markdown
<!-- Agent Directive: Review staged modifications before commit -->
Invoke: `cartograph ai-review --staged --offline --fast`
Action: If blocker findings exist, apply suggested patch before committing.
```
:::

:::{tab-item} Structured XML Context
:sync: xml

```xml
<ai_review_report status="PASSED" findings_count="0" offline="true">
  <summary contract_checks_passed="18" regression_risk="NONE"/>
</ai_review_report>
```
:::
::::

### 2. Exporting and Applying Unified Remediation Patches

Export suggested fixes to a patch file and apply them atomically:

```bash
# Export patch
cartograph ai-review --branch "origin/main...HEAD" --export-patch fix.patch

# Apply patch
cartograph ai-review --apply-patch fix.patch
```

### 3. CI Gate for PR Blocker Findings

Fail CI if blocker or breaking contract findings are detected:

```bash
cartograph ai-review --branch "origin/main...HEAD" --fail-on-blocker --fail-on-breaking
```
