---
status: proposta
---

# Núcleo sem dependência de UI

Só o crate `app` depende de GPUI; parser, análise, edição, layout, cena e export são Rust puro. Motivo: testes rápidos e headless, e reaproveitamento (CLI, LSP standalone).
