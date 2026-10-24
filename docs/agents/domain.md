# Documentação de domínio

Como as skills de engenharia devem consumir a documentação de domínio deste repositório.
Todo glossário e ADR é escrito em **português (pt-BR)**; identificadores de código ficam em inglês.

## Antes de explorar, ler

- **`GLOSSARY.md`** na raiz do repositório.
- **`openspec/adr/`**: ler as ADRs que tocam a área em que vai trabalhar.

> **Neste repositório as ADRs ficam em `openspec/adr/`, junto com as specs.** Quando uma skill
> (ex.: `domain-modeling`, `improve-codebase-architecture`) citar `docs/adr/`, use
> `openspec/adr/`. Nunca crie `docs/adr/`.

Se algum desses arquivos não existir, **seguir em silêncio**: não apontar a ausência nem sugerir
criá-los de antemão. A skill `/domain-modeling` (via `/grill-with-docs` e
`/improve-codebase-architecture`) cria esses arquivos sob demanda, quando termos ou decisões
forem de fato resolvidos.

## Estrutura (contexto único)

```
/
├── GLOSSARY.md
├── openspec/adr/
│   ├── 0001-gpui-via-gpui-kit.md
│   └── 0002-dbml-estrito-como-formato-canonico.md
└── crates/
```

ADRs: numeração sequencial `NNNN-slug.md`, frontmatter `status:` com `proposta`, `aceita`,
`obsoleta` ou `substituída pela ADR-NNNN`; seções opcionais `## Opções consideradas` e
`## Consequências`.

## Usar o vocabulário do glossário

Quando a saída nomear um conceito do domínio (título de issue, proposta de refatoração,
hipótese, nome de teste), usar o termo como definido no `GLOSSARY.md`. Não derivar para
sinônimos que o glossário manda evitar.

Se o conceito necessário ainda não está no glossário, é um sinal: ou está inventando
linguagem que o projeto não usa (reconsiderar), ou há uma lacuna real (anotar para o
`/domain-modeling`).

## Apontar conflitos com ADRs

Se a saída contradiz uma ADR existente, dizer isso explicitamente em vez de sobrescrever em
silêncio:

> _Contradiz a ADR-0004 (texto como único caminho de mudança), mas vale reabrir porque…_
