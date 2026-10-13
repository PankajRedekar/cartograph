// Cartograph Interactive SVG Graph Explorer (docs/_static/js/graph-explorer.js)
document.addEventListener("DOMContentLoaded", function () {
  const container = document.getElementById("living-graph-explorer");
  if (!container) return;

  let zoomLevel = 1.0;
  const svg = container.querySelector("svg");
  if (!svg) return;

  const zoomInBtn = document.getElementById("graph-zoom-in");
  const zoomOutBtn = document.getElementById("graph-zoom-out");
  const resetBtn = document.getElementById("graph-reset");

  function applyTransform() {
    svg.style.transform = `scale(${zoomLevel})`;
    svg.style.transformOrigin = "center center";
    svg.style.transition = "transform 0.2s ease";
  }

  if (zoomInBtn) {
    zoomInBtn.addEventListener("click", () => {
      zoomLevel = Math.min(zoomLevel + 0.2, 2.5);
      applyTransform();
    });
  }

  if (zoomOutBtn) {
    zoomOutBtn.addEventListener("click", () => {
      zoomLevel = Math.max(zoomLevel - 0.2, 0.4);
      applyTransform();
    });
  }

  if (resetBtn) {
    resetBtn.addEventListener("click", () => {
      zoomLevel = 1.0;
      applyTransform();
    });
  }
});
