# 07 — Skills e ferramentas para desenvolvimento assistido

Skills do Claude Code ficam em `.claude/skills/<nome>/SKILL.md`; plugins do projeto ficam
declarados em `.claude/settings.json` (escopo projeto, versionado junto com o repo).

## Instalado (2026-10-24)

| Ferramenta | Como | O que faz aqui |
|---|---|---|
| **OpenSpec** 1.15.0 | `npm i -g @fission-ai/openspec` + `openspec init --tools claude --language pt-BR` | Fluxo *spec-driven*: cada mudança vira `openspec/changes/<nome>/` com proposal, specs (delta), design e tasks. Comandos `/opsx:propose`, `/opsx:explore`, `/opsx:apply`, `/opsx:archive`, `/opsx:sync`, `/opsx:update`. Contexto e regras do projeto em `openspec/config.yaml` |
| **caveman** 4.0.1 | plugin `caveman@caveman` (escopo projeto) | Respostas do chat curtas e diretas (economia de tokens). Não afeta arquivos: código, docs, commits e specs continuam em prosa normal. Desligar: "stop caveman" / "normal mode" |
| **ponytail** 5.3.2 | plugin `ponytail@ponytail` (escopo projeto) | Força a menor solução completa (YAGNI, stdlib/deps já instaladas primeiro). Níveis `/ponytail lite\|full\|ultra\|off`. Nunca corta validação, segurança, acessibilidade |
| **mattpocock-skills** 1.3.1 | plugin `mattpocock-skills@claude-plugins-official` (escopo projeto, atualiza sozinho) | As 27 skills. **Têm prioridade** sobre as demais quando cobrem o mesmo terreno. Setup feito: issue tracker local (`.scratch/`), labels padrão, `GLOSSARY.md` + `openspec/adr/` |
| **defense-in-depth** | `.claude/skills/defense-in-depth/` (adaptada de obra/superpowers) | Validação em todas as camadas (entrada, negócio, ambiente, instrumentação), mapeada para os crates do projeto |

### Fluxo de trabalho resultante (prioridade: mattpocock-skills)

```
ideia ──▶ /grill-with-docs ──▶ GLOSSARY.md + openspec/adr/ atualizados
      ──▶ /to-spec        ──▶ .scratch/<feature>/spec.md
      ──▶ /to-tickets     ──▶ .scratch/<feature>/issues/NN-*.md (com bloqueios)
      ──▶ /implement      ──▶ código (tdd, ponytail, defense-in-depth) ──▶ code-review
```

Sobreposições resolvidas a favor do Matt (tabela completa no `CLAUDE.md`):
`grill-with-docs`/`grill-me` > `/opsx:explore`; `to-spec`/`to-tickets`/`implement` >
OpenSpec; `mattpocock-skills:code-review` > `/code-review` nativo. OpenSpec fica
instalado para uso sob pedido explícito. Para trabalhos grandes
(mais de uma sessão), `/wayfinder` mapeia as decisões em `.scratch/<esforço>/map.md`.
`prototype` é pensado para web; nos spikes Rust/GPUI usar só as regras gerais dele
(descartável, fácil de rodar, sem persistência).

### Observações
- Os hooks de caveman e ponytail rodam com `node`. O Node vem do **fnm** e só está no
  PATH de shells interativos: **reinicie o Claude Code a partir de um terminal novo** para
  os hooks funcionarem (sem `node` eles saem sem fazer nada).
- OpenSpec coleta estatísticas anônimas de uso. Para desligar:
  `openspec config set telemetry.enabled false` (ou `OPENSPEC_TELEMETRY=0`).
- ponytail tem um "task board" local (`~/.config/ponytail/band/<projeto>.csv`) ligado por
  padrão; webhook vem vazio (nada sai da máquina). Ajustar com `/plugin configure ponytail@ponytail`.
- **OpenSpec em pt-BR**: as 6 skills `.claude/skills/openspec-*` foram traduzidas à mão
  (marcadas com `translatedBy` no frontmatter), os comandos `/opsx:*` viraram atalhos em
  português que chamam essas skills, e o schema padrão é `spec-driven-ptbr`
  (`openspec/schemas/spec-driven-ptbr/`, fork do `spec-driven` com instruções e templates
  traduzidos). Só as palavras-chave exigidas pela CLI ficam em inglês (`## ADDED
  Requirements`, `### Requirement:`, `#### Scenario:`, `SHALL`/`MUST`, `WHEN`/`THEN`);
  `openspec validate --strict` aceita o resultado.
- **Cuidado ao atualizar o OpenSpec**: `openspec update` regenera `.claude/skills/openspec-*`
  e `.claude/commands/opsx/*` **em inglês**, sobrescrevendo a tradução. Para atualizar:
  `npm i -g @fission-ai/openspec@latest`, `openspec update`, revisar o diff no git e
  retraduzir o que mudou. O schema `spec-driven-ptbr` não é tocado pelo update; compare-o
  com o `spec-driven` novo (`openspec schema which spec-driven`) para trazer melhorias.
- Do superpowers só a `defense-in-depth` foi copiada (não instalada como plugin), porque o
  plugin completo traz um fluxo próprio (brainstorming, writing-plans, worktrees, TDD
  obrigatório) que competiria com as skills do Matt. O brainstorming foi descartado em favor
  de `grill-me`/`grill-with-docs`. Atualizações do upstream são manuais.

## Skills de UI e GPUI (instaladas em 2026-10-25, escopo projeto)

