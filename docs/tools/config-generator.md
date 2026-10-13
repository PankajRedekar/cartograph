# ⚙️ Visual `cartograph.toml` Configurator

Configure your project settings below to generate an optimized `cartograph.toml` file in real time.

<div style="margin: 20px 0; padding: 24px; border: 1px solid var(--color-background-border); border-radius: 8px; background: var(--color-background-secondary);">
  <form id="cartograph-config-form">
    <div style="margin-bottom: 16px;">
      <label style="font-weight: bold; display: block; margin-bottom: 6px;">Default Agent Output Format:</label>
      <label style="margin-right: 16px;"><input type="radio" name="format" value="xml" checked> Structured XML (&lt;cartograph_context&gt;)</label>
      <label><input type="radio" name="format" value="json"> JSON</label>
    </div>

    <div style="margin-bottom: 16px;">
      <label style="font-weight: bold; display: block; margin-bottom: 6px;">AI Agent Token Budget Ceiling:</label>
      <input type="range" name="max_tokens" min="1000" max="8000" step="500" value="2500" style="width: 280px;" oninput="this.nextElementSibling.value = this.value">
      <output style="font-weight: bold; margin-left: 10px;">2500</output> tokens
    </div>

    <div style="margin-bottom: 16px;">
      <label style="font-weight: bold; display: block; margin-bottom: 6px;">Auto-Install Skills on Index:</label>
      <label><input type="checkbox" name="auto_install" value="true"> Auto-scaffold skills when <code>cartograph index</code> runs</label>
    </div>
  </form>

  <div style="margin-top: 24px;">
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
      <strong>Generated cartograph.toml:</strong>
      <button id="copy-config-btn" class="sd-btn sd-btn-primary" style="padding: 6px 14px; border-radius: 4px; cursor: pointer;">Copy Configuration</button>
    </div>
    <pre id="cartograph-config-output" style="padding: 16px; background: var(--color-background-primary); border: 1px solid var(--color-background-border); border-radius: 6px; font-family: monospace;"></pre>
  </div>
</div>
