# DB Modeling para VS Code

Extensão oficial do **DB Modeling** para o Visual Studio Code. Permite visualização e modelagem bidirecional de bancos de dados relacionais utilizando o formato **DBML**.

## Recursos

- **Canvas Interativo:** Renderização de tabelas, colunas, chaves primárias e relacionamentos com curvas suaves.
- **Sincronização em Tempo Real:** Edições no texto refletem no canvas, e edições visuais viram patches no documento `.dbml`.
- **Desfazer/Refazer Unificado:** Totalmente integrado ao histórico nativo de Undo/Redo (`Ctrl+Z` / `Ctrl+Y`) do VS Code.
- **Sidecar de Layout:** Preserva posições visuais no arquivo `<schema>.dbml.layout.json`.

## Como Usar

1. Abra qualquer arquivo `.dbml` no VS Code.
2. Clique no ícone de canvas no topo da aba do editor ou execute o comando `DB Modeling: Abrir no Canvas Visual` na paleta (`Ctrl+Shift+P`).
