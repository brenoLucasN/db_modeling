# AGENTS.md — DB Modeling

Instruções de contexto, diretrizes de arquitetura, vocabulário e padrões de desenvolvimento para agentes de IA operando no projeto **DB Modeling**.

---

## 1. Visão Geral e Escopo do Produto

**DB Modeling** é um aplicativo desktop nativo e ecossistema de ferramentas para modelagem visual de bancos de dados (na linha de dbdiagram.io, DrawSQL e Azimutt), com núcleo em **Rust**, focado em alta performance (120 fps) e fluidez de UX/UI.

* **Conceito Central:** O documento de texto (DBML) e o diagrama visual (Canvas) são **duas visões sincronizadas do mesmo documento**. Alterações no canvas viram patches no texto e vice-versa.
* **Modalidades de Distribuição:**
  1. **App Desktop Nativo (GPUI):** Alta performance via GPU (DirectX no Windows, Vulkan/Wayland/X11 no Linux, Metal no macOS).
  2. **Extensão VS Code Oficial (`.vsix`):** Editor visual customizado (`CustomTextEditor`) integrado com Webview Canvas 2D e núcleo Rowan/CST rodando em WebAssembly localmente.
  3. **CLI Web Server (`dbml run` / `dbml serve`):** Servidor HTTP/WebSocket local ultraleve em Axum com auto-reload e sincronização contínua com o editor de código externo do usuário.
* **Plataformas Alvo:** Windows 10/11, Ubuntu 22.04/24.04, Fedora 40/41 e macOS (Apple Silicon e Intel).
* **Dentro do Escopo:**
  * Modelagem visual e textual bidirecional completa.
  * Formato texto canônico: **DBML estrito e válido**.
  * Sidecar de layout em JSON (`<schema>.dbml.layout.json`).
  * Exportação de diagramas para **PNG, JPEG, SVG vetorial e Mermaid `erDiagram`**.
  * Teclado primeiro, paleta de comandos (`Ctrl+K`), edição inline (sem diálogos modais).
  * Instaladores profissionais (Wizards) com resolução de dependências, atualização in-loco e desinstalação limpa.
* **Fora do Escopo:**
  * Conexão/leitura direta de banco de dados, introspecção ao vivo.
  * Exportação/importação de SQL, engenharia reversa e migrações.

---

## 2. Princípios Arquiteturais Inegociáveis

1. **Texto DBML como Fonte da Verdade Persistida:** O arquivo `.dbml` do usuário deve permanecer sempre DBML válido. Metadados do app (posições, cores, zoom, views) ficam obrigatoriamente no sidecar `<nome>.dbml.layout.json`.
2. **CST (Rowan) como Fonte da Verdade em Memória:** O parser gera uma árvore sintática sem perdas (CST) contendo espaços e comentários. O modelo semântico `Schema` é apenas uma projeção derivada do CST.
3. **Caminho Único de Mutação via Patches Mínimos:** Toda ação executada pelo canvas/UI emite um comando semântico processado pelo crate `dbml-edit`, que localiza o nó no CST e gera um **patch textual mínimo** sobre o buffer. Nunca reescreva o arquivo inteiro; preserve rigorosamente a formatação e comentários do usuário.
4. **Isolamento de UI:** O núcleo do projeto (`schema`, `dbml-syntax`, `dbml-analysis`, `dbml-edit`, `layout`, `dbml-wasm`) é composto por crates Rust puras, modulares e testáveis via `cargo test` de forma headless. Apenas o crate `crates/app` depende de GPUI, e o pacote `packages/canvas-web` implementa a renderização Web.
5. **Canvas-First e Zero Modais:** O app abre no canvas (o editor DBML começa recolhido). A criação e edição de tabelas, colunas, tipos e relações é feita inline, sem caixas de diálogo bloqueantes, e com suporte total a undo/redo unificado.

---

## 3. Estrutura do Workspace (Crates e Pacotes)

