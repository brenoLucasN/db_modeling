# 08 — Agentes e roteamento de modelos

Em vez de rodar tudo no Opus, a sessão principal funciona como **orquestradora**: entende o
pedido, decide e delega cada atividade a um subagente com o modelo adequado. Os agentes ficam
em `.claude/agents/` (versionados com o projeto).

## Modelos disponíveis (preço por milhão de tokens, entrada / saída)

| Modelo | Preço | Perfil |
|---|---|---|
| Haiku 5.5 | $0,10 / $0,50 | Rápido e muito barato; busca, tarefas mecânicas |
| Sonnet 5.5 | $2 / $10 | Bom custo-benefício para codar e pesquisar |
| Opus 5.5 | $4 / $20 | Raciocínio forte; design, revisão, bugs difíceis (modelo da sessão principal) |
| Fable 5.1 | $10 / $50 | O mais capaz; só sob pedido explícito |

Fonte: tabela de modelos da skill `claude-api` (preços da API, cache de 2026-10-06). Num plano
de assinatura a proporção de consumo de cota segue a mesma ordem.

## Agentes

| Agente | Modelo / esforço | Ferramentas | Quando usar |
|---|---|---|---|
| `explorador` | Haiku · low | somente leitura | Achar arquivos, símbolos, usos, "como funciona hoje" |
| `mecanico` | Haiku · low | edição | Mudança mecânica já especificada: renomear, fmt/clippy --fix, atualizar refs, traduzir, boilerplate, mensagem de commit |
| `pesquisador` | Sonnet · medium | web + escrita em `docs/pesquisa/` | API do GPUI/gpui-kit, versões de crates, specs DBML/Mermaid; a skill `research` |
| `implementador` | Sonnet · high | edição + Bash + Skill | Implementar ticket/tarefa já especificado, com TDD |
| `arquiteto` | Opus · medium | leitura + escrita só de docs | Design, fronteiras de crates, ADRs, revisar spec/plano |
| `revisor` | Opus · medium | somente leitura + Bash | Revisão de código; revisores paralelos do `code-review` |
| `depurador` | Opus · medium | edição + Bash + Skill | Bug difícil, teste instável, regressão de performance |

## Regras de roteamento (para a sessão principal)

1. **Delegue por padrão** o trabalho que cabe num agente acima; a sessão principal fica com
   conversa com o usuário, decisões, entrevistas (`grill-me`/`grill-with-docs`) e síntese.
2. **Comece barato e escale com motivo**:
   - busca → `explorador`; se a resposta pede julgamento → `arquiteto`;
   - implementação → `implementador` (Sonnet); escale passando `model: opus` **e
     `effort: medium`** na chamada (senão ele herda o `high` do frontmatter) quando a tarefa
     tocar `dbml-edit`/CST, o canvas do GPUI, ou quando o implementador falhar duas vezes ou
     devolver bloqueio;
   - bug que o implementador não resolveu → `depurador`.
3. **Opus sempre em `medium`.** Opus com esforço `high` (ou acima) **nunca** é usado por
   iniciativa própria: só quando a tarefa já foi tentada em Opus `medium` sem sucesso **e** o
   usuário pedir explicitamente o `high` — aí, `effort: high` na chamada daquela tarefa.
   Sonnet e Haiku seguem o esforço do frontmatter.
4. **Fable só sob pedido explícito** do usuário (ou com a aprovação dele para um problema
   específico em que Opus já falhou), via `model: fable` na chamada.
5. **Skills que disparam subagentes** usam estes agentes: `research` → `pesquisador`;
   `code-review` → `revisor` (cada eixo); `implement`/`implement-spec` → `implementador`;
   `improve-codebase-architecture` → `arquiteto`; varreduras de código → `explorador`.
6. **O subagente não vê a conversa**: o prompt precisa trazer o objetivo, os arquivos/tickets
   relevantes, as decisões já tomadas e o formato de resposta esperado.
7. **Paralelize** tarefas independentes (várias buscas, vários tickets sem bloqueio entre si)
   em chamadas simultâneas; tarefas que dependem uma da outra, em sequência.
8. **Não delegue** o que é mais rápido fazer direto (ler um arquivo conhecido, uma edição de
   uma linha) — delegar tem custo de contexto.

## Ajustes

- Trocar o modelo de um agente: editar `model:`/`effort:` no frontmatter do arquivo em
  `.claude/agents/`.
- Forçar um modelo para todos os subagentes (ex.: economizar numa sessão):
  `CLAUDE_CODE_SUBAGENT_MODEL=haiku` com `CLAUDE_CODE_SUBAGENT_MODEL_FORCE=1` no `env` do
  `.claude/settings.local.json`.
- A sessão principal pode rodar num modelo mais barato com `/model sonnet`; nesse caso o
  `arquiteto`/`revisor`/`depurador` continuam em Opus pelo frontmatter.
- Os hooks de caveman e ponytail também se aplicam aos subagentes (`SubagentStart`).
