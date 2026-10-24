# DB Modeling

App desktop para modelar bancos de dados visualmente, com o texto DBML e o canvas como
duas visões do mesmo documento.

## Documento

**Schema**:
O modelo semântico de um banco: tabelas, colunas, refs, índices, enums e grupos. É sempre derivado do texto DBML.
_Avoid_: modelo, banco, database

**Documento DBML**:
O arquivo `.dbml` do usuário; fonte da verdade persistida e sempre DBML válido.
_Avoid_: arquivo de schema, fonte

**Sidecar**:
O arquivo `<nome>.dbml.layout.json` ao lado do documento DBML, com tudo que é do app e não do schema (posições, cores, views, opções de export).
_Avoid_: arquivo de layout, metadados, projeto

**CST**:
Árvore sintática sem perdas do documento DBML (inclui espaços e comentários), base para diagnósticos e patches.
_Avoid_: AST, árvore

**Patch**:
Edição textual mínima no documento DBML gerada a partir de um comando.
_Avoid_: diff, reescrita

**Comando**:
Intenção semântica emitida pela UI (ex.: `AddColumn`, `RenameTable`, `CreateRef`) que vira um patch.
_Avoid_: ação (reservado para actions do GPUI), operação

**Diagnóstico**:
Erro ou aviso sobre o documento DBML (sintaxe ou semântica), mostrado no texto e no canvas.
_Avoid_: erro de validação, lint

## Modelagem

**Tabela**:
Entidade do schema com nome, colunas e configurações (`Table` no DBML).
_Avoid_: entidade, model

**Coluna**:
Campo de uma tabela com nome, tipo e flags (pk, not null, unique, increment, default).
_Avoid_: atributo, campo

**Ref**:
Relação entre colunas de duas tabelas, com cardinalidade (`1-1`, `1-N`, `N-1`, `N-N`).
_Avoid_: relacionamento, FK (FK é só o lado da coluna que referencia), link

**Enum**:
Tipo enumerado declarado no schema e usável como tipo de coluna.

**Grupo**:
Conjunto nomeado de tabelas (`TableGroup` no DBML), desenhado como área no canvas.
_Avoid_: área, cluster, módulo

## Visual

**Canvas**:
A superfície do diagrama onde tabelas e refs são desenhadas e editadas.
_Avoid_: diagrama (o resultado), board, quadro

**View**:
Subconjunto nomeado de tabelas com layout próprio, guardado no sidecar.
_Avoid_: página, aba, perspectiva

**Layout**:
Posições e aparência das tabelas no canvas; manual ou gerado por auto-layout.

**Cena**:
Descrição visual pura (caixas, textos, arestas, cores) gerada a partir de schema + layout, usada pelo export.
_Avoid_: render, desenho

**Export**:
Saída do diagrama em PNG, JPEG ou Mermaid `erDiagram`.
_Avoid_: geração, impressão
