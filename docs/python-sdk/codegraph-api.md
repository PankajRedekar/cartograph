# 🔌 `CodeGraph` & `AsyncCodeGraph` Lifecycles

The Python SDK exposes two primary interfaces: `CodeGraph` for synchronous code and data science notebooks, and `AsyncCodeGraph` for asynchronous frameworks.

## Synchronous `CodeGraph`

```python
import cartograph

# Context manager guarantees cleanup
with cartograph.open(path=".", profile="default") as cg:
    # 1. Inspect graph scale
    print(f"Total symbols: {cg.node_count()}, edges: {cg.edge_count()}")

    # 2. Query symbol
    node = cg.get_symbol("authenticate")
    print(f"File: {node.file_path}, Line: {node.start_line}")

    # 3. Graph traversal
    callers = cg.get_callers("authenticate")
    print(f"Direct callers: {[c.name for c in callers]}")
```

### GIL Release Mechanics
Every heavy graph traversal or AST parsing operation invokes Rust's `py.allow_threads()`. Multiple Python worker threads (e.g. in Gunicorn or Celery) can run parallel Cartograph queries without blocking the Python runtime.

## Asynchronous `AsyncCodeGraph`

```python
import asyncio
from cartograph.aio import AsyncCodeGraph

async def analyze():
    async with AsyncCodeGraph.open() as acg:
        impact = await acg.impact("authenticate")
        print(f"Risk score: {impact.risk_score}")

asyncio.run(analyze())
```
