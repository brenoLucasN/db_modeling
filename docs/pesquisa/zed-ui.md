# Zed: padrões de UI/UX, janela e GPUI para o DB Modeling

Data: 2026-10-25. Fonte: clone raso de `zed-industries/zed`, commit `18f9d7e7318383a6f8ff8a8838a2746f8c6b376a` (HEAD de 2026-10-25), em `/tmp/zed-src`. Nada foi executado, só lido. Citações no formato `caminho:linha` relativo ao repositório do Zed. Observação: o GPUI do Zed (HEAD) é mais novo que o nosso `gpui-pre =0.3.9`; as APIs de janela citadas abaixo foram conferidas também em `~/.cargo/registry/src/*/gpui-pre-0.3.9`.

## Aviso de licença (ler primeiro)

- GPL-3.0-or-later (campo `license` no `Cargo.toml` de cada crate): `workspace`, `title_bar`, `platform_title_bar`, `ui`, `theme`, `theme_settings`, `settings_content`, `command_palette`, `command_palette_hooks`, `project_panel`, `tab_switcher`, `outline_panel`, `theme_selector`, `picker`, `zed`. Tudo nestes crates é **só inspiração**: não copiar código nem estrutura linha a linha. Números (alturas, durações) e ideias não são código, mas reescrever com as nossas palavras e tipos.
- Apache-2.0: `gpui`, `gpui_windows`, `gpui_linux`, `gpui_wgpu` (arquivos `LICENSE-APACHE`). Nós já usamos esse código via `gpui-pre`; copiar trechos pequenos é permitido com atribuição, mas não há necessidade.
- Temas `assets/themes/{one,ayu,gruvbox}`: cada pasta tem `LICENSE` MIT (copyright de terceiros: GitHub, Ike Ku; o de gruvbox está com placeholder). Hex de cores são fatos, mas se copiarmos paletas inteiras, incluir o aviso MIT. Recomendação: derivar nossa paleta, não copiar.
- Não copie blocos grandes. Este relatório contém só trechos de 1-2 linhas para identificar o ponto.

## Resumo executivo

1. O Zed monta a janela como `Workspace` (centro com `Pane`s) + três `Dock`s (esquerda/direita/baixo) que hospedam `Panel`s; botões de painel ficam na barra de status, não numa barra de atividades (`workspace/src/dock.rs:36`, `:1387`).
2. Hierarquia de superfícies: chrome (título/status) mais claro, painéis intermediários, editor mais escuro; a aba ativa tem a cor do editor, então "funde" com o conteúdo. One Dark: `#3b414d` / `#2f343e` / `#282c33` (`assets/themes/one/one.json`). O nosso `glass.rs` tem chrome mais escuro que a superfície: invertido.
3. Tema = ~143 tokens semânticos (`border`, `border_variant`, `element_*`, `ghost_element_*`, `text*`, status...) + modelo de elevação em 5 níveis com sombras sobrepostas (`theme/src/styles/colors.rs`, `ui/src/styles/elevation.rs`).
4. Transparência: `WindowBackgroundAppearance::{Opaque,Transparent,Blurred,MicaBackdrop,MicaAltBackdrop}` vem do tema (`background.appearance`) e é reaplicado ao trocar tema (`zed/src/main.rs:798`). O 0.3.9 já tem todas as variantes.
5. **Risco para Ubuntu**: no Linux o blur só existe em Wayland+KDE (`org_kde_kwin_blur`, `gpui-pre-linux-0.3.9/src/linux/wayland/window.rs:2323`). GNOME/Ubuntu e X11 ficam com transparência simples ou opaco: o design não pode depender de blur.
6. Com fundo não opaco o GPUI desliga antialiasing subpixel de texto (`gpui/src/window.rs:4917`, também em 0.3.9 `window.rs:4899`): texto fica mais fino; chrome translúcido precisa de contraste extra.
7. Acabamento: título 32px no Windows (34 mín. no Linux), aba 32px, botões de caption 36x32 com glifos Segoe Fluent Icons, handle de resize de 6px com reset por duplo clique, toast 150ms `ease_out_quint`.
8. Bom padrão barato de copiar em ideia: hover/selected "ghost" (transparente → cor de hover), fechar aba só no hover, badge de contagem tintado em vez de vermelho sólido, pré-visualização ao vivo do tema com reversão no cancelar.
9. Não adotar: docks genéricos, splits de pane, multi-workspace, colaboração, serialização em SQLite: complexidade que o app não precisa.
10. Maior impacto/menor custo: inverter hierarquia de superfícies, Mica no Win11, glifos nativos nos botões de janela, handle de resize + duplo clique, elevação padronizada para paleta/popovers.

