# 🤖 `cartograph skills` Cookbook

The `cartograph skills` suite manages **Skills-First AI agent configurations**, enabling 1-click scaffolding, drift detection, environment health doctoring, and productivity ROI analytics.

## Common Workflows

### 1. 1-Click Skill Installation for All Detected IDEs

Auto-detect IDEs (Claude Code, Cursor, Windsurf, Antigravity, Aider) and scaffold `.agent/skills/` in `<15ms`:

::::{tab-set}
:::{tab-item} CLI Command
:sync: cli

```bash
cartograph skills install --yes
```
```

:::{tab-item} Python SDK
:sync: python

```python
import cartograph

# Skills are managed natively through the CLI binary
# Invoke via subprocess or CLI in turn-1 agent setup:
# cartograph skills install --target claude
```
```

:::{tab-item} AI Agent Skill
:sync: agent

```markdown
<!-- Agent Turn-1 Self-Installation -->
Invoke: `cartograph skills install --yes`
Outcome: Scaffolds all 10 canonical SKILL.md bundles into .agent/skills/
```
```

:::{tab-item} Structured XML Context
:sync: xml

```xml
<skills_installation status="SUCCESS" installed="10" duration_ms="12">
  <ide target="claude" file="CLAUDE.md" updated="true"/>
  <ide target="cursor" file=".cursorrules" updated="true"/>
</skills_installation>
```
```
````

### 2. Upgrading Installed Skills via 3-Way Merge

Upgrade installed templates to match the running binary without overwriting user rules:

```bash
cartograph skills update
```

### 3. Environment Doctor & Auto-Remediation

Diagnose missing skills, stale SQLite database caches, and auto-repair:

```bash
cartograph skills check --fix
```

### 4. Telemetry ROI & Productivity Stats

Display tokens saved, full-file reads avoided, and test minutes spared:

```bash
cartograph skills stats
```
