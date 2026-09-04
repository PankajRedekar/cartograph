## Purpose
Performs blast-radius impact analysis across symbols, files, git branches, or pull requests to map downstream dependencies and flag potential breaking changes.

## Requirements

### Requirement: Symbol and File Blast-Radius Mapping
The system SHALL traverse incoming and outgoing graph edges from a modified symbol or file to list all affected downstream components, views, models, and tests.

#### Scenario: Calculating impact for a modified database model
- **WHEN** `cartograph impact "AccrualPeriod"` is run
- **THEN** cartograph outputs all views, serializers, background tasks, and unit tests that consume `AccrualPeriod`.

### Requirement: Git Diff Branch Impact Analysis
The system SHALL analyze modified files and symbols in a git branch diff against the target branch to map full-stack blast radius.

#### Scenario: Analyzing uncommitted changes or branch diff
- **WHEN** `cartograph impact --branch "origin/main...HEAD"` is executed
- **THEN** cartograph computes the collective blast radius of all changed symbols across the diff.
