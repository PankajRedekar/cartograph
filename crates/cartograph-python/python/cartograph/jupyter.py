"""Jupyter and IPython display integrations for cartograph."""
from __future__ import annotations

import html
from typing import Any


def render_mermaid_html(mermaid_code: str) -> str:
    """Render a mermaid diagram inside Jupyter / IPython notebook via HTML."""
    escaped = html.escape(mermaid_code)
    return f"""
    <div class="cartograph-mermaid-container" style="background: #1e1e2e; padding: 16px; border-radius: 8px;">
        <pre class="mermaid" style="background: transparent;">
{escaped}
        </pre>
        <script type="module">
            import mermaid from 'https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.esm.min.mjs';
            mermaid.initialize({{ startOnLoad: true, theme: 'dark' }});
            mermaid.run();
        </script>
    </div>
    """


class RichMermaidDisplay:
    """Wrapper that displays mermaid diagrams natively in Jupyter."""

    def __init__(self, content: str):
        self.content = content

    def _repr_html_(self) -> str:
        return render_mermaid_html(self.content)

    def __str__(self) -> str:
        return self.content

    def __repr__(self) -> str:
        return self.content
