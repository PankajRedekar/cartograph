# 💥 `cartograph impact` Cookbook

The `cartograph impact` command calculates the **transitive blast radius, affected dependents, and SemVer breaking change risks** of code changes in `<25ms`.

## Common Workflows

### 1. Pre-Edit Blast Radius Calculation

Inspect all upstream callers that depend on a function before modifying its signature:

::::{tab-set}
:::{tab-item} CLI Command
:sync: cli

```bash
cartograph impact --symbol "authenticate" --depth 3 --fail-on-breaking
```
```

:::{tab-item} Python SDK
:sync: python

```python
import cartograph

with cartograph.open() as cg:
    impact = cg.impact("authenticate", max_depth=3)
    print(f"Risk Score: {impact.risk_score}")
    print(f"Direct Dependents: {len(impact.direct_dependents)}")
    if impact.has_breaking_changes:
        raise ValueError("Breaking change detected in core authentication contract!")
```
```

:::{tab-item} AI Agent Skill
:sync: agent

```markdown
<!-- Agent Directive: Pre-edit blast radius calculation -->
Invoke: `cartograph impact --symbol "authenticate" --format xml`
Verify: Review <blast_radius_analysis> risk_score and breaking attribute before proceeding.
```
```

:::{tab-item} Structured XML Context
:sync: xml

```xml
<blast_radius_analysis root="auth::authenticate" risk_score="0.75" breaking="false">
  <direct_dependents count="4">
    <node file="src/routes/login.rs" line="34" symbol="login_handler" depth="1"/>
    <node file="src/routes/api.rs" line="92" symbol="verify_jwt" depth="1"/>
  </direct_dependents>
</blast_radius_analysis>
```
```
````

### 2. CI/CD Gate on Pull Request Diff Range

Ensure no breaking changes are merged into the main branch:

```bash
cartograph impact --branch "origin/main...HEAD" --fail-on critical
```

### 3. Speculative Rename / Delete Simulation

Test the architectural consequence of deleting or renaming a symbol without modifying code:

```bash
cartograph impact --simulate-delete "LegacySessionManager"
```
