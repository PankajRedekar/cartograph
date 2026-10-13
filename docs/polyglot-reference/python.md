# 🐍 Python Language Reference

- **Tree-Sitter Grammar**: `tree-sitter-python`
- **Supported Versions**: Python 3.8 through 3.13+.

## AST Symbol Extraction

- **Functions & Methods**: Synchronous `def` and asynchronous `async def`.
- **Classes**: Class definitions, base classes (`class Sub(Base)`), and metaclasses.
- **Decorators**: Resolves `@staticmethod`, `@classmethod`, `@property`, and custom route decorators (e.g. `@app.get("/users")`).
- **Type Hints**: PEP 484 and PEP 526 type annotations.
