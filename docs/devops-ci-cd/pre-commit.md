# 🪝 Local Git Pre-Commit Hooks

Enforce atomic AST syntax and contract checks before developers can commit code:

```yaml
# .pre-commit-config.yaml
repos:
  - repo: local
    hooks:
      - id: cartograph-verify
        name: Cartograph Pre-Commit Gate
        entry: cartograph verify --staged --fail-fast
        language: system
        pass_filenames: false
        stages: [commit]
```
