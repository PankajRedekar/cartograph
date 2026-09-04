## Purpose
Traces full-stack request flows from HTTP URL patterns to handler views, controllers, business logic, and underlying database models.

## Requirements

### Requirement: End-to-End Route Tracing
The system SHALL map HTTP endpoint routes to their corresponding view functions, serializers, and database models.

#### Scenario: Listing mapped routes for an API pattern
- **WHEN** `cartograph routes "accrual"` is executed
- **THEN** cartograph displays the complete request path: URL pattern -> View Controller -> Service Layer -> Database Model.
