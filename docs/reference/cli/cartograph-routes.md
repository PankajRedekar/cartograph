# `cartograph routes`

Map full-stack API routes, frontend caller linkings, and backend dependencies (<15ms)

## Synopsis

```bash
cartograph routes [OPTIONS] [<pattern>]
```

## Arguments

| Argument | Description | Required |
| :--- | :--- | :--- |
| `<pattern>` | Optional route path pattern filter (e.g. "/api/v1/users", "/orders/:id") | No |

## Options & Flags

| Flag | Name | Value | Default | Description |
| :--- | :--- | :--- | :--- | :--- |
| `-p` | `--path` | `PATH` | `.` | Root directory of repository |
| `-d` | `--db` | `DB` | `default` | Database profile or SQLite file path |
| `-m` | `--method` | `METHOD` | - | Filter by HTTP method (GET, POST, PUT, DELETE, PATCH) |
| - | `--component` | `COMPONENT` | - | Filter routes invoked by a frontend UI component (e.g. "CheckoutButton") |
| - | `--show-callers` | `SHOW_CALLERS` | - | Show clientside 360° UI component callers for matched routes |
| - | `--model` | `MODEL` | - | Inverted lookup: filter routes that read or modify a specific ORM model (e.g. "Order") |
| - | `--table` | `TABLE` | - | Inverted lookup: filter routes that touch a specific database table (e.g. "orders") |
| - | `--handler` | `HANDLER` | - | Filter by handler function or controller name |
| - | `--framework` | `FRAMEWORK` | - | Filter by web framework (e.g. "express", "fastapi", "nextjs", "rails", "axum") |
| - | `--auth-only` | `AUTH_ONLY` | - | Show only authenticated / protected endpoints |
| - | `--public-only` | `PUBLIC_ONLY` | - | Show only public / unauthenticated endpoints |
| - | `--issues-only` | `ISSUES_ONLY` | - | Show only routes with health/security issues (shadowing, unauthenticated mutations, dangling) |
| - | `--dead-only` | `DEAD_ONLY` | - | Show only dead/dangling routes (no controller handler found) |
| - | `--shadowed-only` | `SHADOWED_ONLY` | - | Show only shadowed routes (masked by an earlier route pattern) |
| - | `--verify-contracts` | `VERIFY_CONTRACTS` | - | Verify API contract drift and breaking changes across git branches |
| - | `--branch` | `BRANCH` | - | Git branch diff range for contract verification (default: "main...HEAD") |
| - | `--fail-on-breaking` | `FAIL_ON_BREAKING` | - | CI/CD Gate: fail with exit code 1 if breaking contract changes are detected |
| - | `--openapi` | `OPENAPI` | - | Export discovered routes as OpenAPI 3.1.0 specification |
| `-D` | `--depth` | `DEPTH` | `5` | Maximum downstream traversal depth (default: 5) |
| `-f` | `--format` | `FORMAT` | - | Presentation output format: terminal, markdown, json, xml (auto-detected if omitted) |
| - | `--json` | `JSON` | - | Output results formatted as JSON (shorthand for --format json) |
| `-i` | `--interactive` | `INTERACTIVE` | - | Interactive terminal mode |

## Examples

::::{tab-set}
:::{tab-item} Basic Command
```bash
cartograph routes
```
:::

:::{tab-item} JSON Output
```bash
cartograph routes --json
```
:::
::::
