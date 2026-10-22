use super::mermaid::MermaidRenderer;
use super::svg::SvgRenderer;
use cartograph_core::diagram::{DiagramGraph, DiagramTheme};

pub struct HtmlRenderer;

impl HtmlRenderer {
    pub fn render(graph: &DiagramGraph, theme: DiagramTheme) -> String {
        let svg_content = SvgRenderer::render(graph, theme);
        let mermaid_content = MermaidRenderer::render(graph);
        let title = &graph.title;

        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{title} - Cartograph</title>
  <style>
    * {{ box-sizing: border-box; margin: 0; padding: 0; }}
    body {{
      font-family: Inter, system-ui, -apple-system, sans-serif;
      background-color: #0f172a;
      color: #f8fafc;
      overflow: hidden;
      height: 100vh;
      display: flex;
      flex-direction: column;
    }}
    header {{
      background-color: #1e293b;
      border-bottom: 1px solid #334155;
      padding: 12px 24px;
      display: flex;
      align-items: center;
      justify-content: space-between;
      z-index: 10;
    }}
    .title-group h1 {{ font-size: 16px; font-weight: 700; color: #f8fafc; }}
    .title-group p {{ font-size: 11px; color: #94a3b8; }}
    .controls {{ display: flex; gap: 10px; align-items: center; }}
    .search-input {{
      background: #0f172a;
      border: 1px solid #334155;
      color: #f8fafc;
      padding: 6px 12px;
      border-radius: 6px;
      font-size: 13px;
      outline: none;
      width: 220px;
    }}
    .search-input:focus {{ border-color: #6366f1; }}
    .btn {{
      background: #334155;
      color: #f8fafc;
      border: none;
      padding: 6px 12px;
      border-radius: 6px;
      font-size: 12px;
      font-weight: 500;
      cursor: pointer;
      transition: background 0.15s ease;
    }}
    .btn:hover {{ background: #475569; }}
    .btn-primary {{ background: #6366f1; }}
    .btn-primary:hover {{ background: #4f46e5; }}
    #viewport {{
      flex: 1;
      overflow: hidden;
      position: relative;
      cursor: grab;
      user-select: none;
    }}
    #viewport:active {{ cursor: grabbing; }}
    #canvas-container {{
      width: 100%;
      height: 100%;
      display: flex;
      align-items: center;
      justify-content: center;
      transform-origin: center center;
    }}
    svg {{ width: 100%; height: 100%; }}
  </style>
</head>
<body>
  <header>
    <div class="title-group">
      <h1>📊 {title}</h1>
      <p>Cartograph Visual Architecture Diagram • {nodes_count} nodes • {edges_count} edges</p>
    </div>
    <div class="controls">
      <input type="text" id="search-box" class="search-input" placeholder="Search symbols... (/)" />
      <button class="btn" onclick="zoomIn()">➕ Zoom In</button>
      <button class="btn" onclick="zoomOut()">➖ Zoom Out</button>
      <button class="btn" onclick="resetView()">🔄 Reset</button>
      <button class="btn btn-primary" onclick="copyMermaid()">📋 Copy Mermaid</button>
    </div>
  </header>
  <main id="viewport">
    <div id="canvas-container">
      {svg_content}
    </div>
  </main>

  <script>
    let scale = 1.0;
    let panX = 0;
    let panY = 0;
    let isDragging = false;
    let startX = 0;
    let startY = 0;

    const container = document.getElementById('canvas-container');
    const viewport = document.getElementById('viewport');

    function updateTransform() {{
      container.style.transform = `translate(${{panX}}px, ${{panY}}px) scale(${{scale}})`;
    }}

    function zoomIn() {{
      scale = Math.min(scale * 1.25, 4.0);
      updateTransform();
    }}

    function zoomOut() {{
      scale = Math.max(scale / 1.25, 0.2);
      updateTransform();
    }}

    function resetView() {{
      scale = 1.0;
      panX = 0;
      panY = 0;
      updateTransform();
    }}

    viewport.addEventListener('wheel', (e) => {{
      e.preventDefault();
      const zoomFactor = e.deltaY < 0 ? 1.1 : 0.9;
      scale = Math.min(Math.max(scale * zoomFactor, 0.2), 4.0);
      updateTransform();
    }}, {{ passive: false }});

    viewport.addEventListener('mousedown', (e) => {{
      if (e.button === 0) {{
        isDragging = true;
        startX = e.clientX - panX;
        startY = e.clientY - panY;
      }}
    }});

    window.addEventListener('mousemove', (e) => {{
      if (isDragging) {{
        panX = e.clientX - startX;
        panY = e.clientY - startY;
        updateTransform();
      }}
    }});

    window.addEventListener('mouseup', () => {{
      isDragging = false;
    }});

    const searchBox = document.getElementById('search-box');
    searchBox.addEventListener('input', (e) => {{
      const query = e.target.value.toLowerCase().trim();
      const nodes = document.querySelectorAll('.node-group');
      nodes.forEach(node => {{
        const title = node.querySelector('title')?.textContent.toLowerCase() || '';
        const rect = node.querySelector('.node-box');
        if (query && title.includes(query)) {{
          rect.style.stroke = '#22c55e';
          rect.style.strokeWidth = '3px';
        }} else {{
          rect.style.stroke = '';
          rect.style.strokeWidth = '';
        }}
      }});
    }});

    window.addEventListener('keydown', (e) => {{
      if (e.key === '/' && document.activeElement !== searchBox) {{
        e.preventDefault();
        searchBox.focus();
      }}
    }});

    const mermaidSource = {mermaid_json};
    function copyMermaid() {{
      navigator.clipboard.writeText(mermaidSource).then(() => {{
        alert('Mermaid syntax copied to clipboard!');
      }});
    }}
  </script>
</body>
</html>
"#,
            title = html_escape(title),
            nodes_count = graph.nodes.len(),
            edges_count = graph.edges.len(),
            svg_content = svg_content,
            mermaid_json =
                serde_json::to_string(&mermaid_content).unwrap_or_else(|_| "\"\"".to_string()),
        )
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
