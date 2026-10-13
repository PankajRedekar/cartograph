# 🗺️ Cartograph Documentation Portal

Welcome to the comprehensive developer documentation portal and living knowledge architecture for **Cartograph**, the ultra-fast native AST code graph intelligence engine for human developers, data scientists, and autonomous AI coding agents.

::::{grid} 1 2 2 3
:gutter: 3

:::{grid-item-card} 🚀 Fast Quickstart
:link: getting-started/index
:link-type: doc

Index a 50,000-line polyglot codebase in <250ms and run your first AST graph search.
:::

:::{grid-item-card} 🛠️ CLI Cookbooks
:link: cookbooks/index
:link-type: doc

14 exhaustive cookbooks for locate, impact, test selection, routes, linting, and more.
:::

:::{grid-item-card} 🤖 AI Agent Skills
:link: agent-integration/index
:link-type: doc

Zero-token lazy-loaded skills for Claude Code, Cursor, Windsurf, Antigravity, and Aider.
:::

:::{grid-item-card} 🐍 Python Data Science
:link: python-sdk/index
:link-type: doc

Zero-copy columnar export to Polars & Pandas via Arrow, plus NetworkX graph analytics.
:::

:::{grid-item-card} 🚢 DevOps & CI/CD
:link: devops-ci-cd/index
:link-type: doc

Pre-commit gates, automated PR reviews, breaking change detection, and SARIF export.
:::

:::{grid-item-card} 🌐 Polyglot Support
:link: polyglot-reference/index
:link-type: doc

AST grammar nuances across Rust, TypeScript, JavaScript, Python, Ruby, Go, SQL, and Markdown.
:::
::::

---

## ⚡ Synchronized Multi-Language Experience

Every concept and operation in Cartograph is accessible via CLI, Python SDK, or AI Agent Skill:

::::{tab-set}
:::{tab-item} CLI Command
:sync: cli

```bash
# Calculate blast-radius impact of modifying the authentication symbol
cartograph impact --symbol "authenticate" --depth 3 --fail-on-breaking
```
```

:::{tab-item} Python SDK
:sync: python

```python
import cartograph

with cartograph.open() as cg:
    impact = cg.impact(symbol="authenticate", max_depth=3)
    print(f"Direct dependents: {len(impact.direct_dependents)}")
    if impact.has_breaking_changes:
        print(f"Risk Score: {impact.risk_score}")
```
```

:::{tab-item} AI Agent Skill
:sync: agent

```markdown
<!-- Agent Directive: Invoke prior to modifying core authentication contracts -->
Invoke: `cartograph impact --symbol "authenticate" --format xml`
Action: Inspect <blast_radius_analysis> to locate all callers before editing.
```
```

:::{tab-item} Structured XML Context
:sync: xml

```xml
<blast_radius_analysis root="auth::authenticate" risk_score="0.82" breaking="true">
  <affected_dependents count="14">
    <node file="src/routes/login.rs" line="45" symbol="login_handler" impact="direct"/>
  </affected_dependents>
</blast_radius_analysis>
```
```
````

---

```{toctree}
:maxdepth: 2
:caption: 🚀 Getting Started
:hidden:

getting-started/index
```

```{toctree}
:maxdepth: 2
:caption: 🧠 Core Concepts
:hidden:

core-concepts/index
```

```{toctree}
:maxdepth: 2
:caption: 🛠️ CLI Cookbooks
:hidden:

cookbooks/index
```

```{toctree}
:maxdepth: 2
:caption: 🤖 AI Agent Integration
:hidden:

agent-integration/index
```

```{toctree}
:maxdepth: 2
:caption: 🐍 Python SDK & Data Science
:hidden:

python-sdk/index
```

```{toctree}
:maxdepth: 2
:caption: 🚢 CI/CD & DevOps
:hidden:

devops-ci-cd/index
```

```{toctree}
:maxdepth: 2
:caption: 🌐 Polyglot Reference
:hidden:

polyglot-reference/index
```

```{toctree}
:maxdepth: 2
:caption: 🎛️ Interactive Tools
:hidden:

tools/config-generator
tools/agent-prompt-wizard
tools/cheatsheet
```

```{toctree}
:maxdepth: 2
:caption: 📖 Reference Manuals
:hidden:

reference/cli/index
reference/configuration
reference/database-schema
reference/python/index
```

```{toctree}
:maxdepth: 2
:caption: 📜 Living OpenSpec
:hidden:

openspec/index
```