```text
crates/
  schema/        # Modelo semântico puro: Table, Column, Type, Ref, Index, Enum, ids estáveis
  dbml-syntax/   # Lexer (logos) + parser tolerante a erros → CST rowan (sem dependência de UI)
  dbml-analysis/ # CST → Schema, diagnósticos, resolução de nomes, inferências, renomeações
  dbml-edit/     # Comandos da UI → patches textuais mínimos sobre o CST
  scene/         # Descrição geométrica pura da cena (caixas, textos, arestas, cores) para canvas e export
  layout/        # Algoritmos de auto-layout (Sugiyama/camadas), roteamento de arestas e geometria
  export/        # Rasterização: scene → SVG → PNG/JPEG (resvg/tiny-skia/image); Schema → Mermaid
  project/       # Gerenciamento de arquivos .dbml + sidecar .layout.json, watcher (notify), autosave
  dbml-wasm/     # Bindings WebAssembly (wasm-bindgen), line index UTF-16 e pontes semânticas
  cli/           # Utilitário CLI (dbml run/serve) com servidor Axum, WebSocket e assets embutidos
  app/           # Aplicação desktop GPUI: janelas, canvas customizado, editor, docks, atalhos

packages/
  canvas-web/    # Componente Canvas 2D interativo compartilhado (TypeScript, Viewport2D, Vitest)

vscode-extension/# Extensão oficial do VS Code (CustomTextEditor, Wasm integrado, toolbar)
```

---

## 4. Stack e Bibliotecas (Versões Fixadas)

> **Atenção:** O ecossistema GPUI sofre quebras de API frequentes. Consulte o código-fonte/docs da versão fixada em vez de usar exemplos legados da internet.

| Categoria | Crates / Versões | Propósito |
| :--- | :--- | :--- |
| **UI Desktop** | `gpui-kit` =0.8.0, `gpui-pre` =0.3.9, `gpui-component` =0.8.0 | Janelas, docking, editor de código integrado, componentes base |
| **Parsing & CST** | `logos` 0.16.1, `rowan` 0.17.0, `tree-sitter` 0.26.x | Lexer de alta velocidade, CST sem perdas, highlight sintático |
| **Conformidade** | `dbml-rs` 1.0.1 | Oráculo de validação nos testes |
| **Grafo & Geometria** | `petgraph` 0.8.3, `kurbo` 0.13.1 | Topologia de relações, curvas bezier, hit-testing de arestas |
| **Exportação** | `resvg` 0.48.1, `tiny-skia` 0.12.0, `image` 0.25.10 | Renderização e compressão de imagens PNG/JPEG |
| **Servidor Web & CLI**| `axum` 0.8, `tokio` 1.x, `tokio-tungstenite` 0.29, `rust-embed` 8.x | Servidor local, WebSockets bidirecionais e assets embutidos |
| **WebAssembly** | `wasm-bindgen` 0.2.129, `serde-wasm-bindgen` 0.6 | Compilação do núcleo CST/Rowan para a web e VS Code |
| **Frontend Web** | `typescript` 5.7, `vite` 6.x, `vitest` 5.x, `esbuild` 0.25 | Canvas 2D, viewport afim e suíte de testes unitários |
| **Testes** | `insta` (snapshot testing), `proptest` (testes baseados em propriedade) | Garantia de idêntica recomposição byte a byte |
| **Utilidades** | `serde`, `serde_json`, `notify` 8.x, `nucleo` 0.5 (fuzzy), `thiserror`, `anyhow` | Serialização, watch de arquivos, busca rápida |

---

## 5. Vocabulário do Domínio (Canônico)

Conforme definido em `GLOSSARY.md`, utilize rigorosamente os termos canônicos e evite sinônimos nas discussões, código, testes e documentação:

### Documento
* **Schema** — O modelo semântico derivado do DBML. *(Evitar: modelo, banco, database)*
* **Documento DBML** — O arquivo `.dbml` do usuário. *(Evitar: arquivo de schema, fonte)*
* **Sidecar** — O arquivo `<nome>.dbml.layout.json`. *(Evitar: arquivo de layout, metadados, projeto)*
* **CST** — Árvore sintática concreta sem perdas. *(Evitar: AST, árvore)*
* **Patch** — Edição textual mínima resultante de um comando. *(Evitar: diff, reescrita)*
* **Comando** — Intenção semântica emitida pela UI (ex.: `AddColumn`). *(Evitar: ação, operação)*
* **Diagnóstico** — Erro ou aviso sintático/semântico. *(Evitar: erro de validação, lint)*

### Modelagem
* **Tabela** — Entidade `Table` no DBML. *(Evitar: entidade, model)*
* **Coluna** — Campo com nome, tipo e flags. *(Evitar: atributo, campo)*
* **Ref** — Relação entre colunas com cardinalidade (`1-1`, `1-N`, etc.). *(Evitar: relacionamento, FK, link)*
* **Enum** — Tipo enumerado.
* **Grupo** — Conjunto de tabelas (`TableGroup`). *(Evitar: área, cluster, módulo)*

