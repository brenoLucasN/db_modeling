---
status: proposta
---

# Export de imagem via cena → SVG → raster

O export PNG/JPEG não captura a janela GPUI: geramos SVG a partir da mesma geometria do canvas (crate `scene`) e rasterizamos com `resvg`. Motivo: independe de janela/GPU, permite resolução arbitrária e é testável headless. Título, legenda e marca d'água opcionais são elementos da cena.

## Consequências

Manter paridade visual canvas × export (risco R8 em `docs/riscos.md`).
