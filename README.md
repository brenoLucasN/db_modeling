<div align="center">

# DB Modeling

**Modele bancos de dados no canvas ou no texto — os dois são a mesma coisa.**

App desktop nativo para **Windows e Ubuntu**, escrito em **Rust + GPUI**.
Na linha do dbdiagram.io, DrawSQL e Azimutt, com foco total em UI/UX.

![Rust](https://img.shields.io/badge/Rust-1.99-orange?logo=rust)
![GPUI](https://img.shields.io/badge/UI-GPUI-blue)
![Plataformas](https://img.shields.io/badge/Windows%20%7C%20Ubuntu-lightgrey)
![Status](https://img.shields.io/badge/status-Fase%200%2F1-yellow)

<img src="docs/img/canvas-escuro.png" alt="Canvas do DB Modeling no tema escuro, com tabelas agrupadas e relações" width="900">

</div>

## O que é

Ferramentas de modelagem costumam ser **texto primeiro** (o diagrama é só um preview) ou
**UI primeiro** (lentas, cheias de modais, ruins de versionar). O DB Modeling junta as duas:

- **Schema em DBML**, um texto legível que versiona bem em git.
- **Canvas de primeira classe**: tabelas, colunas e relações editáveis direto no diagrama.
- **Edição bidirecional**: mexer num lado atualiza o outro, e suas edições viram patches
  mínimos no texto, preservando comentários e formatação.
- **Rápido, offline e nativo**, com teclado primeiro e command palette (`Ctrl+K`).

Sem conexão com banco e sem SQL: o foco é o visual.

## Em ação

### Texto e canvas lado a lado

O editor DBML abre ao lado do canvas (`Ctrl+E`) e as duas visões ficam sempre sincronizadas.

<img src="docs/img/editor-dbml.png" alt="Editor DBML ao lado do canvas" width="900">

### Tema claro e escuro

<img src="docs/img/canvas-claro.png" alt="Canvas do DB Modeling no tema claro" width="900">

### O que já aparece nas imagens

- Tabelas com PK, FK, tipos e `NOT NULL`, e relações roteadas com cardinalidade (1 / N).
- **Grupos** coloridos (`TableGroup`) e painel lateral com tabelas, relações e diagnósticos.
- Barra de ferramentas (selecionar, mover, nova tabela, auto-layout, ajustar à tela), zoom e painel de propriedades.
- Editor DBML com realce de sintaxe e contagem de erros e avisos na barra de status.

> Em desenvolvimento: export **PNG, JPEG e Mermaid `erDiagram`**, múltiplas vistas, sugestões
> inteligentes e instaladores. Veja o [roadmap](docs/06-roadmap.md).

## Como buildar e usar

Toolchain fixada em `rust-toolchain.toml` (rustc 1.99.1); o `rustup` a instala sozinho.

| Objetivo | Comando |
|---|---|
| Rodar o app (Linux) | `cargo run -p app` |
| Build release | `cargo build --release -p app` |
| Testes do núcleo | `cargo test --workspace` |
| Lint / formatação | `cargo clippy --workspace` · `cargo fmt --all` |
| Pacote Fedora (`.rpm` p/ DNF) | `scripts/build-fedora.sh` (gera pacote `.rpm` em `dist/`) |
| Pacote Ubuntu/Debian (`.deb`) | `scripts/build-deb.sh` (gera pacote `.deb` em `dist/`) |
| `.exe` Windows (cross, do WSL) | `scripts/build-windows.sh` (gera `.exe` em `dist/` e no Desktop) |
| macOS (`.dmg` e bundle `.app`) | `scripts/build-macos.sh` (gera `.dmg` e `.app` em `dist/`) |
| Atualizar versão do projeto | `scripts/bump-version.sh <versao>` (atualiza `Cargo.toml`) |

### Instalação e Atualização de Versões

Todos os instaladores são gerados no diretório `dist/` já integrados com o menu de aplicativos (`.desktop` no Linux, `.app` no macOS) e ícones oficiais em múltiplas resoluções:

- **Passo a passo para lançar uma atualização:**
  1. Atualize a versão unificada: `./scripts/bump-version.sh 0.1.1`
  2. Gere o pacote desejado (`./scripts/build-fedora.sh`, `./scripts/build-deb.sh`, `./scripts/build-windows.sh` ou `./scripts/build-macos.sh`)
- **Fedora (`.rpm` via DNF):**
  - Instalação / Upgrade: `sudo dnf install ./dist/db-modeling-*.rpm` (ou `sudo dnf upgrade ./dist/db-modeling-*.rpm`)
- **Ubuntu / Debian (`.deb` via APT):**
  - Instalação / Upgrade: `sudo apt install ./dist/db-modeling_*_amd64.deb` (ou `sudo dpkg -i ./dist/db-modeling_*_amd64.deb`)
- **Windows (`.exe`):**
  - Executável versionado em `dist/db-modeling-v<versao>-x86_64.exe` (copiado automaticamente para o Desktop do Windows quando gerado a partir do WSL2).
- **macOS (`.dmg` e `.app`):**
  - Imagem de disco montável em `dist/db-modeling-v<versao>-arm64.dmg` (com alias `-aarch64.dmg` para chips ARM / Apple Silicon M1/M2/M3/M4). Ao abrir, basta arrastar o `DB Modeling.app` para a pasta `/Applications`.

- **Ubuntu 22.04/24.04:** o GPUI precisa dos headers de Vulkan, X11/xkbcommon, Wayland e
  fontconfig/freetype (lista exata em [docs/04-stack.md](docs/04-stack.md)). Roda em Wayland e X11.
- **Fedora:** o GPUI precisa dos mesmos headers de desenvolvimento (`sudo dnf install libxkbcommon-devel libxkbcommon-x11-devel libxcb-devel freetype-devel fontconfig-devel vulkan-loader-devel wayland-devel libxkbfile-devel`). Roda em Wayland e X11.
- **Troubleshooting de link (`undefined hidden symbol`):** se a compilação falhar inicialmente por dependência ausente e depois apresentar erros de `rust-lld: error: undefined hidden symbol`, o cache incremental da crate `app` pode ter ficado inconsistente. Rode `cargo clean -p app && cargo build -p app` para limpar apenas os objetos do app (sem recompilar as dependências externas).
- **WSL2:** compila normalmente, mas rodar o GPUI no WSLg ainda não foi validado (risco R1).
- **Perfil dev:** já vem com `opt-level = 1` (dependências em 3); sem isso o GPUI fica muito lento.
- **Instaladores:** pacotes nativos Fedora (`.rpm`), Debian/Ubuntu (`.deb`) e executável Windows (`.exe`) disponíveis via scripts em `scripts/`. Demais formatos (AppImage/MSI) planejados.

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
