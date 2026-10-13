# `cartograph test-selector`

Select and execute predictive tests impacted by code modifications (<15ms)

## Synopsis

```bash
cartograph test-selector [OPTIONS]
```

## Options & Flags

| Flag | Name | Value | Default | Description |
| :--- | :--- | :--- | :--- | :--- |
| `-p` | `--path` | `PATH` | `.` | Root directory of repository |
| `-d` | `--db` | `DB` | `default` | Database profile or SQLite file path |
| - | `--branch` | `BRANCH` | - | Analyze git branch diff range (e.g. main...HEAD or origin/main...HEAD) |
| - | `--staged` | `STAGED` | - | Analyze uncommitted git staged changes |
| `-g` | `--granularity` | `GRANULARITY` | `auto` | Test granularity: auto, case, suite, file, package (default: auto) |
| - | `--max-tests` | `MAX_TESTS` | - | Maximum number of tests to select (fail-fast truncation) |
| - | `--min-confidence` | `MIN_CONFIDENCE` | `0.5` | Minimum edge confidence threshold (0.0 to 1.0) |
| `-D` | `--depth` | `DEPTH` | `5` | Maximum graph traversal depth (default: 5) |
| - | `--ignore-docstrings` | `IGNORE_DOCSTRINGS` | `true` | Bypass test execution when changes are only docstrings/comments/markdown (default: true) |
| - | `--fail-on-untested` | `FAIL_ON_UNTESTED` | - | CI/CD Gate: fail with exit code 1 if any modified symbol has zero test coverage |
| - | `--min-coverage` | `MIN_COVERAGE` | - | CI/CD Gate: fail with exit code 1 if test coverage ratio is below percentage (e.g. 80.0) |
| `-r` | `--run` | `RUN` | - | Execute the selected test runner commands |
| - | `--bail` | `BAIL` | - | Stop execution immediately on first test failure when using --run |
| - | `--dry-run` | `DRY_RUN` | - | Print execution commands without running |
| - | `--command-only` | `COMMAND_ONLY` | - | Output only executable runner commands, newline separated (ideal for piping to bash) |
| - | `--files-only` | `FILES_ONLY` | - | Output only unique test file paths, newline separated |
| `-f` | `--format` | `FORMAT` | - | Presentation output format: terminal, markdown, json, matrix, script |
| - | `--json` | `JSON` | - | Output results formatted as JSON (shorthand for --format json) |

## Examples

::::{tab-set}
:::{tab-item} Basic Command
```bash
cartograph test-selector
```
:::

:::{tab-item} JSON Output
```bash
cartograph test-selector --json
```
:::
::::
