# Domain Documentation

How engineering skills should consume domain documentation in this repository.
All glossaries, specs, and ADRs are written in **English (en-US)**; code identifiers are also in English.

## Before exploring, read

- **`GLOSSARY.md`** at the repository root.
- **`openspec/adr/`**: read the ADRs covering the area you are working on.

> **In this repository, ADRs live in `openspec/adr/` alongside specs.** When a skill
> (e.g., `domain-modeling`, `improve-codebase-architecture`) refers to `docs/adr/`, use
> `openspec/adr/`. Never create `docs/adr/`.

If any of these files do not exist, **proceed silently**: do not point out their absence or suggest
creating them upfront. The `/domain-modeling` skill (via `/grill-with-docs` and
`/improve-codebase-architecture`) creates these files on demand when terms or decisions
are actually resolved.

## Structure (single context)

```
/
├── GLOSSARY.md
├── openspec/adr/
│   ├── 0001-gpui-via-gpui-kit.md
│   └── 0002-dbml-estrito-como-formato-canonico.md
└── crates/
```

ADRs: sequential numbering `NNNN-slug.md`, frontmatter `status:` with `proposed`, `accepted`,
`deprecated`, or `superseded by ADR-NNNN`; optional sections `## Considered Options` and
`## Consequences`.

## Use glossary vocabulary

When output names a domain concept (issue title, refactoring proposal, hypothesis, test name),
use the term as defined in `GLOSSARY.md`. Do not drift toward synonyms that the glossary advises avoiding.

If a required concept is not yet in the glossary, it is a signal: either you are inventing
terminology the project does not use (reconsider), or there is a genuine gap (note it for
`/domain-modeling`).

## Flag conflicts with ADRs

If output contradicts an existing ADR, state this explicitly instead of silently overriding it:

> _Contradicts ADR-0004 (text as the single path of change), but worth reopening because…_
