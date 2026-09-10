# fullstack-route-mapper Specification

## Purpose
Traces full-stack request flows from frontend UI components and HTTP URL patterns through middlewares, controller/handler views, domain business logic, and underlying database models, enabling 360° static API discovery, API contract drift verification, inverted model-to-route impact analysis, route hygiene auditing, and synthetic OpenAPI 3.1 generation in sub-3ms.

## Requirements

### Requirement: End-to-End Vertical Route Tracing
The system SHALL map HTTP endpoint routes to their corresponding middleware chain, controller/handler views, domain services, ORM data models, and database tables in under 3ms.

#### Scenario: Listing mapped routes for an API pattern
- **WHEN** `cartograph routes "checkout"` is executed
- **THEN** cartograph displays the complete request path:
  `URL Pattern -> Middlewares -> View Controller -> Service Layer -> Database Models -> SQL Tables`.

#### Scenario: Multi-hop service and database model resolution
- **GIVEN** route `POST /api/v1/invoices` maps to `InvoiceController::create`
- **AND** `InvoiceController::create` calls `BillingService::generate_invoice`
- **AND** `BillingService::generate_invoice` writes to `Invoice` model and `invoices` table
- **WHEN** tracing `POST /api/v1/invoices`
- **THEN** cartograph discovers both `Invoice` and `invoices` as touched entities at hop distance 3.

---

### Requirement: Frontend-to-Backend 360° Call Linking
The system SHALL identify clientside API invocations (`fetch`, `axios`, `@tanstack/react-query`, Next.js Server Actions) in TypeScript/JavaScript frontend files and link calling UI components directly to corresponding server route nodes.

#### Scenario: Finding API routes called by a frontend React component
- **GIVEN** a React component `CheckoutButton` calling `fetch('/api/v1/checkout', { method: 'POST' })`
- **WHEN** `cartograph routes --component CheckoutButton` is executed
- **THEN** cartograph outputs the mapped route `POST /api/v1/checkout` and its entire backend execution trace down to `Invoice` model.

#### Scenario: Pinpointing frontend callers of an API route
- **GIVEN** route `POST /api/v1/checkout` called by `src/components/CheckoutModal.tsx` and `src/pages/QuickPay.tsx`
- **WHEN** `cartograph routes "checkout" --show-callers` is executed
- **THEN** cartograph lists `CheckoutModal` and `QuickPay` with exact file and line coordinates.

---

### Requirement: API Contract Drift & Breaking Change Verification
The system SHALL compare route signatures across git branches or pull requests to detect breaking changes (removed endpoints, altered methods, newly mandatory parameters, breaking DTO response schemas) and enforce CI quality gates.

#### Scenario: Detecting a removed public API endpoint in a PR branch
- **GIVEN** route `DELETE /api/v1/tokens/{id}` removed in the current PR branch
- **WHEN** `cartograph routes --verify-contracts --branch "origin/main...HEAD"` is executed
- **THEN** cartograph flags the breaking change `RouteRemoved` and exits with code 1 if `--fail-on-breaking` is specified.

#### Scenario: Detecting newly added mandatory parameters without defaults
- **GIVEN** endpoint `POST /api/v1/checkout` adds required path parameter `{currency}` without a default
- **WHEN** running contract verification against the base branch
- **THEN** cartograph reports `NewRequiredParam` as a breaking contract modification.

---

### Requirement: Polyglot AST Route Extraction
The system SHALL identify and parse route declarations statically across TypeScript/JavaScript, Python, Go, Rust, and Ruby without executing application code.

#### Scenario: TypeScript / Express route extraction
- **WHEN** parsing Express.js source code containing `router.post('/checkout', authMiddleware, checkoutHandler)`
- **THEN** cartograph creates a `NodeKind::Route` node with method `POST`, path `/checkout`, links `authMiddleware` as middleware, and maps to `checkoutHandler`.

#### Scenario: Next.js App Router route extraction
- **WHEN** scanning a Next.js App Router repository with `app/api/v1/users/[id]/route.ts`
- **THEN** cartograph normalizes the file path to `/api/v1/users/{id}` and maps exported functions `GET`, `POST`, and `DELETE` to distinct HTTP route nodes.

#### Scenario: Python FastAPI route extraction
- **WHEN** parsing Python code with `@router.post("/items/{item_id}", dependencies=[Depends(verify_token)])`
- **THEN** cartograph extracts the route with method `POST`, path `/items/{item_id}`, registers `verify_token` as authentication middleware, and extracts `item_id` as a path parameter.

