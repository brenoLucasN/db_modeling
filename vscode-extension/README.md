# DB Modeling for VS Code

Official **DB Modeling** extension for Visual Studio Code. Provides visual, bidirectional relational database modeling powered by **DBML (Database Markup Language)**.

## Features

- **Interactive Canvas:** Render tables, columns, primary/foreign keys, groups, and orthogonal relationships.
- **Real-Time Bidirectional Sync:** Text edits update the canvas immediately, and visual edits generate minimal textual patches in your `.dbml` document.
- **Unified Undo/Redo:** Fully integrated with VS Code's native Undo/Redo stack (`Ctrl+Z` / `Ctrl+Y`).
- **Deterministic Layout Sidecar:** Preserves visual positions and colors in `<schema>.dbml.layout.json`.

## How to Use

1. Open any `.dbml` file in VS Code.
2. Click the canvas icon in the editor title bar or run `DB Modeling: Open in Visual Canvas` from the Command Palette (`Ctrl+Shift+P`).
