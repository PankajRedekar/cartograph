# 📊 `cartograph diagram` Cookbook

The `cartograph diagram` command exports **visual architecture diagrams, call graphs, sequence diagrams, and ER schemas** via the pure-Rust Sugiyama layout engine.

## Common Workflows

### 1. Generating Mermaid.js Architecture Diagrams

Generate an architectural overview diagram for documentation:

::::{tab-set}
:::{tab-item} CLI Command
:sync: cli

```bash
cartograph diagram --type architecture --format mermaid
```
```

:::{tab-item} Python SDK
:sync: python

```python
import cartograph

with cartograph.open() as cg:
    diagram = cg.diagram(diagram_type="architecture", format="mermaid")
    print(diagram.content)
```
```

:::{tab-item} AI Agent Skill
:sync: agent

```markdown
<!-- Agent Directive: Export architectural call graph -->
Invoke: `cartograph diagram --symbol "<NAME>" --format mermaid`
Verify: Review upstream callers and downstream callees visually.
```
```

:::{tab-item} Structured XML Context
:sync: xml

```xml
<architecture_diagram type="callgraph" target="authenticate">
  <node id="auth" label="authenticate" file="src/auth.rs"/>
  <node id="login" label="login_handler" file="src/routes/login.rs"/>
  <edge from="login" to="auth" kind="Calls"/>
</architecture_diagram>
```
```
````

### 2. Standalone Scalable Vector Graphics (SVG)

Export a standalone SVG file with embedded dark/light themes and open in browser:

```bash
cartograph diagram --type callgraph --format svg --out callgraph.svg --open
```

### 3. PR Blast Radius Diff Overlay

Overlay changed files onto the architectural graph:

```bash
cartograph diagram --branch "origin/main...HEAD" --format svg --out pr_diff.svg
```
