---
status: accepted
---

# Text as the single path of change

Edits made on the canvas become semantic commands that generate minimal textual patches in DBML; the `Schema` is always derived from the text. This gives undo/redo, autosave, file watching, and diagnostics a single unified pipeline. The alternative (in-memory model as source and regenerated text) would lose user comments and formatting.

## Consequences

`dbml-edit` is complex and requires strong property tests (text ↔ UI round-trip).
