---
status: accepted
---

# Core free of UI dependencies

Only the `app` crate depends on GPUI; parser, analysis, editing, layout, scene, and Wasm bindings are pure Rust. Reason: fast headless testing and code reuse across targets (CLI, WebAssembly, VS Code extension).
