---
status: proposta
---

# DBML estrito como formato canônico

O arquivo do usuário é sempre DBML válido, que abre no dbdiagram sem ajustes. Tudo o que é do app (posições, cores, views, opções de export) vai num sidecar JSON por schema (`schema.dbml.layout.json`, chaves em ordem estável). Rejeitamos linguagem própria (sem ecossistema) e superset de DBML (quebra compatibilidade). Detalhes em `docs/02-linguagem.md`.
