# `cartograph diagram`

Generate visual code diagrams (architecture, call graph, ER schema, sequence, logic, diff, concept)

## Synopsis

```bash
cartograph diagram [OPTIONS] [<query>]
```

## Arguments

| Argument | Description | Required |
| :--- | :--- | :--- |
| `<query>` | Target symbol query, route pattern, or keyword | No |

## Options & Flags

| Flag | Name | Value | Default | Description |
| :--- | :--- | :--- | :--- | :--- |
| `-p` | `--path` | `PATH` | `.` | Root directory of repository |
| `-d` | `--db` | `DB` | `default` | Database profile or SQLite file path |
| `-t` | `--type` | `DIAGRAM_TYPE` | - | Diagram archetype: architecture, callgraph, er, sequence, logic, diff, concept (auto-detected if omitted) |
| `-f` | `--format` | `FORMAT` | - | Output format: mermaid, svg, html, text, d2, dot, plantuml, json, xml (default: mermaid) |
| - | `--theme` | `THEME` | - | Visual color theme: dark, light, high_contrast (default: dark) |
| `-D` | `--depth` | `DEPTH` | - | Traversal depth (default: 2) |
| - | `--direction` | `DIRECTION` | - | Subgraph traversal direction: upstream (callers), downstream (callees), both (default: both) |
| `-n` | `--max-nodes` | `MAX_NODES` | - | Maximum number of nodes before pruning by degree centrality (default: 35) |
| - | `--collapse-modules` | `COLLAPSE_MODULES` | `true` | Collapse sibling files into package/module clusters (default: true) |
| - | `--include-tests` | `INCLUDE_TESTS` | - | Include test symbols and test files in diagram |
| - | `--include-stdlib` | `INCLUDE_STDLIB` | - | Include standard library symbols in diagram |
| `-c` | `--concept` | `CONCEPT` | - | Concept taxonomy keyword for concept cluster diagram |
| - | `--branch` | `BRANCH` | - | Git branch or revision for PR blast-radius diff overlay (e.g. main...HEAD) |
| - | `--staged` | `STAGED` | - | Overlay git staged changes onto diagram |
| `-o` | `--out` | `FILE` | - | Save diagram output directly to a file |
| - | `--open` | `OPEN` | - | Open generated diagram in the default system browser |
| `-i` | `--interactive` | `INTERACTIVE` | - | Launch interactive terminal split-pane diagram explorer |

## Examples

::::{tab-set}
:::{tab-item} Basic Command
```bash
cartograph diagram
```
:::

:::{tab-item} JSON Output
```bash
cartograph diagram --json
```
:::
::::