Copiadas de repositórios públicos (clone + revisão do conteúdo; nada foi executado) para
`.claude/skills/`. Atualizações do upstream são manuais.

| Skill | Origem | Para que serve aqui |
|---|---|---|
| `gpui-kit-design-guides` | `longbridge/gpui-component` | Guia **normativo** de design para apps GPUI Kit (layout, espaçamento, cor, densidade, estados, overlays, movimento). Ler antes de qualquer tela |
| `gpui-kit` | `longbridge/gpui-component` | Referência de API do GPUI Kit/componentes (ações, elementos, estilos, testes) |
| `impeccable` | `pbakaus/impeccable` | Comandos de crítica e polimento de UI (`/audit`, `/polish`, `/typeset`…). Há variantes `*.native.md` nas referências; o foco original é web |
| `design-taste-frontend` (`taste-skill`) | `Leonxlnx/taste-skill` | Direção de arte anti-genérico. **Foco em web**: os scripts `scripts/*.mjs` dirigem um Chrome real (CDP) e só servem para HTML/CSS |

Limite: `impeccable` e `taste-skill` foram feitas para web; no GPUI valem como critério de
revisão visual, não como gerador de código. Para screenshots do app nativo o plano é o
harness com `render_to_image` do GPUI (ver conversa de 2026-10-25) e UI Automation no Windows.

## Avaliação: mattpocock/skills (v1.3.1, commit 5beb35b)

Decisão do usuário: **instalar todas e priorizá-las**. A tabela abaixo é a avaliação
original, mantida como referência de quanto cada skill deve servir a este projeto.

| Skill | Veredito | Motivo |
|---|---|---|
| `tdd` | ✅ Usar | Parser, análise, `dbml-edit` e export são lógica pura com contratos claros; red→green em fatias verticais combina com snapshots `insta` |
| `diagnosing-bugs` | ✅ Usar | Loop de diagnóstico para bugs difíceis e regressões de performance (canvas a 120 fps); complementa defense-in-depth |
| `codebase-design` | ✅ Usar | Vocabulário de módulos profundos/seams — útil para desenhar as fronteiras entre crates |
| `domain-modeling` | ✅ Usar | Mantém `GLOSSARY.md` (Schema, Table, Ref, View, Sidecar, Cena…) e ADRs. Migração para `openspec/adr/` feita |
| `research` | ✅ Usar | Pesquisa em fontes primárias com arquivo citado — essencial com a API instável do GPUI |
| `writing-for-agents` | ✅ Usar | Guia para escrever as skills do projeto (gpui-patterns, dbml-spec…) e o CLAUDE.md |
| `grill-me` / `grilling` | ✅ Usar | Entrevista para afinar ideias e estressar decisões (substitui o brainstorming do superpowers) |
| `handoff`, `wait-what`, `retro` | 🟡 Opcional | Utilidades genéricas e baratas |
| `improve-codebase-architecture` | 🟡 Depois | Só faz sentido quando houver código |
| `pr`, `git-guardrails-claude-code` | 🟡 Depois | Quando houver repositório git/GitHub |
| `prototype` | ❌ Não | Pensado para protótipos web (HTML/rotas); nossos spikes são Rust/GPUI |
| `to-spec`, `to-tickets`, `implement`, `implement-spec`, `wayfinder`, `triage`, `setup-matt-pocock-skills` | ❌ Não | Duplicam o OpenSpec e dependem de issue tracker |
| `code-review` | ❌ Não | Conflita com o `/code-review` nativo do Claude Code; depende de setup |
| `ask-matt`, `teach`, `to-questionnaire`, `wizard` | ❌ Não | Pouco valor para este projeto agora |
| misc (`migrate-to-shoehorn`, `setup-pre-commit`, `scaffold-exercises`) e `in-progress/*` | ❌ Não | Específicas de TypeScript/cursos ou instáveis |

## Do projeto (a criar conforme o código nascer)

| Skill | Gatilho | Conteúdo |
|---|---|---|
| `gpui-patterns` | Ao escrever/alterar código em `crates/app` | Entity/Context/Render, actions e keybindings, FocusHandle, elementos customizados, armadilhas da versão fixada do GPUI |
| `dbml-spec` | Ao mexer em parser/análise/edição | Gramática DBML suportada, mapeamento CST→Schema, como adicionar construção nova + testes de conformidade |
| `ui-command` | Ao criar uma ação nova de UI | Receita: comando semântico → patch em `dbml-edit` → teste de round-trip → action GPUI → atalho → command palette → checklist de UX |
| `export` | Ao mexer em `scene`/`export` | Pipeline cena → SVG → PNG/JPEG, fontes embarcadas, título/legenda/marca d'água, tradução para Mermaid e sanitização, snapshots |
| `ux-review` | Ao concluir um recurso visual | Checklist de `docs/05-ux.md` aplicado ao diff |
| `run-app` | Rodar/screenshot do app | Compilar e abrir no Ubuntu/WSL e no Windows, dependências de sistema, flags de debug, captura de tela |

Também: GPUI Kit skills (`npx skills add longbridge/gpui-kit`) — avaliar na Fase 0, junto
com o spike do GPUI; revisar o conteúdo antes de instalar.

## Outras ferramentas
- Hooks (`.claude/settings.json`): `cargo fmt` após edições e `cargo clippy` antes de
  finalizar — configurar quando houver código.
- `/code-review` e `/simplify` (nativos) em cada mudança.
