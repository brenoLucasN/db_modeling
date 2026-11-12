---
status: accepted
---

# Custom parser with lossless CST

We use `logos` + a recursive-descent parser with error recovery + `rowan` CST instead of `dbml-rs`, because the editor requires error tolerance, accurate byte spans, and editing without losing comments or formatting. `dbml-rs` is used only as a conformance oracle in tests.
