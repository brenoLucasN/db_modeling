---
status: accepted
---

# Strict DBML as canonical format

The user's file is always valid DBML that opens in dbdiagram.io without adjustments. Everything specific to the app (positions, colors, views, export options) lives in a JSON sidecar per schema (`schema.dbml.layout.json`, with keys in deterministic order). We rejected a custom language (no ecosystem) and a DBML superset (breaks compatibility). Details in `docs/02-linguagem.md`.
