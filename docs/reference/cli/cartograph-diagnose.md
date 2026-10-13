# `cartograph diagnose`

Topological Root-Cause Detective & Crash Log Analyzer (<15ms offline, AST-grounded root-cause pinpointing)

## Synopsis

```bash
cartograph diagnose [OPTIONS] [<input>]
```

## Arguments

| Argument | Description | Required |
| :--- | :--- | :--- |
| `<input>` | Raw traceback string, file path, or "-" for stdin | No |

## Options & Flags

| Flag | Name | Value | Default | Description |
| :--- | :--- | :--- | :--- | :--- |
| `-f` | `--file` | `FILE` | - | Path to crash log file |
| - | `--sentry` | `FILE` | - | Path to Sentry or Datadog APM JSON payload |
| `-p` | `--path` | `PATH` | `.` | Root directory of repository |
| `-d` | `--db` | `DB` | `default` | Database profile or SQLite file path |
| - | `--deep` | `DEEP` | - | Deep mode: traverse callers beyond stack frames into upstream orchestrators |
| - | `--git-blame` | `GIT_BLAME` | `true` | Weight lines by git blame recency (default true) |
| - | `--branch` | `BRANCH` | - | Correlate with active branch/PR diff (e.g. "origin/main") |
| - | `--offline` | `OFFLINE` | - | Run in pure offline deterministic mode (no LLM calls) |
| - | `--provider` | `PROVIDER` | - | LLM provider (anthropic, openai, gemini, ollama, custom) |
| - | `--model` | `MODEL` | - | LLM model name |
| - | `--export-patch` | `EXPORT_PATCH` | - | Export unified remediation patch to file |
| - | `--apply-patch` | `APPLY_PATCH` | - | Apply suggested fixes from patch to codebase |
| - | `--scaffold-test` | `SCAFFOLD_TEST` | - | Synthesize framework-native regression test reproducer |
| - | `--json` | `JSON` | - | Output results formatted as JSON |
| - | `--sarif` | `SARIF` | - | Output results formatted as SARIF 2.1.0 for GitHub Code Scanning |
| - | `--markdown` | `MARKDOWN` | - | Output results formatted as Markdown |
| - | `--xml` | `XML` | - | Output results formatted as AI prompt XML |
| `-i` | `--interactive` | `INTERACTIVE` | - | Interactive terminal TUI mode (split-pane failure explorer) |

## Examples

::::{tab-set}
:::{tab-item} Basic Command
```bash
cartograph diagnose
```
:::

:::{tab-item} JSON Output
```bash
cartograph diagnose --json
```
:::
::::
