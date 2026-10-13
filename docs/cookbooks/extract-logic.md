# 📋 `cartograph extract-logic` Cookbook

The `cartograph extract-logic` command parses AST expressions to **extract domain validation rules, pricing policies, and business constraints**, exporting them as Gherkin BDD scenarios or Decision Tables.

## Common Workflows

### 1. Extract Validation Rules as Gherkin Scenarios

Generate human-readable BDD specifications from validation logic:

::::{tab-set}
:::{tab-item} CLI Command
:sync: cli

```bash
cartograph extract-logic --symbol "validate_order" --gherkin
```
```

:::{tab-item} Python SDK
:sync: python

```python
import cartograph

with cartograph.open() as cg:
    logic = cg.extract_logic(symbol="validate_order", format="gherkin")
    print(logic.content)
```
```

:::{tab-item} AI Agent Skill
:sync: agent

```markdown
<!-- Agent Directive: Extract validation rules as Gherkin scenarios -->
Invoke: `cartograph extract-logic --symbol "validate_order" --gherkin`
Action: Generate unit tests covering all extracted BDD edge cases.
```
```

:::{tab-item} Structured XML Context
:sync: xml

```xml
<business_rules symbol="validate_order" file="src/order.rs">
  <rule id="R1" condition="order.total &gt; 1000.0" outcome="require_manager_approval"/>
  <rule id="R2" condition="user.is_suspended" outcome="reject_transaction"/>
</business_rules>
```
```
````

### 2. Exporting Markdown Decision Tables

Synthesize tabular business rule logic:

```bash
cartograph extract-logic --concept "discount" --table
```

### 3. Business Rule Regression Gate

Detect if a PR accidentally loosens or deletes critical business rules:

```bash
cartograph extract-logic --branch "origin/main...HEAD" --fail-on-relaxed --fail-on-removed
```