#### Scenario: Ruby on Rails resource route expansion
- **WHEN** parsing Rails `config/routes.rb` containing `resources :invoices`
- **THEN** cartograph expands all 7 RESTful actions (`index`, `new`, `create`, `show`, `edit`, `update`, `destroy`) linking to corresponding `InvoicesController` actions.

#### Scenario: Go and Rust web router extraction
- **WHEN** parsing Go code with Gin `r.GET("/users/:id", getUser)` or Rust code with Axum `Router::new().route("/users/:id", get(get_user))`
- **THEN** cartograph extracts the canonical route `/users/{id}` and links to the respective handler function.

---

### Requirement: Universal URL Normalization & Hierarchical Prefix Assembly
The system SHALL normalize heterogeneous path variable syntax (`:id`, `<int:id>`, `[id]`, `{id}`) into canonical RFC 6570 URI templates (`{id}`) and concatenate nested router mount prefixes.

#### Scenario: Parameter syntax normalization
- **WHEN** parsing `/api/users/:id`, `/api/users/<int:id>`, and `/api/users/[id]`
- **THEN** cartograph normalizes all representations to `/api/users/{id}`.

#### Scenario: Nested router prefix concatenation
- **GIVEN** root router mounts `app.use('/api/v1', v1Router)`
- **AND** sub-router mounts `v1Router.use('/billing', billingRouter)`
- **AND** route definition `billingRouter.post('/checkout', handler)`
- **WHEN** assembling route URLs
- **THEN** cartograph produces the fully qualified path `/api/v1/billing/checkout`.

---

### Requirement: Inverted Model-to-Route Impact Mapping
The system SHALL support reverse queries from ORM database models or SQL tables upstream to all public HTTP endpoints that read or mutate them.

#### Scenario: Finding all routes touching a specific database model
- **WHEN** `cartograph routes --model Invoice` is executed
- **THEN** cartograph traverses reverse call and mapping edges from `Invoice` to identify all ingress routes (`POST /api/v1/invoices`, `GET /api/v1/invoices/{id}`, `GET /admin/billing`).

#### Scenario: Finding all routes touching a SQL table
- **WHEN** `cartograph routes --table invoices` is executed
- **THEN** cartograph discovers all API routes that execute queries against the `invoices` table.

---

### Requirement: Route Hygiene, Shadowing & Security Auditing
The system SHALL detect dead routes, orphaned handlers, route shadowing collisions, and unauthenticated mutating routes.

#### Scenario: Detecting dead or dangling routes
- **GIVEN** route `path('payments/', views.process_payment)` where `process_payment` does not exist
- **WHEN** running `cartograph routes --dead`
- **THEN** cartograph flags the route with `DanglingHandler` warning.

#### Scenario: Detecting shadowed route patterns
- **GIVEN** route `/api/v1/users/{id}` registered before route `/api/v1/users/me` in a non-radix router
- **WHEN** running `cartograph routes --shadowed`
- **THEN** cartograph flags `/api/v1/users/me` as shadowed by `/api/v1/users/{id}`.

#### Scenario: Auditing unauthenticated mutations
- **GIVEN** route `DELETE /api/v1/users/{id}` without any authentication middleware or decorator
- **WHEN** running `cartograph routes --public` or `cartograph routes --issues`
- **THEN** cartograph flags the endpoint with `UnauthenticatedMutation` security alert.

---

### Requirement: Synthetic OpenAPI 3.1 Generation & Presentation
The system SHALL synthesize valid OpenAPI 3.1 specifications and provide ANSI terminal, interactive TUI, Markdown with Mermaid sequence diagrams, JSON, and AI Agent XML outputs.

#### Scenario: Exporting OpenAPI 3.1 specification
- **WHEN** `cartograph routes --format openapi` is executed
- **THEN** cartograph outputs a valid OpenAPI 3.1 JSON/YAML schema containing all paths, operations, parameters, docstring summaries, and linked schema models.

#### Scenario: Auto-generating Mermaid sequence diagrams
- **WHEN** `cartograph routes "checkout" --format markdown` is executed
- **THEN** cartograph renders a Markdown report containing a Mermaid `sequenceDiagram` showing the end-to-end request flow from Client through Router, Middlewares, Controller, Service, and Database.
