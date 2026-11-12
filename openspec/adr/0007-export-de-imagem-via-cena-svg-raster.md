---
status: accepted
---

# Image export via scene → SVG → raster

PNG/SVG export does not capture the raw window framebuffer: we generate SVG and high-DPI raster output directly from the shared scene geometry (`layout::SceneGraph`). Reason: window/GPU independent, supports arbitrary resolution (2x+), and is testable headlessly. Optional title and color legend are rendered as scene elements.

## Consequences

Maintain visual parity between interactive canvas and exported images.
