# Issue tracker: Markdown local

As issues e specs deste repositório ficam em arquivos Markdown dentro de `.scratch/`.
Todo o conteúdo (títulos, specs, tickets, comentários) é escrito em **português (pt-BR)**.

## Convenções

- Um diretório por feature: `.scratch/<slug-da-feature>/`
- A spec fica em `.scratch/<slug-da-feature>/spec.md`
- Issues de implementação são um arquivo por ticket em `.scratch/<slug-da-feature>/issues/<NN>-<slug>.md`, numerados a partir de `01` — nunca um único arquivo com todos os tickets
- O estado de triagem fica numa linha `Status:` perto do topo de cada issue (ver `triage-labels.md` para os valores)
- Comentários e histórico de conversa são acrescentados ao fim do arquivo, sob o título `## Comentários`

## Quando uma skill disser "publicar no issue tracker"

Criar um arquivo novo em `.scratch/<slug-da-feature>/` (criando o diretório se preciso).

## Quando uma skill disser "buscar o ticket relevante"

Ler o arquivo no caminho indicado. Normalmente o usuário passa o caminho ou o número da issue.

## Operações de wayfinding

Usadas pelo `/wayfinder`. O **mapa** é um arquivo com um arquivo **filho** por ticket.

- **Mapa**: `.scratch/<esforço>/map.md` (corpo com Notas / Decisões até agora / Névoa).
- **Ticket filho**: `.scratch/<esforço>/issues/NN-<slug>.md`, numerado a partir de `01`, com a pergunta no corpo. Uma linha `Tipo:` registra o tipo do ticket (`pesquisa`/`protótipo`/`entrevista`/`tarefa`); uma linha `Status:` registra `reivindicado`/`resolvido`.
- **Bloqueio**: uma linha `Bloqueado por: NN, NN` perto do topo. O ticket está desbloqueado quando todos os arquivos listados estão `resolvido`.
- **Fronteira**: procurar em `.scratch/<esforço>/issues/` arquivos abertos, desbloqueados e não reivindicados; o de menor número vence.
- **Reivindicar**: definir `Status: reivindicado` e salvar antes de qualquer trabalho.
- **Resolver**: acrescentar a resposta sob o título `## Resposta`, definir `Status: resolvido` e acrescentar um ponteiro de contexto (resumo + link) em "Decisões até agora" no `map.md`.
