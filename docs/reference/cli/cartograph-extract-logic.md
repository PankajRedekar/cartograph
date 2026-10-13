# `cartograph extract-logic`

Extract domain business logic, validation rules, and invariants with variable provenance (<25ms)

## Synopsis

```bash
cartograph extract-logic [OPTIONS] [<query>]
```

## Arguments

| Argument | Description | Required |
| :--- | :--- | :--- |
| `<query>` | Target symbol query, concept keyword, or file path (optional) | No |

## Options & Flags

| Flag | Name | Value | Default | Description |
| :--- | :--- | :--- | :--- | :--- |
| `-s` | `--symbol` | `SYMBOL` | - | Target symbol name to extract rules for |
| `-c` | `--concept` | `CONCEPT` | - | Concept taxonomy search keyword (e.g. discount, pricing, auth, kyc) |
| `-f` | `--file` | `FILE` | - | Filter analysis to a specific file |
| `-p` | `--path` | `PATH` | `.` | Root directory of repository |
| `-d` | `--db` | `DB` | `default` | Database profile or SQLite file path |
| - | `--deep` | `DEEP` | - | Perform inter-procedural call-tree BFS traversal |
| `-D` | `--depth` | `DEPTH` | `3` | Maximum inter-procedural call traversal depth (default: 3) |
| - | `--branch` | `BRANCH` | - | Git branch or revision for PR rule diff comparison (e.g. main or origin/main...HEAD) |
| - | `--fail-on-relaxed` | `FAIL_ON_RELAXED` | - | CI/CD Gate: fail with exit code 1 if any business rule was relaxed/loosened |
| - | `--fail-on-removed` | `FAIL_ON_REMOVED` | - | CI/CD Gate: fail with exit code 1 if any existing business rule was deleted/removed |
| - | `--format` | `FORMAT` | - | Presentation output format: terminal, gherkin, table, xml, json (auto-detected if omitted) |
| - | `--json` | `JSON` | - | Output results formatted as JSON (shorthand for --format json) |
| - | `--gherkin` | `GHERKIN` | - | Output results formatted as Gherkin BDD scenarios (shorthand for --format gherkin) |
| - | `--table` | `TABLE` | - | Output results formatted as Markdown Decision Table (shorthand for --format table) |
| - | `--xml` | `XML` | - | Output results formatted as AI prompt XML (shorthand for --format xml) |

## Examples

::::{tab-set}
:::{tab-item} Basic Command
```bash
cartograph extract-logic
```
:::

:::{tab-item} JSON Output
```bash
cartograph extract-logic --json
```
:::
::::