## a. Arquitetura de janela

**O que o Zed faz**
- `Workspace` guarda `center` (árvore `PaneGroup` de `Member::{Axis, Pane}`, `workspace/src/pane_group.rs:296`) e três `Dock` (`dock.rs:283`). `Workspace::render` empilha: barra de título, área (docks + centro), barra de status (`workspace.rs:9673`).
- `Pane` contém `Item`s (abas). `trait Item` define `tab_content`, `tab_icon`, `tab_tooltip_text` etc. (`item.rs:170`). Abas vivem na barra do pane, **abaixo** da barra de título (a barra de título do Zed não tem abas).
- `trait Panel` (`dock.rs:36`): `persistent_name`, `position`, `set_position`, `default_size`, `min_size`, `icon`, `icon_tooltip`, `toggle_action`, `is_zoomed`/`set_zoomed`, `activation_priority`. Registro: cada painel carrega async e chama `workspace.add_panel` (`zed/src/zed.rs:778`).
- Resize: handle absoluto de `RESIZE_HANDLE_SIZE = 6px` centrado na borda, cursor `col_resize`, arrasto via `on_drag(DraggedDock)`; **duplo clique** chama `reset_panel_sizes` (`dock.rs:25`, `:1271-1330`). Splits de pane usam hitbox de 4px e mínimos 80px (horizontal) / 100px (vertical) (`pane_group.rs:22-24`).
- Persistência: `DockData { visible, active_panel, zoom }` (`persistence/model.rs:204`), tamanhos como JSON `PanelSizeState` sob a chave `dock_panel_size` (`dock.rs:378,408`), gravação com throttle de 200ms (`workspace.rs:177`).
- Zoom de pane: `ToggleZoom` emite `Event::ZoomIn/ZoomOut`; o workspace esconde os docks (`pane.rs:1453`, `workspace.rs:8753`).

**Por que funciona**: painéis são entidades independentes com contrato pequeno; layout persistido por chave estável.

**Recomendação (pequena)**: manter `Pane` como enum no `shell.rs` (já é assim). Adicionar: (1) mínimos de largura nos `resizable_panel` (ex.: 200px sidebar, 260px inspector); (2) duplo clique no divisor reseta para `SIDEBAR_W`/`INSPECTOR_W` (verificar se `gpui_kit::component::resizable` expõe o divisor; senão, overlay próprio de 6px); (3) gravar larguras e painel ativo em `settings.rs` (hoje só tema/destaque), com debounce ~200ms. Arquivos: `shell.rs` (h_resizable ~l.1894), `settings.rs`.

## b. Barra de título e transparência

**O que o Zed faz**
- `PlatformTitleBar` (`platform_title_bar/src/platform_title_bar.rs`): `h_flex` com `.window_control_area(WindowControlArea::Drag)`; arrasto no Linux via `mouse_down` + `mouse_move` → `window.start_window_move()` (l.241); duplo clique → `window.zoom_window()` (Linux) ou `titlebar_double_click()` (Mac); no Windows o comportamento é da plataforma. Menu de contexto da janela no clique direito (Linux CSD).
- Altura: Windows `px(32.)`, demais `max(1.75*rem, 34px)` (`ui/src/utils/constants.rs:17-24`).
- Botões no Windows (`platforms/platform_windows.rs`): fonte "Segoe Fluent Icons" (build >= 22000) ou "Segoe MDL2 Assets"; glifos `\u{e921}` min, `\u{e922}` max, `\u{e923}` restore, `\u{e8bb}` close (l.16, ~60-70); cada botão `w(36px)`, `h_full`, texto 10px, `window_control_area(Min|Max|Close)`, `.occlude()`. Hover: `ghost_element_hover`; fechar: `rgb(232,17,32)` com texto branco, ativo a 80%.
- Linux com decorações do cliente: `LinuxWindowControls` com ícones SVG, layout de botões vindo do sistema (`cx.button_layout()`), cantos arredondados 10px, sombra 10px, borda 1px (`theme/src/theme.rs:53-57`), `set_client_inset` (`workspace.rs:12093`).
- Conteúdo da barra de título: menu da aplicação, nome do projeto (botão "ghost" pequeno com popover de recentes, `title_bar.rs:798`), ramo, etc.
- Fundo da janela: `WindowOptions.window_background = tema.window_background_appearance()` (`zed/src/zed.rs:411`); ao mudar settings reaplica `set_background_appearance` em todas as janelas (`zed/src/main.rs:798`). Tema JSON: chave `"background.appearance": opaque|transparent|blurred|mica_backdrop|mica_alt_backdrop` (`settings_content/src/theme.rs:530,1361`). Nenhum tema embutido usa; o autor do tema coloca alfa nas cores. A documentação de `gpui` diz: no modo opaco o tema deve definir cor totalmente opaca (`gpui/src/platform.rs:2861`).
- Implementação Windows: `Blurred` = `SetWindowCompositionAttribute` estado 4 (acrílico, build >= 17763); Mica = `DwmSetWindowAttribute(DWMWA_SYSTEMBACKDROP_TYPE)` 2/4, build >= 22621 (`gpui_windows/src/window.rs:915-950,1608`).

