# `cartograph locate`

Localize and retrieve a 1-turn comprehensive code subgraph context (<3ms)

## Synopsis

```bash
cartograph locate [OPTIONS] <query> [<path>]
```

## Arguments

| Argument | Description | Required |
| :--- | :--- | :--- |
| `<query>` | Target symbol query, path, or natural language intent (supports inline directives e.g. 'kind:function') | Yes |
| `<path>` | Root directory of repository | No |

## Options & Flags

| Flag | Name | Value | Default | Description |
| :--- | :--- | :--- | :--- | :--- |
| `-d` | `--db` | `DB` | `default` | Database profile or SQLite file path |
| `-k` | `--kind` | `KIND` | - | Filter target symbol AST kind (e.g. function, method, class, struct, interface) |
| - | `--file` | `FILE` | - | Path filter (substring or glob) |
| `-l` | `--lang` | `LANG` | - | Language filter (e.g. rust, typescript, python, go) |
| `-D` | `--depth` | `DEPTH` | - | BFS subgraph traversal depth (default: 1) |
| - | `--flow` | `FLOW` | - | Enable end-to-end execution flow tracing (auto depth along Calls/MapsTo) |
| - | `--direction` | `DIRECTION` | - | Subgraph traversal direction: in (callers), out (callees), both (default) |
| `-e` | `--edges` | `EDGES` | - | Comma-separated edge types to follow (e.g. 'Calls,MapsTo,Tests,Imports') |
| - | `--min-confidence` | `MIN_CONFIDENCE` | - | Minimum edge confidence threshold (0.0 to 1.0) |
| - | `--resolve-types` | `RESOLVE_TYPES` | - | Inline parameter and return type signatures |
| - | `--detail` | `DETAIL` | `full` | Context density tier: minimal, signatures, full (default: full) |
| - | `--code` | `CODE` | `true` | Include implementation source code snippet |
| `-C` | `--context` | `CONTEXT` | - | Extra lines of source context before/after target definition |
| `-m` | `--max-tokens` | `MAX_TOKENS` | - | AI token budget ceiling for context pruning |
| `-n` | `--limit` | `LIMIT` | - | Maximum number of candidate clusters to return (default: 5) |
| `-f` | `--format` | `FORMAT` | - | Presentation output format: terminal, markdown, json, xml (auto-detected if omitted) |
| `-i` | `--interactive` | `INTERACTIVE` | - | Interactive terminal disambiguation prompt if query matches multiple symbols |

## Examples

::::{tab-set}
:::{tab-item} Basic Command
```bash
cartograph locate
```
:::

:::{tab-item} JSON Output
```bash
cartograph locate --json
```
:::
::::
