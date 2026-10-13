# 📈 `cartograph complexity` Cookbook

The `cartograph complexity` command performs **4-quadrant hotspot analysis**, calculating cognitive complexity, cyclomatic complexity, God-class anti-patterns, and **Danger Zone** items (high complexity with zero test coverage).

## Common Workflows

### 1. Top Hotspots & Danger Zone Analysis

Inspect top technical debt hotspots in the repository:

::::{tab-set}
:::{tab-item} CLI Command
:sync: cli

```bash
cartograph complexity --top 10 --danger-zone
```
```

:::{tab-item} Python SDK
:sync: python

```python
import cartograph

with cartograph.open() as cg:
    report = cg.complexity(top=10)
    for h in report.hotspots:
        print(f"Quadrant: {h.quadrant} | {h.symbol} (Score: {h.score}, Test Coverage: {h.test_coverage})")
```
```

:::{tab-item} AI Agent Skill
:sync: agent

```markdown
<!-- Agent Directive: Identify God-classes and refactoring targets -->
Invoke: `cartograph complexity --top 5 --god-class --format xml`
Action: Refactor flagged methods into cohesive sub-modules.
```
```

:::{tab-item} Structured XML Context
:sync: xml

```xml
<complexity_heatmap top="10" total_hotspots="3">
  <hotspot quadrant="Q1_DANGER_ZONE" symbol="SessionManager::handle_request" score="84" coverage="0.0">
    <coordinates file="src/session.rs" line="145"/>
    <recommendation>Extract session lifecycle logic into separate trait implementation.</recommendation>
  </hotspot>
</complexity_heatmap>
```
```
````

### 2. Standalone Interactive HTML Dashboard

Generate a shareable, visual complexity heatmap dashboard:

```bash
cartograph complexity --html --out heatmap.html
```

### 3. CI Gate on Cognitive Complexity Delta

Prevent PRs from introducing severe complexity spikes:

```bash
cartograph complexity --branch "origin/main...HEAD" --fail-on-hotspots
```