**Recomendações**
1. `main.rs`: escolher `window_background` por plataforma/versão: Windows 11 (>= 22621) `MicaAltBackdrop` (menos pesado que acrílico, segue o papel de parede); Windows 10 `Blurred`; Linux `Blurred` só se KDE Wayland, senão `Opaque` (ou `Transparent` com alfa alto). Se não conseguir detectar versão, manter `Blurred` e garantir que `chrome` seja legível mesmo sem blur.
2. `glass.rs`: tratar alfa do `chrome`/`sidebar` como parâmetro dependente do suporte a blur (ex.: `0xb8` com blur, `0xf2` sem). Hoje `0xb8/0xc7` sem blur no Ubuntu fica "sujo" sobre o desktop.
3. `shell.rs titlebar()` (l.683-930): trocar os glifos desenhados com `div` pelos glifos da fonte Segoe (fallback atual se a fonte não existir); altura 32px e botões 36px (ou 46px, padrão nativo: decisão visual nossa); hover do fechar `#E81120`/branco; demais usam `g.hover`. Guardar `.occlude()` em cada botão.
4. Decisão a confirmar: abas dentro da barra de título (nosso caso, estilo navegador) são válidas, mas o Zed separa; se a barra de 40px ficar pesada, mover abas para uma tab bar de 32px abaixo e deixar a barra de título em 32px com menu + paleta + controles.

## c. Sistema de tema, espaçamento e componentes base

**Tokens** (`theme/src/styles/colors.rs`): `border`, `border_variant`, `border_focused`, `border_selected`, `border_transparent`, `border_disabled`; superfícies `background`, `surface_background`, `elevated_surface_background`, `editor_background`; estados `element_*` e `ghost_element_*` (background/hover/active/selected/disabled); texto `text`, `text_muted`, `text_placeholder`, `text_disabled`, `text_accent`; ícones; `status_bar_background`, `title_bar_background` (+`inactive`), `tab_bar_background`, `tab_active/inactive_background`, `panel_background`, `panel_indent_guide*`; scrollbar; syntax; `StatusColors` (error/warning/success/info/created/modified/deleted... cada um com `x`, `x_background`, `x_border`, `status.rs`); `PlayerColors`. Diferença `element` (tem fundo próprio) vs `ghost_element` (transparente, só destaca no hover) é a ideia mais útil.

**Valores One** (`assets/themes/one/one.json`): Dark: border `#464b57`, border.variant `#363c46`, background/title/status `#3b414d`, surface/panel/tab_bar `#2f343e`, editor/tab ativa `#282c33`, hover `#363c46`, selected `#454a56`, texto `#dce0e5`, muted `#a9afbc`. Light: border `#c9c9ca`, chrome `#dcdcdd`, panel `#ebebec`, editor `#fafafa`, hover `#dfdfe0`, texto `#242529`. Temas embutidos: One (Dark/Light), Ayu (Dark/Light/Mirage), Gruvbox (6 variantes).

**Troca de tema**: `settings.theme = { mode: system|light|dark, light: "One Light", dark: "One Dark" }` (`assets/settings/default.json:9`); seguimos `SystemAppearance`. O seletor (`theme_selector`) aplica o tema **ao vivo** ao navegar e reverte no cancelar (`override_global` com as configurações originais, `theme_selector.rs:250-265`, `dismissed` l.418).

