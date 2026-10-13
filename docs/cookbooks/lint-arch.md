# 📐 `cartograph lint-arch` Cookbook

The `cartograph lint-arch` command checks **architectural layer boundaries, forbidden dependencies, and circular dependency cycles** in `<10ms`, providing actionable Minimum Feedback Arc Set (MFAS) cut-edge advice.

## Common Workflows

### 1. Strict Boundary & Cycle Enforcement

Run strict architectural validation on the codebase:

::::{tab-set}
:::{tab-item} CLI Command
:sync: cli

```bash
cartograph lint-arch --strict --fail-on error
```
```

:::{tab-item} Python SDK
:sync: python

```python
import cartograph

with cartograph.open() as cg:
    report = cg.lint_arch(strict=True)
    if report.has_violations:
        print(f"Found {len(report.violations)} architectural boundary violations!")
        for cycle in report.cycles:
            print(f"Cycle detected: {' -> '.join(cycle.path)}")
```
```

:::{tab-item} AI Agent Skill
:sync: agent

```markdown
<!-- Agent Directive: Validate architectural boundaries -->
Invoke: `cartograph lint-arch --strict --format xml`
Action: If circular dependencies are reported, invert the suggested MFAS cut-edge.
```
```

:::{tab-item} Structured XML Context
:sync: xml

```xml
<lint_arch_report status="PASSED" total_cycles="0" boundary_violations="0">
  <summary layers_enforced="4" audited_modules="38"/>
</lint_arch_report>
```
```
````

### 2. Exporting to SARIF for GitHub Code Scanning

Export violations formatted as SARIF 2.1.0 for PR annotations:

```bash
cartograph lint-arch --sarif > results.sarif
```

### 3. Architecture Preset Templates

Enforce Clean Architecture or Hexagonal Architecture presets:

```bash
cartograph lint-arch --preset clean-architecture
```
