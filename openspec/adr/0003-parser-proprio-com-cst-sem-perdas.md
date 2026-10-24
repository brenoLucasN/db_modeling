---
status: proposta
---

# Parser próprio com CST sem perdas

Usamos `logos` + parser recursivo-descendente com recuperação de erros + CST `rowan`, em vez de `dbml-rs`, porque o editor precisa de tolerância a erros, spans precisos e edição sem perder comentários/formatação. `dbml-rs` fica só como oráculo de conformidade nos testes.