**Espaçamento e tipografia** (`ui/src/styles/spacing.rs:29`): `DynamicSpacing::BaseNN` com 3 densidades (compact/default/comfortable); tuplas explícitas p/ pequenos, fórmula (n-4, n, n+4) p/ grandes. `TextSize`: Large 16, Default 14, Small 12, XSmall 10 (rem base 16; `ui/src/styles/typography.rs`); `ui_font_size` padrão 16, fixado entre 6 e 100px. Ícones: 10/12/14/16px. Botões: 16/18/22/28/32px (`ButtonSize`, `button_like.rs:455`). Elevação (`ui/src/styles/elevation.rs`): Background, Surface, EditorSurface, ElevatedSurface (sombra 0/2/3px a 12% + 0/1/0 a 6%), ModalSurface (4 camadas, até 12px de blur). Base `elevated()`: `bg elevated_surface`, `rounded_lg`, borda 1px `border_variant`, sombra (`ui/src/traits/styled_ext.rs:6`).

**Componentes**: `Tab` 32px com 1px de borda (conteúdo 31px), slots de 12px (início) e 14px (fim), `px 4`, `gap 4` (`tab.rs:79-80`); `ListItem` com `ghost_element_hover/active/selected`, `rounded_sm`, indent via `indent_step_size`, espaçamento Dense/ExtraDense/Sparse (`list_item.rs:300-440`); `Disclosure` é um `IconButton` (chevron), visível no hover quando aberto; `Tooltip` com título + keybinding à direita + meta muted, `max_w_72`, deslocado do cursor (`pl_2 pt_2.5`), elevação 2 (`tooltip.rs:216`); `CountBadge` 14px com fundo `editor_background` misturado a 40% de erro, não vermelho sólido (`count_badge.rs:27`).

**Recomendações**
1. `glass.rs`: reorganizar os tokens para a hierarquia do Zed: `chrome` > `sidebar` > `surface`, aba ativa = `surface`. Hoje (escuro) chrome `#141518` < surface `#1f2023`; testar chrome `~#2a2c31`, sidebar `~#232529`, surface `#1b1c1f` (ou equivalente claro). Manter alfa só em chrome/sidebar.
2. Separar `border` (forte) e `border_variant` (divisórias internas) em `Glass.edge`/`edge_soft`; separar `hover` (ghost) de `selected`.
3. Elevação: criar `Glass::popover_style(...)`/função em `glass.rs` com fundo `island` opaco-ish, raio `RADIUS_ISLAND`, borda `edge_soft` 1px e 2 sombras (0,2,3px/12% e 0,1,0/6%); usar em paleta, menus, tooltips.
4. Badge da barra de atividades (`shell.rs` rail, ~l.995): trocar `bg(g.danger)` sólido por mistura tintada (surface + danger 40%) com borda 1px.
5. Escala: adicionar `Density` (compact/default/comfortable) em `glass.rs` multiplicando `SP_*`, `TITLEBAR_H`, altura de aba. Depois; não é urgente.
6. Seletor de tema/destaque com pré-visualização ao vivo e reversão no Esc (hoje `ToggleTheme` e swatches): `shell.rs` + `settings.rs`.

## d. Barra de status, project panel, paleta, tab switcher, toasts, vazios

