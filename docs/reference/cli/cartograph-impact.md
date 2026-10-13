# `cartograph impact`

Analyze blast radius, breaking changes, and risk of code modifications (<25ms)

## Synopsis

```bash
cartograph impact [OPTIONS] [<target>]
```

## Arguments

| Argument | Description | Required |
| :--- | :--- | :--- |
| `<target>` | Target symbol query or filepath (optional if using --branch or --staged) | No |

## Options & Flags

| Flag | Name | Value | Default | Description |
| :--- | :--- | :--- | :--- | :--- |
| `-t` | `--target` | `TARGET_FLAG` | - | Target symbol query or filepath (flag form) |
| `-p` | `--path` | `PATH` | `.` | Root directory of repository |
| `-d` | `--db` | `DB` | `default` | Database profile or SQLite file path |
| - | `--branch` | `BRANCH` | - | Analyze git branch diff range (e.g. main...HEAD or origin/master...HEAD) |
| - | `--staged` | `STAGED` | - | Analyze uncommitted git staged changes |
| - | `--simulate-delete` | `SIMULATE_DELETE` | - | Speculatively simulate deleting a symbol |
| - | `--simulate-rename` | `SIMULATE_RENAME` | - | Speculatively simulate renaming a symbol (format: old_name:new_name) |
| - | `--verify-semver` | `VERIFY_SEMVER` | - | Enforce Semantic Versioning policy (patch, minor, major) |
| - | `--direction` | `DIRECTION` | `dependents` | Graph traversal direction: dependents (callers), dependencies (callees), both (default: dependents) |
| `-D` | `--depth` | `DEPTH` | `3` | Maximum traversal depth (default: 3) |
| - | `--min-confidence` | `MIN_CONFIDENCE` | - | Minimum edge confidence threshold (0.0 to 1.0) |
| - | `--no-damping` | `NO_DAMPING` | - | Disable high-degree utility damping |
| - | `--fail-on` | `FAIL_ON` | - | CI/CD Gate: fail with exit code 1 if risk meets or exceeds threshold (low, medium, high, critical) |
| - | `--min-test-coverage` | `MIN_TEST_COVERAGE` | - | CI/CD Gate: fail with exit code 1 if test coverage ratio is below threshold (0.0 to 1.0) |
| `-f` | `--format` | `FORMAT` | - | Presentation output format: terminal, markdown, json, xml (auto-detected if omitted) |
| - | `--json` | `JSON` | - | Output results formatted as JSON (shorthand for --format json) |
| `-i` | `--interactive` | `INTERACTIVE` | - | Interactive terminal TUI mode |

## Examples

::::{tab-set}
:::{tab-item} Basic Command
```bash
cartograph impact
```
:::

:::{tab-item} JSON Output
```bash
cartograph impact --json
```
:::
::::
