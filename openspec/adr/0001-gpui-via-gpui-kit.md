---
status: proposta
---

# GPUI consumido via gpui-kit

Queremos UI nativa de alta performance em Windows e Ubuntu. Decidimos usar GPUI (`gpui-pre`, snapshots oficiais da Zed) através do `gpui-kit`/`gpui-component` da Longbridge, com versões exatas fixadas, deixando o `gpui-kit` ditar a versão do GPUI. O crate `gpui` no crates.io está parado desde 2025-10, por isso não é usado.

## Consequências

A API do GPUI quebra com frequência: upgrades em lote e GPUI isolado no crate `app` (ver [0005](0005-nucleo-sem-dependencia-de-ui.md)).
