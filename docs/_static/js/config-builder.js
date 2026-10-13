// Cartograph Interactive Config Builder (docs/_static/js/config-builder.js)
document.addEventListener("DOMContentLoaded", function () {
  const form = document.getElementById("cartograph-config-form");
  const output = document.getElementById("cartograph-config-output");
  const copyBtn = document.getElementById("copy-config-btn");

  if (!form || !output) return;

  function updateConfig() {
    const defaultFormat = form.querySelector('input[name="format"]:checked')?.value || "xml";
    const maxTokens = form.querySelector('input[name="max_tokens"]')?.value || "2500";
    const autoInstall = form.querySelector('input[name="auto_install"]')?.checked ? "true" : "false";

    const toml = `# Generated cartograph.toml
[core]
threads = 0
cache_dir = ".cartograph"

[agent]
default_format = "${defaultFormat}"
max_tokens = ${maxTokens}
degrade_neighbor_bodies = true
enable_type_inlining = true

[skills]
preset = "standard"
auto_install_on_index = ${autoInstall}
target_ides = ["claude", "cursor", "antigravity"]
`;
    output.textContent = toml;
  }

  form.addEventListener("input", updateConfig);
  if (copyBtn) {
    copyBtn.addEventListener("click", function () {
      navigator.clipboard.writeText(output.textContent).then(() => {
        copyBtn.textContent = "✔ Copied!";
        setTimeout(() => (copyBtn.textContent = "Copy Configuration"), 2000);
      });
    });
  }

  updateConfig();
});
