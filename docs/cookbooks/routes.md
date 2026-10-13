# 🌐 `cartograph routes` Cookbook

The `cartograph routes` command traces **full-stack HTTP endpoints** from frontend UI callers through route handlers, middlewares, and ORM models down to SQL database tables.

## Common Workflows

### 1. Endpoint Discovery & Handler Tracing

Inspect an endpoint's complete downstream execution path:

::::{tab-set}
:::{tab-item} CLI Command
:sync: cli

```bash
cartograph routes "/api/v1/users/:id" --show-callers
```
```

:::{tab-item} Python SDK
:sync: python

```python
import cartograph

with cartograph.open() as cg:
    routes = cg.routes(pattern="/api/v1/users/:id")
    for r in routes:
        print(f"{r.method} {r.path} -> Handler: {r.handler} (Model: {r.orm_model})")
```
```

:::{tab-item} AI Agent Skill
:sync: agent

```markdown
<!-- Agent Directive: Trace full-stack route dependencies -->
Invoke: `cartograph routes "<PATTERN>" --format xml`
Verify: Review middlewares and downstream database tables touched by endpoint.
```
```

:::{tab-item} Structured XML Context
:sync: xml

```xml
<route_map method="GET" path="/api/v1/users/:id" framework="actix-web">
  <handler file="src/routes/users.rs" line="54" symbol="get_user_handler"/>
  <middlewares>
    <middleware name="JwtAuthMiddleware"/>
    <middleware name="RateLimiter"/>
  </middlewares>
  <orm_model name="User" table="users"/>
</route_map>
```
```
````

### 2. Inverted ORM Model & Table Lookup

Find all HTTP routes that touch a specific database table:

```bash
cartograph routes --table "orders" --method POST
```

### 3. API Contract Breaking Change Gate

Verify route contracts against a base branch:

```bash
cartograph routes --verify-contracts --branch "origin/main...HEAD" --fail-on-breaking
```
