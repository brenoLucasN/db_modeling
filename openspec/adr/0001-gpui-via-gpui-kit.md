---
status: accepted
---

# GPUI consumed via gpui-kit

We want a high-performance native UI on Windows, Linux, and macOS. We decided to use GPUI (`gpui-pre`, official Zed snapshots) via Longbridge's `gpui-kit`/`gpui-component`, with exact pinned versions, letting `gpui-kit` dictate the GPUI version. The `gpui` crate on crates.io has been inactive since 2025-10 and is therefore not used.

## Consequences

The GPUI API changes frequently: upgrades are done in batches and GPUI is isolated inside the `app` crate (see [0005](0005-nucleo-sem-dependencia-de-ui.md)).
