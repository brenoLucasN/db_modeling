---
status: aceita
---

# Escopo: só modelagem visual

O produto não conecta nem lê bancos e não faz SQL (export, import ou migrações). Exports são PNG, JPEG e Mermaid `erDiagram`. Alvos: Windows 10/11 e Ubuntu 22.04/24.04; macOS fora por ora.

## Consequências

Stack menor (sem `sqlparser`/`sqlx`); o catálogo de tipos é genérico, não amarrado a dialeto; o investimento vai para canvas, edição e export.
