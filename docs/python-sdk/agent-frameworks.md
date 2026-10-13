# 🤖 AI Agent Framework Integrations (LangChain, LlamaIndex)

Integrate Cartograph directly into multi-agent workflows as a native tool primitive:

## LangChain Tool Integration

```python
from langchain.tools import tool
import cartograph

@tool
def code_graph_locate(symbol: str) -> str:
    """Locate symbol definition, physical coordinates, and callers in 1 turn."""
    with cartograph.open() as cg:
        res = cg.locate(symbol)
        return f"File: {res.root.file_path}:{res.root.start_line} Callers: {len(res.callers)}"

@tool
def code_graph_impact(symbol: str) -> str:
    """Calculate blast radius and breaking change risk score for a symbol."""
    with cartograph.open() as cg:
        res = cg.impact(symbol)
        return f"Risk: {res.risk_score}, Breaking: {res.has_breaking_changes}"
```
