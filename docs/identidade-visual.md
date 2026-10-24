# Identidade visual — DB Modeling

## Conceito
**Duas tabelas ligadas por uma relação.** O gesto central do produto (modelar tabelas e
conectá-las coluna a coluna) é a própria marca: dois cartões com cabeçalho, ligados por uma
relação ortogonal com portas — o mesmo desenho que o canvas usa. Sóbrio, técnico, legível em
16 px, sem metáforas genéricas de "banco de dados" (cilindros).

## Arquivos (`crates/app/assets/brand/`)
| Arquivo | Uso |
|---|---|
| `icon.svg` | Fonte do ícone do app (512×512, quadrado arredondado em vidro azul-noite) |
| `icon-{16,24,32,48,64,128,256,1024}.png` | Rasterizações (geradas) |
| `icon.ico` | Ícone do `.exe` no Windows (embutido via `crates/app/build.rs` + `app.rc`) |
| `mark.svg` | Marca simplificada monocromática (`currentColor`), para 16–32 px e usos sobre cor |

Regenerar PNG/ICO depois de editar o SVG:
`cargo run --manifest-path tools/render-icons/Cargo.toml`

## Construção do ícone
- Base: quadrado 480 com raio 112 (proporção de ícone de app moderno), degradê diagonal
  `#2a4fd6 → #14287a → #0a1230`, brilho superior branco 28% → 0 e aresta interna branca 22%
  (o "vidro").
- Cartões: vidro claro (branco 24% → 8%) com borda branca 45%; cabeçalhos `#5b9bff`
  (tabela de origem) e `#a5c4ff` (tabela de destino); linhas de coluna em branco 75%/45%.
- Relação: traço `#8fb6ff` de 14 px com canto arredondado; portas brancas de raio 12.
- Área de respiro: nada a menos de 72 px da borda do quadrado.

## Cores
| Papel | Escuro | Claro |
|---|---|---|
| Destaque (lima, fixo) | `#c6f240` | `#5f7d00` |
| Superfície (canvas) | `#141518` | `#f7f7f8` |
| Cartão de tabela | `#1f2024` | `#ffffff` |
| Texto | `#e6e7ea` | `#1f2328` |
| Texto secundário | `#a3a6ad` | `#57606a` |
| Erro / aviso / ok | `#f2555a` / `#e5b454` / `#4cc27a` | `#cf222e` / `#9a6700` / `#1a7f37` |

Um só visual (sem destaques ou estilos de diagrama alternativos; valores em `glass.rs`):
cartões com raio 12 e sombra suave, seleção com anel/halo do destaque.
O ícone da marca é sempre azul-noite.

## Tipografia
**Inter** (OFL, embarcada): Regular para texto, Medium para rótulos, SemiBold para títulos e
o nome do produto. Escala: 11,5 / 12,5 / 13,5 / 15 px. Nome do produto sempre "DB Modeling"
(D e B maiúsculos, sem hífen).

## Não fazer
- Esticar, girar ou recolorir o ícone completo (use `mark.svg` para versões monocromáticas).
- Colocar o ícone sobre fundos azuis parecidos sem a aresta clara.
- Usar cilindros de "banco de dados" ou ícones genéricos no lugar da marca.
