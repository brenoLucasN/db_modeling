# CLAUDE.md

**DB Modeling** — app desktop de modelagem de banco de dados (estilo dbdiagram) em
**Rust + GPUI**, para **Windows e Ubuntu**, com foco máximo em UI/UX e edição
bidirecional texto (DBML) ↔ canvas.

## Escopo
- Dentro: modelagem visual + texto DBML; export **PNG, JPEG e Mermaid `erDiagram`**.
- Fora: conexão/leitura de banco, export/import SQL, migrações, macOS.
- Alvos: Windows 10/11, Ubuntu 22.04/24.04. App abre "canvas first".
- Layout em sidecar JSON (`schema.dbml.layout.json`).

## Status
Fase 0/1 em andamento: workspace com `schema`, `dbml-syntax`, `dbml-analysis`, `dbml-edit`, `layout` e `app` (shell GPUI com canvas e editor ao vivo). Antes de implementar, ler `docs/` (índice no
`README.md`). ADRs em `openspec/adr/` com `status: proposta` aguardam os spikes da Fase 0.

## Fluxo de trabalho
**Prioridade: skills do `mattpocock-skills`.** Quando uma skill dele cobre o mesmo
terreno de outra, use a dele:

| Etapa | Usar | Em vez de |
|---|---|---|
| Afinar ideia/design | `/grill-with-docs` (grilling + domain-modeling) ou `/grill-me` | `/opsx:explore` |
| Spec | `/to-spec` → `.scratch/<feature>/spec.md` | `/opsx:propose` |
| Quebrar em tarefas | `/to-tickets` → `.scratch/<feature>/issues/` | tasks do OpenSpec |
| Implementar | `/implement` (usa `tdd` e fecha com `code-review`) | `/opsx:apply` |
| Revisão | `mattpocock-skills:code-review` | `/code-review` nativo |
| Bugs difíceis | `diagnosing-bugs` (+ `defense-in-depth` na correção) | — |

OpenSpec (`/opsx:*`) continua disponível só quando o usuário pedir
explicitamente. Plugins de estilo ativos: caveman (chat curto; arquivos em prosa normal) e
ponytail (menor solução completa). Detalhes em `docs/07-skills.md`.

## Agentes e modelos
A sessão principal **orquestra e delega por padrão** (o usuário pediu isso explicitamente) a
agentes de `.claude/agents/` com modelo fixo. Detalhes e regras de escalonamento em
`docs/08-agentes.md`.

| Atividade | Agente | Modelo |
|---|---|---|
| Buscar/localizar no código | `explorador` | Haiku |
| Mudança mecânica já especificada | `mecanico` | Haiku |
| Pesquisa em docs/código de crates (`research`) | `pesquisador` | Sonnet |
| Implementar ticket/tarefa (`implement`) | `implementador` | Sonnet (escalar p/ Opus `medium` em `dbml-edit`/CST, canvas, ou após 2 falhas) |
| Design, ADRs, revisar plano | `arquiteto` | Opus · medium |
| Revisão de código (`code-review`) | `revisor` | Opus · medium |
| Bug difícil / performance | `depurador` | Opus · medium |

**Opus nunca em `high`** (ou acima) por iniciativa própria: só se a tarefa falhou em Opus
`medium` **e** o usuário pedir o `high` explicitamente. Ao escalar um agente Sonnet para
Opus, passar `effort: medium` na chamada. Fable só com pedido explícito do usuário. Conversa, entrevistas (`grill-me`) e decisões ficam
na sessão principal. Todo prompt de subagente leva objetivo, arquivos, decisões e formato de
resposta — ele não vê esta conversa.

## Agent skills

### Issue tracker

Markdown local em `.scratch/<feature>/` (ainda sem repositório git remoto). Ver `docs/agents/issue-tracker.md`.

### Triage labels

Os cinco papéis canônicos traduzidos (`precisa-triagem`, `precisa-info`, `pronto-para-agente`, `pronto-para-humano`, `nao-sera-feito`). Ver `docs/agents/triage-labels.md`.

### Domain docs

Contexto único: `GLOSSARY.md` na raiz + ADRs em `openspec/adr/` (nunca `docs/adr/`, mesmo que a skill cite esse caminho). Ver `docs/agents/domain.md`.

## Idioma
**Tudo em português (pt-BR)**: conversa, docs, ADRs, glossário, specs, tickets em
`.scratch/`, design docs, skills do projeto, mensagens de commit e descrições de PR — mesmo
quando a skill (do Matt, OpenSpec ou outra) traz template ou exemplos em inglês: traduzir
títulos e seções do template. Exceções: identificadores de código, nomes de crates/arquivos,
comandos, e as palavras-chave que o parser do OpenSpec exige (`## ADDED Requirements`,
`### Requirement:`, `#### Scenario:`, `SHALL`/`MUST`).

## Princípios que não mudam sem discussão
- DBML do usuário permanece **DBML válido**; metadados do app vão no sidecar de layout.
- Edições da UI viram **patches textuais mínimos** (preservar comentários/formatação).
- Só o crate `app` depende de GPUI; o núcleo é Rust puro e testável.
- Toda ação de UI: acessível por teclado, presente na command palette, sem modal, com undo.

## Stack (fixar versões exatas)
`gpui-kit` 0.8 (→ `gpui-pre` =0.3.9, `gpui-component` 0.8), `logos`, `rowan`,
`petgraph`, `kurbo`, `resvg`, `image`, `insta`, `proptest`. Detalhes em `docs/04-stack.md`.
A API do GPUI muda muito: consultar o código-fonte da versão fixada (docs.rs / repositório)
em vez de confiar em exemplos antigos.

## Ambiente
Desenvolvimento em WSL2 (Linux 6.18), rustc 1.99.1. Execução do GPUI em WSLg não validada
(risco R1). Atalhos usam `Ctrl` (sem macOS).
