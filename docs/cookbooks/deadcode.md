# 💀 `cartograph deadcode` Cookbook

The `cartograph deadcode` command executes **mark-and-sweep reachability analysis**, uncovering unused functions, unread struct fields, orphan UI components, and ghost dependencies.

## Common Workflows

### 1. Repository-Wide Dead Code Sweep

Scan for dead symbols and orphaned modules:

::::{tab-set}
:::{tab-item} CLI Command
:sync: cli

```bash
cartograph deadcode --min-lines 15
```
```

:::{tab-item} Python SDK
:sync: python

```python
import cartograph

with cartograph.open() as cg:
    deadcode = cg.deadcode(min_lines=15)
    print(f"Discovered {len(deadcode.symbols)} dead functions/types")
    print(f"Total dead lines: {deadcode.total_dead_lines}")
```
```

:::{tab-item} AI Agent Skill
:sync: agent

```markdown
<!-- Agent Directive: Scan for dead code islands before refactoring -->
Invoke: `cartograph deadcode --json`
Action: Prune unreachable code verified safe by reachability analysis.
```
```

:::{tab-item} Structured XML Context
:sync: xml

```xml
<deadcode_report total_dead_symbols="2" total_dead_lines="48">
  <dead_symbol name="legacy_oauth_callback" file="src/auth.rs" line="230" lines="28"/>
</deadcode_report>
```
```
````

### 2. PR Diff Gate: Accidental Orphan Prevention

Fail CI if a pull request introduces new dead code or leaves orphaned functions:

```bash
cartograph deadcode --branch "origin/main...HEAD" --fail-on-new-deadcode
```

### 3. Generate `.contextignore` to Shield AI Coding Agents

Shield AI agents from ingesting dead code in context loops:

```bash
cartograph deadcode --export-ai-ignore .contextignore
```
