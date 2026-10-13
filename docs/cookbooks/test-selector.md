# 🧪 `cartograph test-selector` Cookbook

The `cartograph test-selector` command maps modified files and symbols to **predictively impacted tests**, running only affected test suites and cutting CI test execution time by **80% to 95%**.

## Common Workflows

### 1. Predictive Test Execution on Staged Changes

Select and run tests covering uncommitted staged edits:

::::{tab-set}
:::{tab-item} CLI Command
:sync: cli

```bash
cartograph test-selector --staged --run --bail
```
```

:::{tab-item} Python SDK
:sync: python

```python
import cartograph

with cartograph.open() as cg:
    selection = cg.select_tests(staged=True)
    print(f"Selected {len(selection.tests)} test cases across {len(selection.files)} files")
    for cmd in selection.runner_commands:
        print(f"Execute: {cmd}")
```
```

:::{tab-item} AI Agent Skill
:sync: agent

```markdown
<!-- Agent Directive: Run predictive test selection after editing code -->
Invoke: `cartograph test-selector --staged --run --bail`
Action: If tests fail, inspect failure trace before attempting commit.
```
```

:::{tab-item} Structured XML Context
:sync: xml

```xml
<test_selection changed_symbols="2" total_tests_selected="5" execution_time_avoided_pct="91.4">
  <runner command="cargo test --test auth_integration_test"/>
  <affected_test file="tests/auth_test.rs" line="112" symbol="test_token_expiry"/>
</test_selection>
```
```
````

### 2. CI/CD Fail-Fast Gate on PR Branch

Enforce that 100% of modified functions have passing test coverage:

```bash
cartograph test-selector --branch "origin/main...HEAD" --fail-on-untested --run --bail
```

### 3. Piping Commands to Shell

Output only raw test runner commands for native CI execution:

```bash
cartograph test-selector --branch "origin/main...HEAD" --command-only | bash
```
