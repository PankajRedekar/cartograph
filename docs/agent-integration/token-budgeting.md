# 🪙 Token Budgeting & Context Economization

Enforce hard token ceilings across any Cartograph command using `--max-tokens <N>`:

```bash
cartograph locate "authenticate" --max-tokens 1500 --format xml
```

## Degradation Behavior

1. **Root Definition**: 100% full implementation preserved.
2. **Inlined Type Contracts**: All parameter and return types preserved.
3. **Neighbor Callers/Callees**: Function bodies collapsed into typed signatures:
   ```rust
   fn verify_token(raw: &str) -> Result<User> /* [34 lines collapsed] */
   ```
4. **Extreme Budget Starvation**: If the budget is exhausted, Cartograph preserves the root signature and appends `warning="budget_exhausted"`, never truncating malformed XML tags.