- **Barra de status** (`workspace/src/status_bar.rs:115`): `h_flex`, `p 4`, `gap 8`, `justify_between`, itens à esquerda e à direita, fundo `status_bar_background`, `role=Toolbar`, setas esq/dir movem foco entre itens, clique direito oferece "Ocultar botão". Botões de painel (IconButton pequeno, `toggle_state`, tooltip com atalho, clique direito = menu "Dock Left/Right/Bottom") ficam aqui (`dock.rs:1387-1580`). Recomendação: manter a barra de atividades à esquerda (padrão VS Code, mais descobrível para um app de modelagem), mas adotar teclado na barra de status (setas) e clique direito → "Ocultar" para itens secundários; `shell.rs statusbar()` l.1807.
- **Project panel** (`project_panel/src/project_panel.rs`): `ListItem` com `indent_level` e `indent_step_size` padrão 20px, espaçamento "comfortable" (Dense) ou "standard" (ExtraDense), largura padrão 240, clique simples abre em pré-visualização, duplo clique fixa, clique do meio abre, Alt+clique expande tudo, guias de indentação, sticky de pastas, indicadores de diagnóstico/git à direita (`default.json:880`). Recomendação para `shell.rs` (painel Arquivos): indent 12-16px, guias de indentação 1px `edge_soft`, contagem de erros por arquivo à direita, aba em "pré-visualização" (título em itálico) no clique simples.
- **Paleta de comandos** (`command_palette/src/command_palette.rs:764`, `picker/src/render.rs`): modal sem moldura própria; `Picker` com campo `h_9` (36px) `px 2.5`, `Divider` fina, lista com `ListItem` inset, correspondência destacada (`HighlightedLabel`), atalho (`KeyBinding`) à direita, rodapé com borda superior `border_variant`, estado vazio como `ListItem` desabilitado muted, contêiner `elevation_3`. `ModalLayer` (`modal_layer.rs:288`): `top_20`, centralizado, clique fora fecha, fundo opcional `elevated_surface` com `fade_out(0.2)`. Recomendação: paleta do `shell.rs` (~l.472, `CommandState`): campo de 36px + divisor, destaque de caracteres casados, atalho alinhado à direita em `text_faint`, deslocamento de topo ~80px.
- **Tab switcher** (`tab_switcher/src/tab_switcher.rs`): `ModalView` com `Picker`; Ctrl+Tab abre e soltar o Ctrl confirma; pré-visualização de arquivo opcional. Depois: útil com várias abas, barato com a nossa lista de `docs`.
- **Toasts** (`toast_layer.rs`): um toast ativo, canto inferior central, 10s, pausa o timer ao passar o mouse (retoma com mínimo de 800ms), botão do meio dispensa, entrada `animate_in(FromBottom, fade)`: 150ms, `ease_out_quint`, opacidade 0,4 → 1 e deslocamento de 40px (`ui/src/styles/animation.rs:41-70`); durações padrão 50/150/300ms. Recomendação: hoje usamos `window.push_notification` (gpui-component); manter, mas verificar duração/pausa no hover e aplicar tom de status (barra lateral colorida usando `ok/warning/danger`).
- **Estados vazios** (`ui/src/components/project_empty_state.rs`): coluna centralizada, `w_48`, texto muted pequeno, botão principal largura total com atalho, divisor "ou", botão secundário. Recomendação: canvas sem tabelas e Arquivos sem projeto no mesmo molde (uma ação primária, uma secundária, atalho visível).

## e. Acabamento: medidas, raios, hover/foco, animação

| Item | Zed | Nosso (hoje) |
|---|---|---|
| Barra de título | 32px (Win) / >= 34px | 40px |
| Aba | 32px (31 + 1 de borda) | 32px |
| Botão de janela | 36px x altura da barra, texto 10px | 44px, glifo desenhado |
| Handle de resize | 6px, duplo clique reseta | divisor padrão do kit |
| Campo do picker | 36px | verificar |
| Raios | `rounded_sm` itens, `rounded_lg` popovers, 10px janela CSD | 8/10px |
| Texto UI | 14 (padrão), 12, 10 | 13,5 / 12,5 / 11,5 |
| Barra de status | ~ 22-24px (p 4 + conteúdo) | 24px |
| Animação | 50/150/300ms, `ease_out_quint` | n/d |

Padrões de interação: fechar aba só no hover (`visible_on_hover`, `pane.rs:3033`), clique do meio fecha aba, alvo de arrasto destaca com `drop_target_background` e borda 2px no lado de entrada (`pane.rs:2973`); foco em lista = borda 1px `border_focused`; `active` mais escuro que `hover`; tooltips sempre com atalho; ícones e rótulos com `aria_label`/`role` (já fazemos).

Recomendações curtas: (1) fechar da aba ativa visível, das outras só no hover (já fazemos); adicionar clique do meio fecha em `shell.rs titlebar()`; (2) estado `active` (pressionado) distinto de `hover` nas abas, rail e botões; (3) transição de entrada de popovers/toasts 150ms `ease_out_quint` via `gpui::Animation` (existe no 0.3.9, conferir `AnimationExt`); (4) reordenar abas por arrasto com marcador de destino.

## Tabela final

