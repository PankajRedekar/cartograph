use cartograph_core::model::{ComplexityRecord, ComplexityReport, HotspotQuadrant};

pub struct HtmlFormatter;

impl HtmlFormatter {
    /// Generate standalone, zero-dependency interactive HTML/SVG dashboard
    pub fn format(report: &ComplexityReport) -> String {
        let json_data = serde_json::to_string(report).unwrap_or_else(|_| "{}".to_string());

        let mut scatter_dots_svg = String::new();
        let chart_width = 800.0;
        let chart_height = 420.0;
        let pad_left = 60.0;
        let pad_bottom = 50.0;
        let pad_top = 20.0;
        let pad_right = 30.0;

        let plot_w = chart_width - pad_left - pad_right;
        let plot_h = chart_height - pad_top - pad_bottom;

        // Max values for plotting
        let max_cc = report
            .top_hotspots
            .iter()
            .map(|r| r.complexity.cyclomatic)
            .max()
            .unwrap_or(30)
            .max(30) as f64;

        for r in &report.top_hotspots {
            // X: churn velocity (0.0 to 1.0)
            let x = pad_left + (r.churn.churn_velocity_score.clamp(0.0, 1.0) * plot_w);
            // Y: cyclomatic (0 to max_cc) - inverted SVG Y
            let y_ratio = (r.complexity.cyclomatic as f64 / max_cc).clamp(0.0, 1.0);
            let y = pad_top + (1.0 - y_ratio) * plot_h;

            let (fill_color, stroke_color, radius) = match r.quadrant {
                HotspotQuadrant::CriticalHotspot => ("#ef4444", "#ffffff", 8.0),
                HotspotQuadrant::ChurnHotspot => ("#f59e0b", "#ffffff", 6.5),
                HotspotQuadrant::StableTechnicalDebt => ("#8b5cf6", "#ffffff", 6.5),
                HotspotQuadrant::Healthy => ("#10b981", "#ffffff", 5.0),
            };

            let title_attr = format!("{}: CC {}, Churn {:.2}", r.name, r.complexity.cyclomatic, r.churn.churn_velocity_score);

            scatter_dots_svg.push_str(&format!(
                r#"<circle cx="{:.1}" cy="{:.1}" r="{:.1}" fill="{fill_color}" fill-opacity="0.85" stroke="{stroke_color}" stroke-width="1.5" cursor="pointer" onclick="selectSymbol('{id}')"><title>{title_attr}</title></circle>"#,
                x, y, radius, id = r.id
            ));
            scatter_dots_svg.push('\n');
        }

        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>Cartograph Complexity & Hotspot Heatmap</title>
  <style>
    :root {{
      --bg-primary: #0a0d14;
      --bg-secondary: #101522;
      --bg-card: rgba(18, 24, 38, 0.7);
      --bg-card-hover: rgba(26, 35, 56, 0.85);
      --border-color: rgba(255, 255, 255, 0.08);
      --text-primary: #f1f5f9;
      --text-secondary: #94a3b8;
      --text-muted: #64748b;
      --accent-indigo: #6366f1;
      --accent-cyan: #06b6d4;
      --color-healthy: #10b981;
      --color-moderate: #f59e0b;
      --color-critical: #ef4444;
      --color-debt: #8b5cf6;
    }}
    * {{ box-sizing: border-box; margin: 0; padding: 0; }}
    body {{
      background: radial-gradient(circle at 50% 0%, #151a30 0%, var(--bg-primary) 70%);
      color: var(--text-primary);
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
      min-height: 100vh;
      line-height: 1.5;
    }}
    code, pre {{ font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace; }}
    header {{
      position: sticky; top: 0; z-index: 100;
      backdrop-filter: blur(16px); -webkit-backdrop-filter: blur(16px);
      background: rgba(10, 13, 20, 0.85);
      border-bottom: 1px solid var(--border-color);
      padding: 1rem 2rem; display: flex; justify-content: space-between; align-items: center;
    }}
    .brand {{ display: flex; align-items: center; gap: 0.75rem; }}
    .brand-logo {{
      width: 36px; height: 36px; background: linear-gradient(135deg, #6366f1, #06b6d4);
      border-radius: 10px; display: flex; align-items: center; justify-content: center; font-size: 1.25rem;
    }}
    .brand-title {{ font-size: 1.15rem; font-weight: 700; }}
    .container {{ max-width: 1400px; margin: 0 auto; padding: 2rem; }}
    .kpi-grid {{
      display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 1.25rem; margin-bottom: 2rem;
    }}
    .kpi-card {{
      background: var(--bg-card); border: 1px solid var(--border-color); border-radius: 14px;
      padding: 1.25rem; backdrop-filter: blur(12px);
    }}
    .kpi-label {{ font-size: 0.8rem; color: var(--text-muted); text-transform: uppercase; margin-bottom: 0.4rem; }}
    .kpi-value {{ font-size: 1.85rem; font-weight: 800; }}
    .card {{
      background: var(--bg-card); border: 1px solid var(--border-color); border-radius: 14px;
      padding: 1.5rem; margin-bottom: 2rem; backdrop-filter: blur(12px);
    }}
    .card-title {{ font-size: 1.1rem; font-weight: 700; margin-bottom: 1rem; display: flex; justify-content: space-between; }}
    table {{ width: 100%; border-collapse: collapse; font-size: 0.88rem; }}
    th {{ text-align: left; padding: 0.75rem 1rem; color: var(--text-muted); border-bottom: 1px solid var(--border-color); }}
    td {{ padding: 0.85rem 1rem; border-bottom: 1px solid rgba(255, 255, 255, 0.04); cursor: pointer; }}
    tr:hover td {{ background: var(--bg-card-hover); }}
    .badge {{
      display: inline-block; padding: 0.2rem 0.6rem; border-radius: 6px; font-size: 0.75rem; font-weight: 600;
    }}
    .badge-q1 {{ background: rgba(239, 68, 68, 0.2); color: #f87171; border: 1px solid rgba(239, 68, 68, 0.4); }}
    .badge-danger {{ background: rgba(220, 38, 38, 0.25); color: #fca5a5; }}
    .btn {{
      padding: 0.4rem 0.8rem; border-radius: 6px; border: 1px solid var(--accent-indigo);
      background: rgba(99, 102, 241, 0.15); color: #c7d2fe; cursor: pointer; font-size: 0.78rem; font-weight: 600;
    }}
    .btn:hover {{ background: rgba(99, 102, 241, 0.3); }}
    /* Drawer */
    #drawer {{
      position: fixed; right: -480px; top: 0; width: 460px; height: 100vh;
      background: #111625; border-left: 1px solid var(--border-color);
      box-shadow: -8px 0 32px rgba(0, 0, 0, 0.5); z-index: 200;
      padding: 2rem; overflow-y: auto; transition: right 0.3s ease;
    }}
    #drawer.open {{ right: 0; }}
    .close-btn {{ float: right; background: none; border: none; color: #fff; font-size: 1.5rem; cursor: pointer; }}
  </style>
</head>
<body>
  <header>
    <div class="brand">
      <div class="brand-logo">🧭</div>
      <div class="brand-title">Cartograph Complexity & Hotspot Heatmap</div>
    </div>
    <div style="font-size: 0.85rem; color: var(--text-muted);">Phase 2.5 • Granularity: {granularity}</div>
  </header>

  <div class="container">
    <div class="kpi-grid">
      <div class="kpi-card">
        <div class="kpi-label">Analyzed Items</div>
        <div class="kpi-value">{total_analyzed}</div>
      </div>
      <div class="kpi-card">
        <div class="kpi-label">Critical Hotspots (Q1)</div>
        <div class="kpi-value" style="color: var(--color-critical);">{critical_hotspots}</div>
      </div>
      <div class="kpi-card">
        <div class="kpi-label">Danger Zone (0 Tests)</div>
        <div class="kpi-value" style="color: var(--color-moderate);">{danger_zone}</div>
      </div>
      <div class="kpi-card">
        <div class="kpi-label">God-Classes</div>
        <div class="kpi-value" style="color: #f87171;">{god_classes}</div>
      </div>
      <div class="kpi-card">
        <div class="kpi-label">Est. Technical Debt</div>
        <div class="kpi-value" style="color: var(--color-debt);">{debt_hours:.1}h</div>
      </div>
      <div class="kpi-card">
        <div class="kpi-label">Maintainability Index</div>
        <div class="kpi-value">{avg_mi:.1}/100</div>
      </div>
    </div>

    <!-- 4-Quadrant Scatter Chart -->
    <div class="card">
      <div class="card-title">
        <span>Adam Tornhill 4-Quadrant Hotspot Matrix</span>
        <span style="font-size: 0.8rem; font-weight: normal; color: var(--text-muted);">X: Git Churn Velocity • Y: Cyclomatic Complexity</span>
      </div>
      <svg width="100%" height="420" viewBox="0 0 800 420" style="background: rgba(10, 14, 24, 0.6); border-radius: 10px;">
        <!-- Axes -->
        <line x1="60" y1="370" x2="770" y2="370" stroke="rgba(255,255,255,0.2)" stroke-width="1.5" />
        <line x1="60" y1="20" x2="60" y2="370" stroke="rgba(255,255,255,0.2)" stroke-width="1.5" />

        <!-- Quadrant Dividers -->
        <line x1="415" y1="20" x2="415" y2="370" stroke="rgba(255,255,255,0.08)" stroke-dasharray="4,4" />
        <line x1="60" y1="195" x2="770" y2="195" stroke="rgba(255,255,255,0.08)" stroke-dasharray="4,4" />

        <!-- Quadrant Labels -->
        <text x="600" y="50" fill="rgba(239,68,68,0.4)" font-size="14" font-weight="700">Q1: Critical Hotspots 🚨</text>
        <text x="600" y="340" fill="rgba(245,158,11,0.3)" font-size="14" font-weight="700">Q2: Churn Hotspots ⚠️</text>
        <text x="90" y="340" fill="rgba(16,185,129,0.3)" font-size="14" font-weight="700">Q3: Healthy Code ✅</text>
        <text x="90" y="50" fill="rgba(139,92,246,0.3)" font-size="14" font-weight="700">Q4: Stable Tech Debt ⏳</text>

        <!-- Dots -->
        {scatter_dots_svg}
      </svg>
    </div>

    <!-- Hotspots Table -->
    <div class="card">
      <div class="card-title">
        <span>Top Refactoring Priorities</span>
      </div>
      <table>
        <thead>
          <tr>
            <th>Rank</th>
            <th>Symbol / Name</th>
            <th>File Coordinates</th>
            <th>CC</th>
            <th>Cognitive</th>
            <th>LOC</th>
            <th>Tests</th>
            <th>Churn</th>
            <th>Hotspot Score</th>
            <th>Status</th>
            <th>Action</th>
          </tr>
        </thead>
        <tbody>
          {table_rows}
        </tbody>
      </table>
    </div>
  </div>

  <!-- Detail Inspection Drawer -->
  <div id="drawer">
    <button class="close-btn" onclick="closeDrawer()">&times;</button>
    <div id="drawer-content">Select a symbol to inspect details.</div>
  </div>

  <script>
    const REPORT = {json_data};

    function selectSymbol(id) {{
      const item = REPORT.top_hotspots.find(r => r.id === id);
      if (!item) return;

      const drawer = document.getElementById('drawer');
      const content = document.getElementById('drawer-content');

      let godClassHtml = '';
      if (item.god_class && item.god_class.is_god_class) {{
        godClassHtml = `
          <div style="margin-top: 1.5rem; padding: 1rem; background: rgba(239,68,68,0.1); border: 1px solid rgba(239,68,68,0.3); border-radius: 8px;">
            <div style="font-weight: 700; color: #f87171; margin-bottom: 0.5rem;">🚨 God-Class Detected (GCI: ${{item.god_class.god_class_index.toFixed(1)}})</div>
            <ul style="font-size: 0.8rem; padding-left: 1.2rem; color: #cbd5e1;">
              ${{item.god_class.triggers.map(t => `<li>${{t}}</li>`).join('')}}
            </ul>
          </div>
        `;
      }}

      content.innerHTML = `
        <h2 style="font-size: 1.3rem; margin-bottom: 0.3rem;">${{item.name}}</h2>
        <div style="font-size: 0.8rem; color: var(--text-muted); margin-bottom: 1.2rem;"><code>${{item.filepath}}:${{item.start_line}}</code></div>

        <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 0.75rem; margin-bottom: 1.5rem;">
          <div class="kpi-card" style="padding: 0.75rem;">
            <div class="kpi-label">Cyclomatic</div>
            <div style="font-size: 1.4rem; font-weight: 700;">${{item.complexity.cyclomatic}}</div>
          </div>
          <div class="kpi-card" style="padding: 0.75rem;">
            <div class="kpi-label">Cognitive</div>
            <div style="font-size: 1.4rem; font-weight: 700;">${{item.complexity.cognitive}}</div>
          </div>
          <div class="kpi-card" style="padding: 0.75rem;">
            <div class="kpi-label">Hotspot Score</div>
            <div style="font-size: 1.4rem; font-weight: 700; color: #f87171;">${{item.hotspot_score.toFixed(1)}}</div>
          </div>
          <div class="kpi-card" style="padding: 0.75rem;">
            <div class="kpi-label">Debt Hours</div>
            <div style="font-size: 1.4rem; font-weight: 700; color: var(--color-debt);">${{item.maintainability.estimated_debt_hours.toFixed(1)}}h</div>
          </div>
        </div>

        <div style="margin-bottom: 1rem; font-size: 0.85rem;">
          <div><strong>Coupling:</strong> Afferent: ${{item.coupling.in_degree}} | Efferent: ${{item.coupling.out_degree}} (Instability: ${{item.coupling.instability.toFixed(2)}})</div>
          <div><strong>Tests Linked:</strong> ${{item.test_deficit.associated_test_count}} (${{item.test_deficit.is_danger_zone ? '<span style="color:#f87171;font-weight:700;">DANGER ZONE</span>' : 'Safe'}})</div>
          <div><strong>AI Context:</strong> ~${{item.ai_friction.estimated_tokens}} tokens (CCR: ${{item.ai_friction.context_complexity_ratio.toFixed(3)}})</div>
        </div>

        ${{godClassHtml}}

        <div style="margin-top: 1.5rem;">
          <button class="btn" style="width: 100%; padding: 0.6rem;" onclick="copyPrompt('${{item.name}}')">Copy Refactoring Prompt</button>
        </div>
      `;

      drawer.classList.add('open');
    }}

    function closeDrawer() {{
      document.getElementById('drawer').classList.remove('open');
    }}

    function copyPrompt(name) {{
      navigator.clipboard.writeText(`cartograph complexity --prompt-refactor ${{name}}`);
      alert(`Refactoring command copied: cartograph complexity --prompt-refactor ${{name}}`);
    }}
  </script>
</body>
</html>"#,
            granularity = report.granularity.as_str(),
            total_analyzed = report.total_analyzed,
            critical_hotspots = report.critical_hotspots_count,
            danger_zone = report.danger_zone_count,
            god_classes = report.god_classes_detected,
            debt_hours = report.total_debt_hours,
            avg_mi = report.avg_maintainability_index,
            scatter_dots_svg = scatter_dots_svg,
            table_rows = render_table_rows(&report.top_hotspots),
            json_data = json_data
        )
    }
}

fn render_table_rows(records: &[ComplexityRecord]) -> String {
    let mut rows = String::new();
    for (idx, r) in records.iter().enumerate() {
        let quad_badge = match r.quadrant {
            HotspotQuadrant::CriticalHotspot => r#"<span class="badge badge-q1">🚨 Q1 Hotspot</span>"#,
            HotspotQuadrant::ChurnHotspot => r#"<span class="badge" style="background:rgba(245,158,11,0.2);color:#f59e0b;">⚠️ Q2 Churn</span>"#,
            HotspotQuadrant::StableTechnicalDebt => r#"<span class="badge" style="background:rgba(139,92,246,0.2);color:#c084fc;">⏳ Q4 Debt</span>"#,
            HotspotQuadrant::Healthy => r#"<span class="badge" style="background:rgba(16,185,129,0.2);color:#34d399;">✅ Q3 Healthy</span>"#,
        };

        let status = if r.test_deficit.is_danger_zone {
            r#"<span class="badge badge-danger">0 Tests ⚠️</span>"#
        } else if r.god_class.as_ref().map(|g| g.is_god_class).unwrap_or(false) {
            r#"<span class="badge" style="background:rgba(220,38,38,0.3);color:#fca5a5;">God-Class</span>"#
        } else {
            r#"<span style="font-size:0.8rem;">Normal</span>"#
        };

        rows.push_str(&format!(
            r#"<tr onclick="selectSymbol('{id}')">
              <td><strong>#{rank}</strong></td>
              <td><strong>{name}</strong></td>
              <td><code>{filepath}:{start_line}</code></td>
              <td><strong>{cc}</strong></td>
              <td>{cog}</td>
              <td>{loc}</td>
              <td>{tests}</td>
              <td>{churn:.2}</td>
              <td><strong style="color:#f87171;">{score:.1}</strong></td>
              <td>{status} {quad_badge}</td>
              <td><button class="btn" onclick="event.stopPropagation(); selectSymbol('{id}')">Inspect</button></td>
            </tr>"#,
            id = r.id,
            rank = idx + 1,
            name = r.name,
            filepath = r.filepath,
            start_line = r.start_line,
            cc = r.complexity.cyclomatic,
            cog = r.complexity.cognitive,
            loc = r.complexity.loc,
            tests = r.test_deficit.associated_test_count,
            churn = r.churn.churn_velocity_score,
            score = r.hotspot_score,
            status = status,
            quad_badge = quad_badge,
        ));
    }
    rows
}
