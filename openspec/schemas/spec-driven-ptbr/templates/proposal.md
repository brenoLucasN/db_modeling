# Proposta

## Por quê

<!-- Explique a motivação desta change. Que problema ela resolve? Por que agora? -->

## O que muda

<!-- Descreva o que vai mudar. Seja específico sobre capabilities novas, modificações ou remoções. -->

## Capabilities

### Capabilities novas
<!-- Capabilities sendo introduzidas. Nomeie cada uma por um comportamento coeso do
     sistema que possa reunir requirements relacionados conforme o sistema evolui. Não
     use nomes de tarefas de implementação nem de seções da proposta. Evite nomes
     genéricos demais. Use kebab-case nos segmentos de caminho que você introduzir
     (ex.: export-imagem ou export/mermaid), seguindo a organização de specs do
     projeto. Cada uma cria specs/<capability-path>/spec.md. -->
- `<capability-path>`: <descrição breve do que esta capability cobre>

### Capabilities modificadas
<!-- Capabilities existentes cujos REQUIREMENTS mudam (não só a implementação).
     Liste aqui apenas se o comportamento no nível da spec mudar. Cada uma precisa de
     um arquivo de spec delta. Use o caminho exato existente em openspec/specs/. Deixe
     vazio se nenhum requirement mudar. Uma change sem nenhuma capability (refatoração
     pura, ferramentas, docs) precisa definir `skip_specs: true` no .openspec.yaml —
     o openspec validate rejeita uma change sem deltas sem esse marcador. Não invente
     um requirement só para passar na validação. -->
- `<existing-capability-path>`: <qual requirement está mudando>

## Fora de escopo

<!-- O que esta change explicitamente NÃO faz. -->

## Impacto

<!-- Crates, módulos, dependências e ADRs (openspec/adr/NNNN) e riscos (R1..R10) afetados -->