| Item | Decisão | Esforço | Arquivo nosso |
|---|---|---|---|
| Inverter hierarquia de superfícies (chrome > sidebar > surface; aba ativa = surface) | Adotar agora | 2-3 h | `glass.rs` |
| `window_background` por plataforma: Mica Alt (Win11), Blurred (Win10/KDE Wayland), Opaque/alto alfa no GNOME/X11 | Adotar agora | 2-4 h (detecção de versão) | `main.rs`, `glass.rs` |
| Botões de janela com glifos Segoe, 32px, close `#E81120` | Adotar agora | 1-2 h | `shell.rs` |
| Duplo clique no divisor reseta; mínimos; persistir larguras | Adotar agora | 3-5 h | `shell.rs`, `settings.rs` |
| Estilo de elevação único (popover/paleta/tooltip) com 2 sombras e `edge_soft` | Adotar agora | 2 h | `glass.rs`, `shell.rs` |
| Paleta: campo 36px, divisor, destaque de match, atalho à direita | Adotar agora | 2-3 h | `shell.rs` |
| Badge tintado, ghost vs selected, estado `active` | Adotar agora | 1-2 h | `glass.rs`, `shell.rs` |
| Estado vazio padrão (primária + secundária + atalho) | Adotar agora | 1-2 h | `shell.rs`, `canvas.rs` |
| Pré-visualização ao vivo de tema/destaque | Depois | 3-4 h | `shell.rs` |
| Densidade (compact/default/comfortable) | Depois | 1 dia | `glass.rs` |
| Tab switcher (Ctrl+Tab) e arrastar abas | Depois | 1 dia | `shell.rs`, `actions.rs` |
| Aba em pré-visualização (itálico) no clique simples | Depois | 3 h | `shell.rs` |
| Docks genéricos (esq/dir/baixo), zoom de pane, splits | Não adotar | n/a | n/a |
| Serialização em SQLite, multi-workspace, colaboração, agente | Não adotar | n/a | n/a |
| Abas na tab bar abaixo do título | Decidir (não é necessário) | 3-4 h | `shell.rs` |
| Tema JSON editável pelo usuário | Não adotar agora (YAGNI) | n/a | n/a |

## Não confirmado

- Se `gpui_kit::component::resizable` expõe o divisor para duplo clique e mínimos.
- Comportamento real do blur no WSLg/Ubuntu (não testado; só leitura de código). Ubuntu GNOME: ausência de blur deduzida do código Wayland (apenas protocolo KDE) e da falta de implementação X11 no 0.3.9.
- Altura nativa dos botões de caption no Windows 11 (46px) é conhecimento geral, não vem do código do Zed.
- Largura do rodapé da barra de status do Zed em px não está fixada no código (derivada de padding + conteúdo).

## Fontes

Repositório: https://github.com/zed-industries/zed (commit `18f9d7e7318383a6f8ff8a8838a2746f8c6b376a`). Arquivos lidos: `crates/platform_title_bar/src/{platform_title_bar.rs,platforms/platform_windows.rs,platforms/platform_linux.rs}`, `crates/title_bar/src/title_bar.rs`, `crates/workspace/src/{workspace.rs,dock.rs,pane.rs,pane_group.rs,status_bar.rs,toast_layer.rs,modal_layer.rs,item.rs,persistence/model.rs}`, `crates/ui/src/{styles/*.rs,traits/styled_ext.rs,utils/constants.rs,components/{tab,tab_bar,list/list_item,disclosure,tooltip,count_badge,project_empty_state}.rs}`, `crates/theme/src/{theme.rs,styles/colors.rs,styles/status.rs,ui_density.rs}`, `crates/theme_settings/src/theme_settings.rs`, `crates/settings_content/src/theme.rs`, `crates/theme_selector/src/theme_selector.rs`, `crates/command_palette/src/command_palette.rs`, `crates/picker/src/render.rs`, `crates/tab_switcher/src/tab_switcher.rs`, `crates/project_panel/src/project_panel.rs`, `crates/gpui/src/{platform.rs,window.rs}`, `crates/gpui_windows/src/window.rs`, `assets/themes/{one,ayu,gruvbox}`, `assets/settings/default.json`. Código pinado: `gpui-pre-0.3.9` (`src/platform.rs:2766`, `src/window.rs:4899`), `gpui-pre-windows-0.3.9/src/window.rs:932`, `gpui-pre-linux-0.3.9/src/linux/wayland/window.rs:2323`. Docs web (zed.dev/docs/themes, /extensions/themes) não mencionam `background.appearance`; sem apoio de documentação, só código.

## Status de adoção (2026-10-25)
Aplicados os 8 itens "Adotar agora": hierarquia de superfícies, fundo por plataforma
(`crates/app/src/platform.rs`), botões de janela com glifos Segoe, larguras de painel
persistidas + "Restaurar larguras dos painéis" (sem duplo clique: o divisor do
`gpui-component` não expõe o evento), elevação (`glass::elevation`), paleta a 80 px do topo,
badge tintado / estado pressionado / borda suave, estados vazios padrão (`widgets.rs`).
Extra: ações disparadas de paleta/menus/diálogos agora chegam ao shell via handlers globais
(`shell::register_global_actions`).
