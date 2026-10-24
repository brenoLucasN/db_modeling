---
status: proposta
---

# Texto como único caminho de mudança

Edições feitas no canvas viram comandos semânticos que geram patches textuais mínimos no DBML; o `Schema` é sempre derivado do texto. Assim undo/redo, autosave, file watching e diagnósticos têm um único caminho. A alternativa (modelo como fonte e texto regenerado) perderia comentários e formatação do usuário.

## Consequências

`dbml-edit` é complexo e precisa de testes de propriedade fortes (round-trip texto ↔ UI).
