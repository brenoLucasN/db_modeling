<div align="center">

# DB Modeling

**Modele bancos de dados no canvas ou no texto — os dois são a mesma coisa.**

Ecossistema nativo de modelagem visual de bancos de dados escrito em **Rust + GPUI / WebAssembly / Axum**.
Na linha do dbdiagram.io, DrawSQL e Azimutt, com foco total em UI/UX e sincronização bidirecional.

![Rust](https://img.shields.io/badge/Rust-1.99-orange?logo=rust)
![GPUI](https://img.shields.io/badge/UI-GPUI-blue)
![WebAssembly](https://img.shields.io/badge/WebAssembly-Wasm-purple)
![Plataformas](https://img.shields.io/badge/Windows%20%7C%20Linux%20%7C%20macOS-lightgrey)
![VS Code](https://img.shields.io/badge/VS%20Code-Extension-007ACC?logo=visualstudiocode)

<img src="docs/img/canvas-escuro.png" alt="Canvas do DB Modeling no tema escuro, com tabelas agrupadas e relações" width="900">

</div>

## O que é

Ferramentas de modelagem costumam ser **texto primeiro** (o diagrama é só um preview estático) ou
**UI primeiro** (lentas, cheias de modais, ruins de versionar). O DB Modeling une os dois mundos:

- **Schema em DBML**, formato canônico legível e ideal para controle de versão em Git.
- **Canvas de primeira classe**: tabelas, colunas e relações editáveis direto no diagrama com zero modais.
- **Edição bidirecional**: mexer num lado atualiza o outro, e suas edições viram **patches mínimos** no texto, preservando rigorosamente comentários e formatação original.
- **Rápido, offline e nativo**, com foco em teclado, atalhos fluidos e command palette (`Ctrl+K`).
- **Exportação completa**: PNG, JPEG, SVG vetorial e sintaxe Mermaid `erDiagram`.

---

## 3 Maneiras de Usar

### 1. App Desktop Nativo (GPUI)
Para quem busca a experiência máxima de performance: janelas nativas aceleradas por hardware (DirectX no Windows, Vulkan no Linux, Metal no macOS) renderizando a 120 fps.

### 2. Extensão Oficial do VS Code (`.vsix`)
Edite e visualize seus diagramas sem sair do editor.
- Abre arquivos `.dbml` no editor visual customizado com clique duplo ou via botão **"Abrir no Canvas Visual"** (`Ctrl+Alt+D` / menu de navegação).
- Núcleo Rowan/CST compilado para **WebAssembly**, calculando patches mínimos com coordenadas de linha/coluna UTF-16 em tempo real.
- Barra de ferramentas integrada para **Auto-Layout** (Sugiyama), **Formatação canônica**, **Exportação para SVG** e **Mermaid**.

### 3. Utilitário CLI & Web Server (`dbml run`)
Para quem edita em Vim, Neovim, Helix, Sublime ou qualquer outro editor externo:
```bash
# Inicia o servidor local Axum e abre o Canvas no navegador padrão
dbml run schema.dbml
# Ou especifique porta customizada:
dbml run schema.dbml --port 4000
```
- Servidor HTTP/WebSocket ultraleve em Axum com assets embutidos em tempo de compilação (`rust-embed`).
- Monitoramento contínuo de arquivos via `notify` com hashing anti-eco: alterou no seu editor, o navegador atualiza na hora; alterou no navegador, o arquivo é salvo atomicamente no disco.

---

## Em Ação

### Texto e canvas lado a lado

O editor DBML abre ao lado do canvas (`Ctrl+E`) e as duas visões ficam sempre sincronizadas.

<img src="docs/img/editor-dbml.png" alt="Editor DBML ao lado do canvas" width="900">

### Tema claro e escuro

<img src="docs/img/canvas-claro.png" alt="Canvas do DB Modeling no tema claro" width="900">

---

## Como Buildar e Testar

Toolchain fixada em `rust-toolchain.toml` (rustc 1.99.1); o `rustup` a instala sozinho.

| Objetivo | Comando |
|---|---|
| **Rodar App Desktop (Linux)** | `cargo run -p app` |
| **Rodar CLI Web Server** | `cargo run -p dbml-cli -- run schema.dbml` |
| **Testes do Workspace Rust** | `cargo test --workspace` |
| **Testes Unitários do Canvas Web** | `cd packages/canvas-web && npm test` |
| **Compilar Wasm** | `cargo build -p dbml-wasm --target wasm32-unknown-unknown --release` |
| **Empacotar Extensão VS Code** | `cd vscode-extension && npm run build && npm run package` |
| **Lint & Formatação** | `cargo clippy --workspace` · `cargo fmt --all` |

---

## Instaladores e Pacotes de Distribuição

Todos os instaladores são gerados no diretório `dist/` com suporte a instalação, atualização in-loco e desinstalação completa:

- **Instalador Universal CLI (Linux/macOS):**
  - Instalação com resolução automática de dependências: `./scripts/install.sh`
  - Desinstalação limpa: `./scripts/install.sh --uninstall`
- **Fedora (`.rpm` via DNF):**
  - Build: `./scripts/build-fedora.sh`
  - Instalação: `sudo dnf install ./dist/db-modeling-*.rpm`
- **Ubuntu / Debian (`.deb` via APT):**
  - Build: `./scripts/build-deb.sh`
  - Instalação: `sudo apt install ./dist/db-modeling_*_amd64.deb`
- **Windows (`.exe` com Inno Setup Wizard):**
  - Build: `./scripts/build-windows.sh`
  - Script Inno Setup: `scripts/windows/installer.iss` (com atalhos no Menu Iniciar, Desktop e associação de arquivos `.dbml`).
- **macOS (`.dmg` e `.pkg`):**
  - Build: `./scripts/build-macos.sh` (gera `.dmg` e instalador nativo `.pkg` com desinstalador).

---

## Documentação

| Documento | Conteúdo |
|---|---|
| [docs/01-visao.md](docs/01-visao.md) | Problema, público, diferenciais, princípios de UX |
| [docs/02-linguagem.md](docs/02-linguagem.md) | DBML vs linguagem própria vs outras; recomendação |
| [docs/03-arquitetura.md](docs/03-arquitetura.md) | Workspace de crates, modelo de dados, sincronização texto↔UI |
| [docs/04-stack.md](docs/04-stack.md) | Bibliotecas propostas, versões e justificativas |
| [docs/05-ux.md](docs/05-ux.md) | Fluxos de interação: canvas, edição só por UI, atalhos |
| [docs/06-roadmap.md](docs/06-roadmap.md) | Fases, MVP e spikes técnicos |
| [docs/07-skills.md](docs/07-skills.md) | Skills/ferramentas para desenvolvimento assistido (Claude Code) |
| [docs/08-agentes.md](docs/08-agentes.md) | Agentes e roteamento de modelos por tipo de atividade |
| [openspec/adr/](openspec/adr/) | Registro de decisões (ADRs), um arquivo por decisão |
| [GLOSSARY.md](GLOSSARY.md) | Vocabulário do domínio |
| [docs/riscos.md](docs/riscos.md) | Riscos técnicos e perguntas em aberto |
