# 📓 Interactive Jupyter Notebook Visualization

`cartograph` includes native `_repr_html_` and `_repr_svg_` hooks that automatically render rich visual diagrams inside Jupyter Notebook, VS Code, and Google Colab cells.

## Rendering Code Subgraphs in Jupyter

```python
import cartograph

cg = cartograph.open()
# Evaluating a diagram object in a notebook cell renders an interactive SVG/HTML diagram!
cg.diagram(query="authenticate", diagram_type="callgraph")
```

The cell outputs an interactive, pan-and-zoom SVG diagram with syntax-highlighted node labels and caller/callee edges.