### Visual
* **Canvas** — Superfície interativa do diagrama. *(Evitar: diagrama, board, quadro)*
* **View** — Subconjunto nomeado de tabelas com layout próprio. *(Evitar: página, aba, perspectiva)*
* **Layout** — Posições e estilos visuais salvos no sidecar.
* **Cena** — Descrição visual geométrica pura (usada no canvas e no export). *(Evitar: render, desenho)*
* **Export** — Geração de PNG, JPEG, SVG ou Mermaid `erDiagram`. *(Evitar: geração, impressão)*

---

## 6. Comandos e Procedimentos de Build

* **Executar app localmente (Linux):** `cargo run -p app`
* **Executar CLI Web localmente:** `cargo run -p dbml-cli -- run schema.dbml`
* **Compilar versão Release:** `cargo build --release -p app`
* **Testes de todo o workspace:** `cargo test --workspace`
* **Testes unitários do Canvas Web:** `cd packages/canvas-web && npm test`
* **Linter e Formatação:** `cargo clippy --workspace` && `cargo fmt --all`
* **Sincronizar todas as plataformas (Wasm + Canvas Web + CLI + VS Code + Desktop):** `./scripts/sync-build.sh` (use `--package` para gerar o `.vsix` e `--release` para binários release)
* **Compilar Wasm:** `cargo build -p dbml-wasm --target wasm32-unknown-unknown --release`
* **Empacotar Extensão VS Code:** `cd vscode-extension && npm run build && npm run package`
* **Build de Pacotes de Distribuição:**
  * Fedora (`.rpm`): `./scripts/build-fedora.sh`
  * Ubuntu/Debian (`.deb`): `./scripts/build-deb.sh`
  * Windows (`.exe` com instalador Inno Setup): `./scripts/build-windows.sh`
  * macOS (`.dmg` e `.pkg` com desinstalador): `./scripts/build-macos.sh`
  * Script universal CLI: `./scripts/install.sh` (com suporte a `--uninstall`)
  * Atualizar versão: `./scripts/bump-version.sh <versao>`
* **Troubleshooting de Linker (`undefined hidden symbol`):**
  Se a compilação falhar com erro de símbolo oculto no `rust-lld` após alterações ou faltas de dependências de sistema, execute:
  ```bash
  cargo clean -p app && cargo build -p app
  ```

---

## 7. Diretrizes para Agentes de IA

1. **Idioma Oficial:** **Tudo em português (pt-BR)** — conversas, commits, PRs, documentações, ADRs e especificações em `.scratch/`. Exceções únicas: identificadores de código, nomes de arquivos/crates e comandos shell.
2. **Registro de Decisões (ADRs):** Ficam exclusivamente em `openspec/adr/` (nunca criar `docs/adr/`). Sempre consulte as ADRs existentes antes de propor alterações estruturais.
3. **Issue Tracker e Specs Locais:** Trabalhos em andamento e divisões de tarefas devem ser documentados em `.scratch/<feature>/spec.md` e `.scratch/<feature>/issues/<NN>-<slug>.md`.
4. **Respeito aos Comentários e Arquivos Existentes:** Nunca limpe comentários explicativos ou docstrings úteis sem solicitação explícita.
5. **Soluções Cirúrgicas:** Prefira a menor alteração completa e funcional. Não adicione dependências não acordadas na stack sem justificativa prévia.
6. **Sincronia Obrigatória entre Desktop, CLI Web e VS Code:**
   * Qualquer nova funcionalidade, comando semântico, atalho ou ajuste de UX/UI deve ser refletido de forma consistente nas 3 modalidades (App Desktop em `crates/app`, Canvas Web compartilhado em `packages/canvas-web`, ponte Wasm em `crates/dbml-wasm`, CLI em `crates/cli` e Extensão em `vscode-extension`).
   * Sempre que alterar o núcleo (`schema`, `dbml-syntax`, `dbml-analysis`, `dbml-edit`, `layout`, `dbml-wasm`) ou o `packages/canvas-web`, execute obrigatoriamente `./scripts/sync-build.sh` para recompilar o Wasm, atualizar o bundle em `crates/cli/web/` e sincronizar `vscode-extension/dist/`.
