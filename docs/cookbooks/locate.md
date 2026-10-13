# 🔍 `cartograph locate` Cookbook

The `cartograph locate` command performs **surgical 1-turn AST subgraph retrieval** in `<3ms`, eliminating blind, repetitive `grep` and `find` loops.

## Common Workflows

### 1. Rapid Symbol Search

Retrieve a function definition, inlined parameter types, callers, and callees:

::::{tab-set}
:::{tab-item} CLI Command
:sync: cli

```bash
cartograph locate "authenticate" --depth 2
```
```

:::{tab-item} Python SDK
:sync: python

```python
import cartograph

with cartograph.open() as cg:
    subgraph = cg.locate("authenticate", max_depth=2)
    print(f"Root symbol: {subgraph.root.name} in {subgraph.root.file_path}")
    print(f"Callers: {[c.name for c in subgraph.callers]}")
```
```

:::{tab-item} AI Agent Skill
:sync: agent

```markdown
<!-- Agent Directive: Locate target symbol and inlined contracts in 1 turn -->
Invoke: `cartograph locate "authenticate" --format xml`
Action: Inspect <code_context> for exact line coordinates and caller contracts.
```
```

:::{tab-item} Structured XML Context
:sync: xml

```xml
<cartograph_context query="authenticate" confidence="0.98">
  <target id="auth::authenticate" file="src/auth.rs" line="42" kind="function">
    <signature>pub fn authenticate(user: &amp;str, token: &amp;str) -&gt; Result&lt;Session&gt;</signature>
  </target>
  <callers count="3">
    <node file="src/routes/login.rs" line="18" symbol="login_handler"/>
  </callers>
</cartograph_context>
```
```
````

### 2. Filtering by AST Kind and Language

Limit retrieval to interfaces in TypeScript:

```bash
cartograph locate "UserRepository" --kind interface --lang typescript
```

### 3. Execution Flow Tracing

Automatically follow calls downstream to database queries:

```bash
cartograph locate "checkout" --flow --direction out
```
