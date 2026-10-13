---
name: cartograph-routes
description: Full-stack HTTP route tracer mapping endpoints from URL patterns through middlewares and handlers down to ORM models and database tables.
schema_version: "2.0"
cartograph_version: "0.2.0"
content_hash: "sha256:routes_v2_gate_hash_020"
triggers:
  - "what handler serves /api/v1/users"
  - "find route for checkout"
  - "trace route to database table"
  - "list all public API routes"
---

# Cartograph Routes

## Overview
Traces full-stack HTTP endpoints from URL route pattern through middlewares, controller handlers, and service layers down to ORM models and database tables.

## Command Invocation
```bash
# Trace a specific route pattern:
cartograph routes --pattern "/api/v1/auth" --format xml

# Detect dead, broken, or shadowed routes:
cartograph routes --issues-only --format xml

# Synthesize OpenAPI specification:
cartograph routes --openapi
```

## Output Schema
Returns `<route_trace>` containing:
- `<endpoint>`: HTTP method, URL pattern, and source declaration.
- `<middlewares>`: Middleware pipeline in execution order.
- `<handler>`: Controller function name, file, and line location.
- `<database_models>`: Downstream ORM entities and database tables touched.

## Autonomous Decision Checklist
- [ ] Run `cartograph routes --pattern "<url>"` when investigating API behavior.
- [ ] Inspect the middleware stack for authentication and rate-limiting guards.
- [ ] Check downstream database models before altering request/response schemas.
