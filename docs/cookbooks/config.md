# ⚙️ `cartograph config` Cookbook

The `cartograph config` command validates, generates, and inspects `cartograph.toml` project configuration files.

## Common Workflows

### 1. Initializing a Starter Configuration File

Scaffold a fresh `cartograph.toml` tailored to Clean Architecture:

```bash
cartograph config init --preset clean-architecture
```

### 2. Validating Configuration Syntax & Rules

Validate syntax, exclude globs, and architectural boundary rules:

```bash
cartograph config validate
```

### 3. Displaying Currently Resolved Project Configuration

Output resolved project settings formatted as JSON:

```bash
cartograph config show --json
```
