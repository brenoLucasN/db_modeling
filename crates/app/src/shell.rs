//! Janela principal. Um *projeto* é uma pasta com um ou mais arquivos `.dbml`
//! (ou um arquivo avulso); cada arquivo aberto vira uma aba com editor e canvas.
//! Layout: barra de título com abas · barra de atividades · painel lateral
//! (Arquivos / Esquema / Problemas) · editor · canvas · inspector · barra de status.
//! Todas as ações passam por `actions::catalog`.

use crate::actions::{self, *};
use crate::canvas::{Diagram, DiagramEvent, Tool};
use crate::glass::*;
use dbml_analysis::Severity;
use gpui_kit::assets::IconName;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::command::{Command, CommandGroup, CommandItem, CommandState};
use gpui_kit::component::input::{Editor, EditorState, Input, InputEvent, InputState};
use gpui_kit::component::kbd::Kbd;
use gpui_kit::component::menu::DropdownMenu;
use gpui_kit::component::resizable::{h_resizable, resizable_panel};
use gpui_kit::component::theme::{Theme, ThemeMode};
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::component::{Icon, InteractiveElementExt as _, Sizable, WindowExt};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use std::path::{Path, PathBuf};

/// Um arquivo aberto (aba).
pub struct Doc {
    path: Option<PathBuf>,
    title: String,
    editor: Entity<EditorState>,
    diagram: Entity<Diagram>,
    dirty: bool,
    diags: Vec<dbml_analysis::Diagnostic>,
}

impl Doc {
    fn counts(&self) -> (usize, usize) {
        let n = |s| self.diags.iter().filter(|d| d.severity == s).count();
        (n(Severity::Error), n(Severity::Warning))
    }
}

/// Campo em edição no inspector (um por vez, como no Figma).
#[derive(Clone, PartialEq, Eq, Debug)]
enum EditTarget {
    Table(String),
    ColName(String, String),
    ColType(String, String),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Pane {
    Files,
    Schema,
    Problems,
}

const STANDARD_TYPES: &[(&str, &str)] = &[
    ("int", "inteiro 4B"),
    ("bigint", "inteiro longo 8B"),
    ("smallint", "inteiro curto 2B"),
    ("serial", "autoincremento"),
    ("varchar(255)", "texto variável"),
    ("text", "texto longo"),
    ("boolean", "booleano"),
    ("uuid", "identificador único"),
    ("timestamp", "data e hora"),
    ("timestamptz", "data e hora com fuso"),
    ("date", "data"),
    ("time", "hora"),
    ("decimal(10,2)", "numérico decimal"),
    ("float", "ponto flutuante"),
    ("double", "ponto flutuante duplo"),
    ("json", "JSON"),
    ("jsonb", "JSON binário"),
    ("bytea", "binário"),
];

fn extract_enums_from_dbml(text: &str) -> Vec<String> {
    let mut enums = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("Enum ") {
            let name = rest
                .split(|c: char| c.is_whitespace() || c == '{')
                .next()
                .unwrap_or("")
                .trim();
            if !name.is_empty() && !enums.contains(&name.to_string()) {
                enums.push(name.to_string());
            }
        }
    }
    enums
}

pub struct Shell {
    /// Pasta do projeto; `None` = arquivos avulsos.
    project: Option<PathBuf>,
    /// `.dbml` encontrados na pasta do projeto.
    files: Vec<PathBuf>,
    docs: Vec<Doc>,
    active: usize,
    untitled: usize,
    pane: Pane,
    sidebar_open: bool,
    editor_open: bool,
    inspector_open: bool,
    mode: Mode,
    /// chrome translúcido (há blur no fundo da janela)
    translucent: bool,
    /// larguras dos painéis (salvas); `panels_gen` recria o grupo ao restaurar
    widths: (f32, f32, f32),
    panels_gen: usize,
    translucent_cards: bool,
    /// largura da coluna de tipos de dados no inspector
    col_type_w: f32,
    /// arraste horizontal para redimensionar a coluna de tipos (x inicial, col_type_w inicial)
    col_resize_start: Option<(Pixels, f32)>,
    editing: Option<(EditTarget, Entity<InputState>, Subscription)>,
    last_summary: Option<crate::canvas::Summary>,
    palette: Entity<CommandState>,
    focus: FocusHandle,
    _subs: Vec<Subscription>,
    dragging_window: bool,
    ui_scale: f32,
    inspector_auto_hide: bool,
}

impl Focusable for Shell {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

/// Aplica claro/escuro ao `gpui-component` e a identidade do app (fonte, acento, raios).
#[allow(dead_code)]
pub fn apply_theme(mode: Mode, cx: &mut App) {
    apply_theme_scaled(mode, 1.0, cx);
}

/// Aplica tema considerando a escala de UI configurada.
pub fn apply_theme_scaled(mode: Mode, scale: f32, cx: &mut App) {
    Theme::change(
        match mode {
            Mode::Dark => ThemeMode::Dark,
            Mode::Light => ThemeMode::Light,
        },
        None,
        cx,
    );
    let g = Glass::of_scaled(mode, false, scale);
    Theme::update(cx, |t| {
        t.font_family = FONT.into();
        t.font_size = px((13.5 * scale).round());
        t.radius = px((6. * scale).round());
        t.radius_lg = px((10. * scale).round());
        // popovers, menus e tooltips: camada elevada com sombra (como no Zed)
        t.shadow = true;
        // acento em todo lugar que o componente usa "primário" (o botão tem campos próprios)
        let (hover, active) = (g.accent.opacity(0.88), g.accent.opacity(0.76));
        t.colors.primary = g.accent;
        t.colors.primary_hover = hover;
        t.colors.primary_active = active;
        t.colors.primary_foreground = gpui_kit::white();
        t.colors.button_primary = g.accent;
        t.colors.button_primary_hover = hover;
        t.colors.button_primary_active = active;
        t.colors.button_primary_foreground = gpui_kit::white();
        t.colors.ring = g.accent;
        t.colors.popover = g.island.opacity(1.);
        t.colors.popover_foreground = g.text;
    });
}

/// Lista os `.dbml` de uma pasta (até 3 níveis, sem pastas ocultas), em ordem.
fn scan(dir: &Path) -> Vec<PathBuf> {
    fn walk(dir: &Path, depth: usize, out: &mut Vec<PathBuf>) {
        let Ok(rd) = std::fs::read_dir(dir) else {
            return;
        };
        let mut entries: Vec<_> = rd.flatten().map(|e| e.path()).collect();
        entries.sort();
        for p in entries {
            let hidden = p
                .file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with('.'));
            if p.is_dir() && depth < 3 && !hidden {
                walk(&p, depth + 1, out);
            } else if p.extension().is_some_and(|e| e == "dbml") {
                out.push(p);
            }
        }
    }
    let mut out = Vec::new();
    walk(dir, 0, &mut out);
    out
}

fn file_title(p: &Path) -> String {
    p.file_name()
        .map_or("?".into(), |n| n.to_string_lossy().into_owned())
}

impl Shell {
    /// `start`: arquivo, pasta ou nada (abre o exemplo).
    pub fn new(
        start: Option<PathBuf>,
        mode: Mode,
        translucent: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let mut this = Self {
            project: None,
            files: Vec::new(),
            docs: Vec::new(),
            active: 0,
            untitled: 0,
            pane: Pane::Schema,
            sidebar_open: std::env::var("DBM_SIDEBAR").is_ok()
                || crate::settings::load().sidebar_open,
            editor_open: std::env::var("DBM_EDITOR").is_ok(),
            inspector_open: std::env::var("DBM_INSPECTOR").is_ok()
                || crate::settings::load().inspector_open,
            mode,
            translucent,
            widths: {
                let p = crate::settings::load();
                (p.sidebar_w, p.editor_w, p.inspector_w)
            },
            // DBM_TRANSLUCENT: usado pelo harness de screenshot
            translucent_cards: std::env::var("DBM_TRANSLUCENT").is_ok()
                || crate::settings::load().translucent_cards,
            col_type_w: crate::settings::load().col_type_w,
            col_resize_start: None,
            panels_gen: 0,
            last_summary: None,
            editing: None,
            palette: cx.new(|cx| CommandState::new(window, cx)),
            focus: cx.focus_handle(),
            _subs: Vec::new(),
            dragging_window: false,
            ui_scale: crate::settings::load().ui_scale,
            inspector_auto_hide: crate::settings::load().inspector_auto_hide,
        };
        // DBM_PANE: painel inicial (usado pelo harness de screenshot)
        this.pane = match std::env::var("DBM_PANE").as_deref() {
            Ok("files") => Pane::Files,
            Ok("problems") => Pane::Problems,
            _ => Pane::Schema,
        };
        let preselect = std::env::var("DBM_SELECT").ok();
        match start {
            Some(p) if p.is_dir() => this.set_project(p, window, cx),
            Some(p) => this.open_path(p, window, cx),
            None => {
                let text = include_str!("../assets/sample.dbml").to_string();
                this.add_doc(text, None, "exemplo.dbml".into(), window, cx);
            }
        }
        // DBM_SELECT: tabela pré-selecionada (harness de screenshot)
        if let Some(t) = preselect {
            this.with_diagram(cx, |d| d.select(&t));
        }
        this
    }

    fn doc(&self) -> Option<&Doc> {
        self.docs.get(self.active)
    }

    fn add_doc(
        &mut self,
        text: String,
        path: Option<PathBuf>,
        title: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let diagram = cx.new(|cx| Diagram::new(dbml_analysis::analyze(&text), cx));
        let editor = cx.new(|cx| {
            let mut st = EditorState::new(window, cx)
                .default_value(text)
                .line_number(true);
            crate::ide::install(&mut st, cx);
            st
        });
        let diags = editor.update(cx, crate::ide::refresh_diagnostics);
        // texto → Schema → canvas, a cada edição
        let sub = cx.subscribe_in(
            &editor,
            window,
            |this: &mut Shell, ed, ev: &InputEvent, _, cx| {
                if !matches!(ev, InputEvent::Change) {
                    return;
                }
                let schema = dbml_analysis::analyze(&ed.read(cx).value());
                let diags = ed.update(cx, crate::ide::refresh_diagnostics);
                if let Some(doc) = this.docs.iter_mut().find(|d| &d.editor == ed) {
                    doc.dirty = true;
                    doc.diags = diags;
                    doc.diagram.update(cx, |d, cx| {
                        d.set_schema(schema);
                        cx.notify();
                    });
                }
                cx.notify();
            },
        );
        // repinta o shell só quando muda algo que ele mostra (seleção, zoom, dica...);
        // mover/arrastar no canvas redesenha apenas o canvas
        let obs = cx.observe(&diagram, |this: &mut Shell, d, cx| {
            let sum = d.read(cx).summary();
            if this.last_summary.as_ref() != Some(&sum) {
                let had_selection = this
                    .last_summary
                    .as_ref()
                    .is_some_and(|s| s.0.is_some() || !s.1.is_empty());
                let now_has_selection = sum.0.is_some() || !sum.1.is_empty();
                if now_has_selection && !had_selection {
                    this.inspector_open = true;
                    this.save_prefs();
                }
                this.last_summary = Some(sum);
                cx.notify();
            }
        });
        let events = cx.subscribe_in(
            &diagram,
            window,
            |this: &mut Shell, _, ev: &DiagramEvent, window, cx| match ev {
                DiagramEvent::CreateRef { from, to } => {
                    let (f, t) = (from.clone(), to.clone());
                    this.edit_text(false, window, cx, |text| {
                        Some(dbml_edit::add_ref(text, (&f.0, &f.1), (&t.0, &t.1)))
                    });
                }
                DiagramEvent::AddColumn(table) => this.add_column_to(table.clone(), window, cx),
                DiagramEvent::Rename(table) => {
                    this.inspector_open = true;
                    this.start_edit(EditTarget::Table(table.clone()), table.clone(), window, cx);
                }
                DiagramEvent::Select(table) => {
                    let _ = table;
                    this.inspector_open = true;
                    this.save_prefs();
                    cx.notify();
                }
            },
        );
        self._subs.extend([sub, obs, events]);
        self.docs.push(Doc {
            path,
            title,
            editor,
            diagram,
            dirty: false,
            diags,
        });
        self.active = self.docs.len() - 1;
        cx.notify();
    }

    fn open_path(&mut self, path: PathBuf, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(i) = self
            .docs
            .iter()
            .position(|d| d.path.as_ref() == Some(&path))
        {
            self.active = i;
            cx.notify();
            return;
        }
        match std::fs::read_to_string(&path) {
            Ok(text) => {
                let title = file_title(&path);
                self.add_doc(text, Some(path), title, window, cx);
            }
            Err(e) => window.push_notification(format!("Não foi possível abrir: {e}"), cx),
        }
    }

    fn set_project(&mut self, dir: PathBuf, window: &mut Window, cx: &mut Context<Self>) {
        self.files = scan(&dir);
        self.project = Some(dir);
        if std::env::var("DBM_PANE").is_err() {
            self.pane = Pane::Files;
        }
        if let Some(first) = self.files.first().cloned() {
            self.open_path(first, window, cx);
        }
        cx.notify();
    }

    fn with_diagram(&mut self, cx: &mut Context<Self>, f: impl FnOnce(&mut Diagram)) {
        if let Some(doc) = self.docs.get(self.active) {
            doc.diagram.update(cx, |d, cx| {
                f(d);
                cx.notify();
            });
        }
    }

    // ---- ações: arquivos e abas ----

    fn new_file(&mut self, _: &NewFile, window: &mut Window, cx: &mut Context<Self>) {
        self.untitled += 1;
        let title = format!("sem-titulo-{}.dbml", self.untitled);
        self.add_doc(String::new(), None, title, window, cx);
        self.editor_open = true;
        self.sidebar_open = false;
        self.save_prefs();
    }

    fn open_file(&mut self, _: &OpenFile, window: &mut Window, cx: &mut Context<Self>) {
        let rx = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: true,
            prompt: Some("Abrir".into()),
        });
        cx.spawn_in(window, async move |this, cx| {
            let Ok(Ok(Some(paths))) = rx.await else {
                return;
            };
            this.update_in(cx, |this, window, cx| {
                for p in paths {
                    this.open_path(p, window, cx);
                }
            })
            .ok();
        })
        .detach();
    }

    fn open_folder(&mut self, _: &OpenFolder, window: &mut Window, cx: &mut Context<Self>) {
        let rx = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some("Abrir projeto".into()),
        });
        cx.spawn_in(window, async move |this, cx| {
            let Ok(Ok(Some(paths))) = rx.await else {
                return;
            };
            let Some(dir) = paths.into_iter().next() else {
                return;
            };
            this.update_in(cx, |this, window, cx| this.set_project(dir, window, cx))
                .ok();
        })
        .detach();
    }

    fn save_file(&mut self, _: &SaveFile, window: &mut Window, cx: &mut Context<Self>) {
        let Some(doc) = self.doc() else { return };
        if let Some(path) = doc.path.clone() {
            self.write(path, window, cx);
            return;
        }
        let dir = self
            .project
            .clone()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
        let rx = cx.prompt_for_new_path(&dir, Some(&doc.title));
        cx.spawn_in(window, async move |this, cx| {
            let Ok(Ok(Some(path))) = rx.await else { return };
            this.update_in(cx, |this, window, cx| this.write(path, window, cx))
                .ok();
        })
        .detach();
    }

    fn write(&mut self, path: PathBuf, window: &mut Window, cx: &mut Context<Self>) {
        let Some(doc) = self.docs.get_mut(self.active) else {
            return;
        };
        let text = doc.editor.read(cx).value().to_string();
        match std::fs::write(&path, text) {
            Ok(()) => {
                window.push_notification(format!("Salvo em {}", path.display()), cx);
                doc.title = file_title(&path);
                doc.path = Some(path);
                doc.dirty = false;
                if let Some(dir) = self.project.clone() {
                    self.files = scan(&dir);
                }
            }
            Err(e) => window.push_notification(format!("Não foi possível salvar: {e}"), cx),
        }
        cx.notify();
    }

    fn close_tab_at(&mut self, i: usize, cx: &mut Context<Self>) {
        if i >= self.docs.len() {
            return;
        }
        // shortcut: fecha sem perguntar mesmo com alterações; trocar por confirmação inline quando houver undo de arquivo
        self.docs.remove(i);
        if self.active >= self.docs.len() {
            self.active = self.docs.len().saturating_sub(1);
        } else if i < self.active {
            self.active -= 1;
        }
        cx.notify();
    }

    fn close_tab(&mut self, _: &CloseTab, _: &mut Window, cx: &mut Context<Self>) {
        self.close_tab_at(self.active, cx);
    }

    fn next_tab(&mut self, _: &NextTab, _: &mut Window, cx: &mut Context<Self>) {
        if !self.docs.is_empty() {
            self.active = (self.active + 1) % self.docs.len();
            cx.notify();
        }
    }

    fn prev_tab(&mut self, _: &PrevTab, _: &mut Window, cx: &mut Context<Self>) {
        if !self.docs.is_empty() {
            self.active = (self.active + self.docs.len() - 1) % self.docs.len();
            cx.notify();
        }
    }

    fn toggle_left_pane(&mut self, target: Option<Pane>, cx: &mut Context<Self>) {
        match target {
            Some(pane) => {
                // clicar no painel já aberto recolhe a barra lateral; caso contrário abre exclusivamente
                if self.sidebar_open && !self.editor_open && self.pane == pane {
                    self.sidebar_open = false;
                } else {
                    self.pane = pane;
                    self.sidebar_open = true;
                    self.editor_open = false;
                }
            }
            None => {
                // alternar editor DBML de forma exclusiva com os outros painéis esquerdos
                if self.editor_open {
                    self.editor_open = false;
                } else {
                    self.editor_open = true;
                    self.sidebar_open = false;
                }
            }
        }
        self.save_prefs();
        cx.notify();
    }

    fn show_pane(&mut self, pane: Pane, cx: &mut Context<Self>) {
        self.toggle_left_pane(Some(pane), cx);
    }

    fn show_files(&mut self, _: &ShowFiles, _: &mut Window, cx: &mut Context<Self>) {
        self.show_pane(Pane::Files, cx);
    }

    fn show_schema(&mut self, _: &ShowSchema, _: &mut Window, cx: &mut Context<Self>) {
        self.show_pane(Pane::Schema, cx);
    }

    fn show_problems(&mut self, _: &ShowProblems, _: &mut Window, cx: &mut Context<Self>) {
        self.show_pane(Pane::Problems, cx);
    }

    // ---- ações: editor ----

    fn format_document(&mut self, _: &FormatDocument, window: &mut Window, cx: &mut Context<Self>) {
        let Some(doc) = self.doc() else { return };
        let text = doc.editor.read(cx).value().to_string();
        let formatted = dbml_edit::format(&text);
        if formatted != text {
            // replace_all entra no histórico: Ctrl+Z desfaz a formatação
            doc.editor
                .update(cx, |e, cx| e.replace_all(formatted, window, cx));
        }
    }

    fn move_line(&mut self, up: bool, window: &mut Window, cx: &mut Context<Self>) {
        let Some(doc) = self.doc() else { return };
        doc.editor.update(cx, |e, cx| {
            let text = e.value().to_string();
            let pos = e.cursor_position();
            let Some((range, new)) = dbml_edit::swap_lines(&text, pos.line as usize, up) else {
                return;
            };
            e.set_selected_range(range, cx);
            e.replace(new, window, cx);
            let line = if up { pos.line - 1 } else { pos.line + 1 };
            e.set_cursor_position(
                lsp_types::Position {
                    line,
                    character: pos.character,
                },
                window,
                cx,
            );
        });
    }

    fn move_line_up(&mut self, _: &MoveLineUp, window: &mut Window, cx: &mut Context<Self>) {
        self.move_line(true, window, cx);
    }

    fn move_line_down(&mut self, _: &MoveLineDown, window: &mut Window, cx: &mut Context<Self>) {
        self.move_line(false, window, cx);
    }

    /// Abre o editor e põe o cursor no início do diagnóstico.
    fn reveal(&mut self, offset: usize, window: &mut Window, cx: &mut Context<Self>) {
        self.editor_open = true;
        self.sidebar_open = false;
        self.save_prefs();
        if let Some(doc) = self.doc() {
            doc.editor.update(cx, |e, cx| {
                let text = e.value().to_string();
                let before = &text[..offset.min(text.len())];
                let line = before.matches('\n').count() as u32;
                let character = before.rsplit('\n').next().unwrap_or("").chars().count() as u32;
                e.set_cursor_position(lsp_types::Position { line, character }, window, cx);
            });
        }
        cx.notify();
    }

    // ---- ações: janela ----

    fn quit(&mut self, _: &Quit, _: &mut Window, cx: &mut Context<Self>) {
        cx.quit();
    }

    fn open_palette(&mut self, _: &OpenPalette, window: &mut Window, cx: &mut Context<Self>) {
        let state = self.palette.clone();
        window.open_dialog(cx, move |dialog, _, _| {
            let state = state.clone();
            dialog
                .w(px(600.))
                .margin_top(px(80.))
                .content(move |content, _, _| {
                    let mut groups: Vec<(&str, Vec<CommandItem>)> = Vec::new();
                    for e in actions::catalog() {
                        if e.action.partial_eq(&OpenPalette) {
                            continue;
                        }
                        let item = CommandItem::new().label(e.label).action(e.action);
                        match groups.iter_mut().find(|(g, _)| *g == e.group) {
                            Some((_, v)) => v.push(item),
                            None => groups.push((e.group, vec![item])),
                        }
                    }
                    let mut cmd = Command::new(&state)
                        .placeholder("Buscar comandos…")
                        .on_confirm(|_, window, cx| window.close_dialog(cx));
                    for (g, items) in groups {
                        cmd = cmd.group(CommandGroup::new().label(g).items(items));
                    }
                    content.child(cmd)
                })
        });
    }

    fn show_shortcuts(&mut self, _: &ShowShortcuts, window: &mut Window, cx: &mut Context<Self>) {
        window.open_dialog(cx, |dialog, _, _| {
            dialog
                .w(px(520.))
                .title("Atalhos de teclado")
                .content(|content, window, _| {
                    let mut list = div().flex().flex_col().gap(px(6.));
                    for e in actions::catalog() {
                        list = list.child(
                            div()
                                .flex()
                                .justify_between()
                                .items_center()
                                .text_size(TEXT_SM)
                                .child(e.label)
                                .children(Kbd::binding_for_action(
                                    e.action.as_ref(),
                                    Some(e.context),
                                    window,
                                )),
                        );
                    }
                    content.child(list)
                })
        });
    }

    fn toggle_theme(&mut self, _: &ToggleTheme, _: &mut Window, cx: &mut Context<Self>) {
        self.mode = self.mode.toggled();
        self.apply_appearance(cx);
    }

    /// Aplica uma transformação no texto do arquivo ativo (com undo) e atualiza o canvas.
    fn edit_text(
        &mut self,
        relayout: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
        f: impl FnOnce(&str) -> Option<String>,
    ) {
        let Some(doc) = self.doc() else { return };
        let text = doc.editor.read(cx).value().to_string();
        let Some(new) = f(&text).filter(|n| *n != text) else {
            return;
        };
        let schema = dbml_analysis::analyze(&new);
        doc.editor
            .update(cx, |e, cx| e.replace_all(new, window, cx));
        doc.diagram.update(cx, |d, cx| {
            d.set_schema(schema);
            if relayout {
                d.relayout();
            }
            cx.notify();
        });
        cx.notify();
    }

    fn start_edit(
        &mut self,
        target: EditTarget,
        initial: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let state = cx.new(|cx| InputState::new(window, cx).default_value(initial));
        let handle = state.read(cx).focus_handle(cx);
        window.focus(&handle, cx);
        let sub = cx.subscribe_in(
            &state,
            window,
            |this: &mut Shell, _, ev: &InputEvent, window, cx| match ev {
                InputEvent::Change => {
                    cx.notify();
                }
                InputEvent::PressEnter { .. } | InputEvent::Blur => {
                    this.commit_edit(window, cx);
                }
                _ => {}
            },
        );
        self.editing = Some((target, state, sub));
        cx.notify();
    }

    /// Aplica um tipo selecionado via autocomplete imediatamente.
    fn apply_column_type(
        &mut self,
        table: String,
        col: String,
        ty: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.editing = None;
        if !ty.is_empty() {
            self.edit_text(false, window, cx, |t| {
                dbml_edit::set_column_type(t, &table, &col, &ty)
            });
        }
        cx.notify();
    }

    /// Aplica o campo em edição no DBML. Nomes inválidos são recusados com aviso.
    fn commit_edit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some((target, state, _)) = self.editing.take() else {
            return;
        };
        let value = state.read(cx).value().trim().to_string();
        let ident = !value.is_empty() && value.chars().all(|c| c.is_alphanumeric() || c == '_');
        match target {
            EditTarget::Table(old) if value != old => {
                if !ident {
                    window
                        .push_notification("Nome de tabela inválido: use letras, números e _.", cx);
                } else {
                    let new = value.clone();
                    self.edit_text(false, window, cx, |t| {
                        Some(dbml_edit::rename_table(t, &old, &new))
                    });
                    self.with_diagram(cx, |d| d.rename(&old, &value));
                }
            }
            EditTarget::ColName(table, old) if value != old => {
                if !ident {
                    window
                        .push_notification("Nome de coluna inválido: use letras, números e _.", cx);
                } else {
                    self.edit_text(false, window, cx, |t| {
                        dbml_edit::rename_column(t, &table, &old, &value)
                    });
                }
            }
            EditTarget::ColType(table, col) if !value.is_empty() => {
                self.edit_text(false, window, cx, |t| {
                    dbml_edit::set_column_type(t, &table, &col, &value)
                });
            }
            _ => {}
        }
        cx.notify();
    }

    fn add_column_to(&mut self, table: String, window: &mut Window, cx: &mut Context<Self>) {
        let Some(doc) = self.doc() else { return };
        let text = doc.editor.read(cx).value().to_string();
        let name = dbml_edit::unique_column_name(&text, &table, "coluna");
        self.edit_text(false, window, cx, |t| {
            dbml_edit::add_column_text(t, &table, &name, "int")
        });
        self.inspector_open = true;
        self.start_edit(EditTarget::ColName(table, name.clone()), name, window, cx);
    }

    fn new_table(&mut self, _: &NewTable, window: &mut Window, cx: &mut Context<Self>) {
        let Some(doc) = self.doc() else { return };
        let text = doc.editor.read(cx).value().to_string();
        let name = dbml_edit::unique_table_name(&text, "tabela");
        self.edit_text(false, window, cx, |t| Some(dbml_edit::add_table(t, &name)));
        self.with_diagram(cx, |d| d.place_new(&name));
        self.inspector_open = true;
        self.start_edit(EditTarget::Table(name.clone()), name, window, cx);
    }

    fn delete_selection(
        &mut self,
        _: &DeleteSelection,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(doc) = self.doc() else { return };
        let names = doc.diagram.read(cx).selection();
        if names.is_empty() {
            return;
        }
        self.edit_text(false, window, cx, |t| {
            Some(
                names
                    .iter()
                    .fold(t.to_string(), |acc, n| dbml_edit::delete_table(&acc, n)),
            )
        });
        self.with_diagram(cx, |d| d.deselect());
    }

    fn rename_selected(&mut self, _: &RenameSelected, window: &mut Window, cx: &mut Context<Self>) {
        let Some(doc) = self.doc() else { return };
        if let Some(t) = doc.diagram.read(cx).selected().map(|t| t.name.clone()) {
            self.inspector_open = true;
            self.start_edit(EditTarget::Table(t.clone()), t, window, cx);
        }
    }

    fn toggle_flag(
        &mut self,
        table: String,
        col: String,
        flag: &'static str,
        on: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.edit_text(false, window, cx, |t| {
            dbml_edit::set_column_flag(t, &table, &col, flag, on)
        });
    }

    fn group_selection(&mut self, _: &GroupSelection, window: &mut Window, cx: &mut Context<Self>) {
        let Some(doc) = self.doc() else { return };
        let names = doc.diagram.read(cx).selection();
        if names.is_empty() {
            window.push_notification("Selecione tabelas (Shift+clique) para agrupar.", cx);
            return;
        }
        self.edit_text(true, window, cx, |t| {
            let refs: Vec<&str> = names.iter().map(String::as_str).collect();
            Some(dbml_edit::group_tables(
                t,
                &refs,
                &dbml_edit::next_group_name(t),
            ))
        });
    }

    fn ungroup_selection(
        &mut self,
        _: &UngroupSelection,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(doc) = self.doc() else { return };
        let names = doc.diagram.read(cx).selection();
        self.edit_text(true, window, cx, |t| {
            Some(
                names
                    .iter()
                    .fold(t.to_string(), |acc, n| dbml_edit::ungroup_table(&acc, n)),
            )
        });
    }

    fn set_color(
        &mut self,
        table: String,
        color: Option<&'static str>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.edit_text(false, window, cx, |t| {
            dbml_edit::set_table_color(t, &table, color)
        });
    }

    fn save_prefs(&self) {
        crate::settings::save(&crate::settings::Settings {
            mode: Some(self.mode),
            sidebar_w: self.widths.0,
            editor_w: self.widths.1,
            inspector_w: self.widths.2,
            col_type_w: self.col_type_w,
            translucent_cards: self.translucent_cards,
            ui_scale: self.ui_scale,
            inspector_auto_hide: self.inspector_auto_hide,
            sidebar_open: self.sidebar_open,
            inspector_open: self.inspector_open,
        });
    }

    fn apply_appearance(&mut self, cx: &mut Context<Self>) {
        apply_theme_scaled(self.mode, self.ui_scale, cx);
        self.save_prefs();
        cx.notify();
    }

    fn set_ui_scale(&mut self, scale: f32, window: &mut Window, cx: &mut Context<Self>) {
        self.ui_scale = (scale * 100.0).round() / 100.0;
        self.apply_appearance(cx);
        window.push_notification(
            format!(
                "Escala da interface: {}%",
                (self.ui_scale * 100.0).round() as i32
            ),
            cx,
        );
    }

    fn zoom_in_ui(&mut self, _: &ZoomInUi, window: &mut Window, cx: &mut Context<Self>) {
        let next = (self.ui_scale + 0.1).min(1.8);
        self.set_ui_scale(next, window, cx);
    }

    fn zoom_out_ui(&mut self, _: &ZoomOutUi, window: &mut Window, cx: &mut Context<Self>) {
        let next = (self.ui_scale - 0.1).max(0.75);
        self.set_ui_scale(next, window, cx);
    }

    fn reset_ui_zoom(&mut self, _: &ResetUiZoom, window: &mut Window, cx: &mut Context<Self>) {
        self.set_ui_scale(1.0, window, cx);
    }

    fn toggle_auto_collapse_inspector(
        &mut self,
        _: &ToggleAutoCollapseInspector,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.inspector_auto_hide = !self.inspector_auto_hide;
        self.save_prefs();
        let msg = if self.inspector_auto_hide {
            "Inspector: ocultar quando nada selecionado (ativado)"
        } else {
            "Inspector: sempre visível (desativado)"
        };
        window.push_notification(msg, cx);
        cx.notify();
    }

    fn reset_panels(&mut self, _: &ResetPanels, _: &mut Window, cx: &mut Context<Self>) {
        let d = crate::settings::Settings::default();
        self.widths = (d.sidebar_w, d.editor_w, d.inspector_w);
        self.col_type_w = d.col_type_w;
        self.panels_gen += 1;
        self.save_prefs();
        cx.notify();
    }

    fn toggle_card_transparency(
        &mut self,
        _: &ToggleCardTransparency,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.translucent_cards = !self.translucent_cards;
        self.save_prefs();
        cx.notify();
    }

    fn toggle_editor(&mut self, _: &ToggleEditor, _: &mut Window, cx: &mut Context<Self>) {
        self.toggle_left_pane(None, cx);
    }

    fn toggle_sidebar(&mut self, _: &ToggleSidebar, _: &mut Window, cx: &mut Context<Self>) {
        if self.sidebar_open && !self.editor_open {
            self.sidebar_open = false;
        } else {
            self.sidebar_open = true;
            self.editor_open = false;
        }
        self.save_prefs();
        cx.notify();
    }

    fn toggle_inspector(&mut self, _: &ToggleInspector, _: &mut Window, cx: &mut Context<Self>) {
        self.inspector_open = !self.inspector_open;
        self.save_prefs();
        cx.notify();
    }

    fn toggle_legend(&mut self, _: &ToggleLegend, _: &mut Window, cx: &mut Context<Self>) {
        self.with_diagram(cx, |d| d.legend_open = !d.legend_open);
    }

    fn fit(&mut self, _: &FitView, _: &mut Window, cx: &mut Context<Self>) {
        self.with_diagram(cx, |d| d.fit());
    }

    fn relayout(&mut self, _: &Relayout, _: &mut Window, cx: &mut Context<Self>) {
        self.with_diagram(cx, |d| d.relayout());
    }

    fn zoom_in(&mut self, _: &ZoomIn, _: &mut Window, cx: &mut Context<Self>) {
        self.with_diagram(cx, |d| d.zoom_by(1.25));
    }

    fn zoom_out(&mut self, _: &ZoomOut, _: &mut Window, cx: &mut Context<Self>) {
        self.with_diagram(cx, |d| d.zoom_by(1. / 1.25));
    }

    fn tool_select(&mut self, _: &ToolSelect, _: &mut Window, cx: &mut Context<Self>) {
        self.with_diagram(cx, |d| d.tool = Tool::Select);
    }

    fn tool_hand(&mut self, _: &ToolHand, _: &mut Window, cx: &mut Context<Self>) {
        self.with_diagram(cx, |d| d.tool = Tool::Hand);
    }

    fn deselect(&mut self, _: &Deselect, _: &mut Window, cx: &mut Context<Self>) {
        self.with_diagram(cx, |d| d.deselect());
    }

    // ---- partes da tela ----

    fn titlebar(&self, g: &Glass, maximized: bool, compact: bool, cx: &mut Context<Self>) -> Div {
        let mut tabs = div()
            .h_full()
            .flex()
            .items_end()
            .gap(px(2.))
            .min_w_0()
            .flex_shrink(1.0)
            .overflow_x_hidden();
        for (i, doc) in self.docs.iter().enumerate() {
            let on = i == self.active;
            tabs = tabs.child(
                div()
                    .id(("tab", i))
                    .role(Role::Tab)
                    .aria_label(doc.title.clone())
                    .group("tab")
                    .h(px(32.))
                    .max_w(px(220.))
                    .min_w(px(70.))
                    .flex_shrink(1.0)
                    .pl(px(10.))
                    .pr(px(4.))
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .rounded_t(px(7.))
                    .text_size(TEXT_SM)
                    .text_color(if on { g.text } else { g.text_muted })
                    .when(on, |d| {
                        d.bg(g.surface).border_1().border_b_0().border_color(g.edge)
                    })
                    .when(!on, |d| {
                        let h = g.hover;
                        d.hover(move |s| s.bg(h))
                    })
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.active = i;
                        cx.notify();
                    }))
                    // clique do meio fecha a aba (como em navegadores e no Zed)
                    .on_mouse_down(
                        MouseButton::Middle,
                        cx.listener(move |this, _, _, cx| this.close_tab_at(i, cx)),
                    )
                    .child(Icon::new(IconName::Database).xsmall().text_color(if on {
                        g.accent
                    } else {
                        g.text_faint
                    }))
                    .child(div().truncate().child(doc.title.clone()))
                    .child(
                        div()
                            .id(("tab-close", i))
                            .role(Role::Button)
                            .aria_label("Fechar aba")
                            .size(px(18.))
                            .rounded(px(4.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .hover({
                                let h = g.hover;
                                move |s| s.bg(h)
                            })
                            .on_click(cx.listener(move |this, _, _, cx| {
                                cx.stop_propagation();
                                this.close_tab_at(i, cx);
                            }))
                            // alterado: ponto; senão o X (visível na aba ativa ou no hover)
                            .child(if doc.dirty {
                                div()
                                    .size(px(7.))
                                    .rounded_full()
                                    .bg(g.text_muted)
                                    .into_any_element()
                            } else {
                                div()
                                    .when(!on, |d| {
                                        d.invisible().group_hover("tab", |s| s.visible())
                                    })
                                    .child(Icon::new(IconName::X).xsmall().text_color(g.text_faint))
                                    .into_any_element()
                            }),
                    ),
            );
        }
        tabs = tabs.child(
            div().h(px(32.)).flex().items_center().child(
                Button::new("tab-new")
                    .ghost()
                    .xsmall()
                    .icon(IconName::Plus)
                    .tooltip_with_action("Novo arquivo", &NewFile, Some(actions::CTX))
                    .on_click(|_, window, cx| window.dispatch_action(Box::new(NewFile), cx)),
            ),
        );

        let main_menu = Button::new("main-menu")
            .ghost()
            .small()
            .icon(IconName::Menu)
            .dropdown_menu(|mut menu, _, _| {
                let mut last = "";
                for e in actions::catalog() {
                    if e.group != last {
                        if !last.is_empty() {
                            menu = menu.separator();
                        }
                        menu = menu.label(e.group);
                        last = e.group;
                    }
                    menu = menu.menu(e.label, e.action);
                }
                menu
            });
        let theme_icon = if self.mode == Mode::Dark {
            IconName::Sun
        } else {
            IconName::Moon
        };
        let mut bar = div()
            .h(TITLEBAR_H)
            .w_full()
            .flex_none()
            .flex()
            .items_center()
            .gap(px(6.))
            .pl(px(8.))
            .bg(g.chrome)
            .border_b_1()
            .border_color(g.edge)
            .child(main_menu)
            .child(
                img("brand/icon-64.png")
                    .size(px(20.))
                    .mx(px(2.))
                    .flex_none(),
            )
            .child(tabs)
            .child(
                div()
                    .id("titlebar-drag")
                    .window_control_area(WindowControlArea::Drag)
                    .flex_1()
                    .h_full()
                    .when(cfg!(not(windows)), |d| {
                        d.on_double_click(|_, window, _| window.zoom_window())
                            .on_mouse_down_out(cx.listener(|this, _, _, _| {
                                this.dragging_window = false;
                            }))
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _, _, _| {
                                    this.dragging_window = true;
                                }),
                            )
                            .on_mouse_down(MouseButton::Right, |ev, window, _| {
                                window.show_window_menu(ev.position);
                            })
                            .on_mouse_up(
                                MouseButton::Left,
                                cx.listener(|this, _, _, _| {
                                    this.dragging_window = false;
                                }),
                            )
                            .on_mouse_move(cx.listener(|this, _, window, _| {
                                if this.dragging_window {
                                    this.dragging_window = false;
                                    window.start_window_move();
                                }
                            }))
                    }),
            )
            .child(
                div()
                    .id("palette-pill")
                    .role(Role::Button)
                    .aria_label("Buscar comandos (Ctrl+K)")
                    .flex_none()
                    .flex()
                    .items_center()
                    .gap(SP_SM)
                    .h(px(26.))
                    .w(if compact { px(30.) } else { px(220.) })
                    .when(compact, |d| d.justify_center())
                    .px(SP_SM)
                    .rounded(px(6.))
                    .border_1()
                    .border_color(g.edge)
                    .bg(g.sidebar)
                    .text_size(TEXT_XS)
                    .text_color(g.text_faint)
                    .hover({
                        let h = g.hover;
                        move |s| s.bg(h)
                    })
                    .on_click(|_, window, cx| window.dispatch_action(Box::new(OpenPalette), cx))
                    .child(Icon::new(IconName::Search).xsmall())
                    .when(!compact, |d| {
                        d.child(div().flex_1().child("Buscar comandos…"))
                            .child("Ctrl K")
                    }),
            )
            .child(
                div().flex_none().child(
                    Button::new("theme-toggle")
                        .ghost()
                        .small()
                        .icon(theme_icon)
                        .tooltip_with_action("Alternar tema", &ToggleTheme, Some(actions::CTX))
                        .on_click(|_, window, cx| {
                            window.dispatch_action(Box::new(ToggleTheme), cx)
                        }),
                ),
            );
        #[cfg(windows)]
        {
            // botões nativos do Windows: glifos da fonte do sistema, 46 px, fechar vermelho
            let font = crate::platform::caption_font();
            let caption = |id: &'static str,
                           label: &'static str,
                           area: WindowControlArea,
                           glyph: &'static str,
                           close: bool| {
                let (hover, ink) = (if close { CAPTION_CLOSE } else { g.hover }, g.text_muted);
                let hover_ink = if close { gpui_kit::white() } else { g.text };
                div()
                    .id(id)
                    .role(Role::Button)
                    .aria_label(label)
                    .window_control_area(area)
                    .occlude()
                    .w(px(46.))
                    .h_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .font_family(font)
                    .text_size(px(10.))
                    .text_color(ink)
                    .hover(move |s| s.bg(hover).text_color(hover_ink))
                    .active(move |s| s.bg(hover.opacity(0.8)))
                    .child(glyph)
            };
            let (max_label, max_glyph) = if maximized {
                ("Restaurar", "\u{e923}")
            } else {
                ("Maximizar", "\u{e922}")
            };
            bar = bar.child(
                div()
                    .h_full()
                    .flex_none()
                    .flex()
                    .child(caption(
                        "cap-min",
                        "Minimizar",
                        WindowControlArea::Min,
                        "\u{e921}",
                        false,
                    ))
                    .child(caption(
                        "cap-max",
                        max_label,
                        WindowControlArea::Max,
                        max_glyph,
                        false,
                    ))
                    .child(caption(
                        "cap-close",
                        "Fechar",
                        WindowControlArea::Close,
                        "\u{e8bb}",
                        true,
                    )),
            );
        }
        #[cfg(not(windows))]
        {
            let (max_label, max_icon) = if maximized {
                ("Restaurar", IconName::WindowRestore)
            } else {
                ("Maximizar", IconName::WindowMaximize)
            };

            bar = bar.child(
                div()
                    .h_full()
                    .flex_none()
                    .flex()
                    .items_center()
                    .child(
                        div()
                            .id("cap-min")
                            .role(Role::Button)
                            .aria_label("Minimizar")
                            .occlude()
                            .w(px(38.))
                            .h_full()
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(g.text_muted)
                            .hover({
                                let h = g.hover;
                                let ink = g.text;
                                move |s| s.bg(h).text_color(ink)
                            })
                            .active({
                                let h = g.hover;
                                move |s| s.bg(h.opacity(0.8))
                            })
                            .on_mouse_down(MouseButton::Left, |_, window, cx| {
                                window.prevent_default();
                                cx.stop_propagation();
                            })
                            .on_click(|_, window, cx| {
                                cx.stop_propagation();
                                window.minimize_window();
                            })
                            .child(Icon::new(IconName::WindowMinimize).small()),
                    )
                    .child(
                        div()
                            .id("cap-max")
                            .role(Role::Button)
                            .aria_label(max_label)
                            .occlude()
                            .w(px(38.))
                            .h_full()
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(g.text_muted)
                            .hover({
                                let h = g.hover;
                                let ink = g.text;
                                move |s| s.bg(h).text_color(ink)
                            })
                            .active({
                                let h = g.hover;
                                move |s| s.bg(h.opacity(0.8))
                            })
                            .on_mouse_down(MouseButton::Left, |_, window, cx| {
                                window.prevent_default();
                                cx.stop_propagation();
                            })
                            .on_click(|_, window, cx| {
                                cx.stop_propagation();
                                window.zoom_window();
                            })
                            .child(Icon::new(max_icon).small()),
                    )
                    .child(
                        div()
                            .id("cap-close")
                            .role(Role::Button)
                            .aria_label("Fechar")
                            .occlude()
                            .w(px(38.))
                            .h_full()
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(g.text_muted)
                            .hover(move |s| s.bg(CAPTION_CLOSE).text_color(gpui_kit::white()))
                            .active(move |s| s.bg(CAPTION_CLOSE.opacity(0.8)))
                            .on_mouse_down(MouseButton::Left, |_, window, cx| {
                                window.prevent_default();
                                cx.stop_propagation();
                            })
                            .on_click(|_, _, cx| {
                                cx.stop_propagation();
                                cx.quit();
                            })
                            .child(Icon::new(IconName::WindowClose).small()),
                    ),
            );
        }
        bar
    }

    /// Barra de atividades (ícones à esquerda), como no VS Code / GitHub Desktop.
    fn rail(&self, g: &Glass) -> Div {
        let errors = self.doc().map_or(0, |d| d.counts().0);
        let item = |id: &'static str,
                    icon: IconName,
                    tip: &'static str,
                    action: Box<dyn Action>,
                    is_active: bool,
                    badge: usize| {
            let on = is_active;
            let tip_action = action.boxed_clone();
            div()
                .id(id)
                .role(Role::Tab)
                .aria_label(tip)
                .relative()
                .size(px(34.))
                .rounded(px(7.))
                .flex()
                .items_center()
                .justify_center()
                .text_color(if on { g.text } else { g.text_faint })
                .when(on, |d| d.bg(g.hover))
                .hover({
                    let (h, t) = (g.hover, g.text);
                    move |s| s.bg(h).text_color(t)
                })
                .active({
                    let a = g.active;
                    move |s| s.bg(a)
                })
                .tooltip(move |window, cx| {
                    Tooltip::new(tip)
                        .action(tip_action.as_ref(), Some(actions::CTX))
                        .build(window, cx)
                })
                .on_click(move |_, window, cx| window.dispatch_action(action.boxed_clone(), cx))
                .child(Icon::new(icon))
                .when(badge > 0, |d| {
                    d.child(
                        div()
                            .absolute()
                            .top(px(3.))
                            .right(px(3.))
                            .min_w(px(14.))
                            .h(px(14.))
                            .px(px(3.))
                            .rounded_full()
                            // tintado (como no Zed), não vermelho sólido
                            .bg(g.danger.opacity(0.22))
                            .border_1()
                            .border_color(g.danger.opacity(0.55))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_size(px(9.))
                            .text_color(g.danger)
                            .child(badge.to_string()),
                    )
                })
        };
        div()
            .w(RAIL_W)
            .h_full()
            .flex_none()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(4.))
            .py(px(8.))
            .bg(g.chrome)
            .border_r_1()
            .border_color(g.edge)
            .child(item(
                "rail-files",
                IconName::Folder,
                "Arquivos do projeto",
                Box::new(ShowFiles),
                self.sidebar_open && !self.editor_open && self.pane == Pane::Files,
                0,
            ))
            .child(item(
                "rail-schema",
                IconName::Table2,
                "Esquema",
                Box::new(ShowSchema),
                self.sidebar_open && !self.editor_open && self.pane == Pane::Schema,
                0,
            ))
            .child(item(
                "rail-problems",
                IconName::TriangleAlert,
                "Problemas",
                Box::new(ShowProblems),
                self.sidebar_open && !self.editor_open && self.pane == Pane::Problems,
                errors,
            ))
            .child(item(
                "rail-editor",
                IconName::Braces,
                "Editor DBML",
                Box::new(ToggleEditor),
                self.editor_open,
                0,
            ))
            .child(div().flex_1())
            .child({
                let (mode, translucent, ui_scale, auto_hide) = (
                    self.mode,
                    self.translucent_cards,
                    self.ui_scale,
                    self.inspector_auto_hide,
                );
                Button::new("rail-appearance")
                    .ghost()
                    .icon(IconName::Palette)
                    .tooltip("Aparência, tamanho da interface e painéis")
                    .dropdown_menu(move |menu, _, _| {
                        menu.label("Tema")
                            .menu_with_check("Escuro", mode == Mode::Dark, Box::new(ToggleTheme))
                            .menu_with_check("Claro", mode == Mode::Light, Box::new(ToggleTheme))
                            .separator()
                            .label("Tamanho da interface (Ctrl+Shift+= / -)")
                            .menu_with_check(
                                "Padrão (100%)",
                                (ui_scale - 1.0).abs() < 0.05,
                                Box::new(ResetUiZoom),
                            )
                            .menu_with_check(
                                "Médio (115%)",
                                (ui_scale - 1.15).abs() < 0.05,
                                Box::new(ZoomInUi),
                            )
                            .menu_with_check(
                                "Grande (130%)",
                                (ui_scale - 1.30).abs() < 0.05,
                                Box::new(ZoomInUi),
                            )
                            .separator()
                            .label("Painéis")
                            .menu_with_check(
                                "Recolher inspector quando nada selecionado",
                                auto_hide,
                                Box::new(ToggleAutoCollapseInspector),
                            )
                            .menu_with_check(
                                "Cartões translúcidos",
                                translucent,
                                Box::new(ToggleCardTransparency),
                            )
                    })
            })
            .child(item(
                "rail-keys",
                IconName::Keyboard,
                "Atalhos de teclado",
                Box::new(ShowShortcuts),
                false,
                0,
            ))
            .child(item(
                "rail-palette",
                IconName::Command,
                "Paleta de comandos",
                Box::new(OpenPalette),
                false,
                0,
            ))
    }

    /// Barra de atividades da direita (Inspector e Legenda), simétrica à barra da esquerda.
    fn right_rail(&self, g: &Glass, cx: &App) -> Div {
        let (has_selection, legend_open) = self.doc().map_or((false, false), |d| {
            let diag = d.diagram.read(cx);
            (
                diag.selected().is_some() || !diag.selection().is_empty(),
                diag.legend_open,
            )
        });

        let item = |id: &'static str,
                    icon: IconName,
                    tip: &'static str,
                    action: Box<dyn Action>,
                    is_active: bool,
                    badge: usize| {
            let on = is_active;
            let tip_action = action.boxed_clone();
            div()
                .id(id)
                .role(Role::Tab)
                .aria_label(tip)
                .relative()
                .size(px(34.))
                .rounded(px(7.))
                .flex()
                .items_center()
                .justify_center()
                .text_color(if on { g.text } else { g.text_faint })
                .when(on, |d| d.bg(g.hover))
                .hover({
                    let (h, t) = (g.hover, g.text);
                    move |s| s.bg(h).text_color(t)
                })
                .active({
                    let a = g.active;
                    move |s| s.bg(a)
                })
                .tooltip(move |window, cx| {
                    Tooltip::new(tip)
                        .action(tip_action.as_ref(), Some(actions::CTX))
                        .build(window, cx)
                })
                .on_click(move |_, window, cx| window.dispatch_action(action.boxed_clone(), cx))
                .child(Icon::new(icon))
                .when(badge > 0, |d| {
                    d.child(
                        div()
                            .absolute()
                            .top(px(3.))
                            .right(px(3.))
                            .min_w(px(14.))
                            .h(px(14.))
                            .px(px(3.))
                            .rounded_full()
                            .bg(g.accent.opacity(0.25))
                            .border_1()
                            .border_color(g.accent.opacity(0.6))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_size(px(9.))
                            .text_color(g.accent)
                            .child(badge.to_string()),
                    )
                })
        };

        div()
            .w(RAIL_W)
            .h_full()
            .flex_none()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(4.))
            .py(px(8.))
            .bg(g.chrome)
            .border_l_1()
            .border_color(g.edge)
            .child(item(
                "rail-inspector",
                IconName::SlidersHorizontal,
                "Inspector / Detalhes (Ctrl+I)",
                Box::new(ToggleInspector),
                self.inspector_open,
                if has_selection { 1 } else { 0 },
            ))
            .child(item(
                "rail-legend",
                IconName::Info,
                "Legenda do diagrama (Ctrl+Shift+L)",
                Box::new(ToggleLegend),
                legend_open,
                0,
            ))
            .child(div().flex_1())
    }

    fn section(g: &Glass, text: impl Into<SharedString>, count: Option<usize>) -> Div {
        div()
            .flex()
            .justify_between()
            .items_center()
            .px(SP_MD)
            .pt(SP_MD)
            .pb(px(4.))
            .text_size(TEXT_XS)
            .font_weight(FontWeight::MEDIUM)
            .text_color(g.text_faint)
            .child(text.into())
            .children(count.map(|n| div().child(n.to_string())))
    }

    fn row(g: &Glass, id: impl Into<ElementId>, on: bool) -> Stateful<Div> {
        div()
            .id(id)
            .role(Role::Button)
            .mx(px(6.))
            .h(px(28.))
            .px(px(8.))
            .rounded(px(6.))
            .flex()
            .items_center()
            .gap(SP_SM)
            .text_size(TEXT_SM)
            .text_color(if on { g.text } else { g.text_muted })
            .when(on, |d| d.bg(g.selected))
            .hover({
                let h = g.hover;
                move |s| s.bg(h)
            })
            .active({
                let a = g.active;
                move |s| s.bg(a)
            })
    }

    fn files_pane(&self, g: &Glass, cx: &mut Context<Self>) -> Div {
        let active_path = self.doc().and_then(|d| d.path.clone());
        let mut list = div().flex().flex_col().gap(px(2.)).px(px(6.));

        let (folder_title, count) = match &self.project {
            Some(p) => (file_title(p), self.files.len()),
            None => ("Arquivos Abertos".to_string(), self.docs.len()),
        };

        let header = div()
            .flex()
            .justify_between()
            .items_center()
            .px(SP_MD)
            .pt(SP_MD)
            .pb(px(8.))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .min_w_0()
                    .flex_1()
                    .child(
                        Icon::new(if self.project.is_some() {
                            IconName::FolderOpen
                        } else {
                            IconName::Files
                        })
                        .xsmall()
                        .text_color(g.accent),
                    )
                    .child(
                        div()
                            .text_size(TEXT_SM)
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(g.text)
                            .truncate()
                            .child(folder_title),
                    )
                    .child(
                        div()
                            .px(px(5.))
                            .py(px(1.))
                            .rounded_full()
                            .bg(g.hover)
                            .text_size(px(10.))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(g.text_muted)
                            .child(count.to_string()),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(2.))
                    .child(
                        Button::new("pane-new-file")
                            .ghost()
                            .xsmall()
                            .icon(IconName::Plus)
                            .tooltip_with_action("Novo arquivo", &NewFile, Some(actions::CTX))
                            .on_click(|_, window, cx| {
                                window.dispatch_action(Box::new(NewFile), cx)
                            }),
                    )
                    .child(
                        Button::new("pane-open-folder")
                            .ghost()
                            .xsmall()
                            .icon(IconName::FolderOpen)
                            .tooltip_with_action("Abrir pasta", &OpenFolder, Some(actions::CTX))
                            .on_click(|_, window, cx| {
                                window.dispatch_action(Box::new(OpenFolder), cx)
                            }),
                    ),
            );

        if let Some(root) = self.project.clone() {
            for (i, f) in self.files.iter().enumerate() {
                let on = active_path.as_ref() == Some(f);
                let rel = f
                    .strip_prefix(&root)
                    .ok()
                    .and_then(|r| r.parent())
                    .map(|p| p.display().to_string())
                    .unwrap_or_default();
                let path = f.clone();
                let fname = file_title(f);
                let is_dirty = self
                    .docs
                    .iter()
                    .find(|d| d.path.as_ref() == Some(f))
                    .is_some_and(|d| d.dirty);

                list = list.child(
                    div()
                        .id(("file", i))
                        .role(Role::Button)
                        .aria_label(fname.clone())
                        .h(px(32.))
                        .px(px(8.))
                        .rounded(px(6.))
                        .flex()
                        .items_center()
                        .gap(px(7.))
                        .text_size(TEXT_SM)
                        .text_color(if on { g.text } else { g.text_muted })
                        .when(on, |d| {
                            d.bg(g.selected)
                                .font_weight(FontWeight::MEDIUM)
                                .border_l_2()
                                .border_color(g.accent)
                        })
                        .when(!on, |d| {
                            let h = g.hover;
                            d.hover(move |s| s.bg(h))
                        })
                        .active({
                            let a = g.active;
                            move |s| s.bg(a)
                        })
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.open_path(path.clone(), window, cx)
                        }))
                        .child(
                            Icon::new(IconName::Database)
                                .size(px(14.))
                                .text_color(if on { g.accent } else { g.text_faint }),
                        )
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .flex()
                                .flex_col()
                                .justify_center()
                                .child(div().truncate().child(fname))
                                .when(!rel.is_empty(), |d| {
                                    d.child(
                                        div()
                                            .text_size(px(10.))
                                            .text_color(g.text_faint)
                                            .truncate()
                                            .child(rel),
                                    )
                                }),
                        )
                        .child(
                            div()
                                .px(px(4.))
                                .py(px(1.))
                                .rounded(px(3.))
                                .bg(g.edge_soft)
                                .text_size(px(9.))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(g.text_faint)
                                .child("DBML"),
                        )
                        .when(is_dirty, |r| {
                            r.child(div().size(px(6.)).rounded_full().bg(g.accent))
                        }),
                );
            }
        } else {
            for (i, d) in self.docs.iter().enumerate() {
                let on = i == self.active;
                let title = d.title.clone();
                list = list.child(
                    div()
                        .id(("open", i))
                        .role(Role::Button)
                        .aria_label(title.clone())
                        .h(px(32.))
                        .px(px(8.))
                        .rounded(px(6.))
                        .flex()
                        .items_center()
                        .gap(px(7.))
                        .text_size(TEXT_SM)
                        .text_color(if on { g.text } else { g.text_muted })
                        .when(on, |st| {
                            st.bg(g.selected)
                                .font_weight(FontWeight::MEDIUM)
                                .border_l_2()
                                .border_color(g.accent)
                        })
                        .when(!on, |st| {
                            let h = g.hover;
                            st.hover(move |s| s.bg(h))
                        })
                        .active({
                            let a = g.active;
                            move |s| s.bg(a)
                        })
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.active = i;
                            cx.notify();
                        }))
                        .child(
                            Icon::new(IconName::Database)
                                .size(px(14.))
                                .text_color(if on { g.accent } else { g.text_faint }),
                        )
                        .child(div().flex_1().truncate().child(title))
                        .child(
                            div()
                                .px(px(4.))
                                .py(px(1.))
                                .rounded(px(3.))
                                .bg(g.edge_soft)
                                .text_size(px(9.))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(g.text_faint)
                                .child("DBML"),
                        )
                        .when(d.dirty, |r| {
                            r.child(div().size(px(6.)).rounded_full().bg(g.accent))
                        }),
                );
            }
        }

        div()
            .flex()
            .flex_col()
            .gap(px(4.))
            .child(header)
            .child(list)
            .when(self.project.is_none(), |d| {
                d.child(
                    div()
                        .mx(px(8.))
                        .mt(SP_MD)
                        .p(SP_MD)
                        .rounded(px(8.))
                        .border_1()
                        .border_color(g.edge)
                        .bg(g.surface.opacity(0.4))
                        .flex()
                        .flex_col()
                        .items_center()
                        .text_center()
                        .gap(px(6.))
                        .child(
                            Icon::new(IconName::FolderOpen)
                                .size(px(24.))
                                .text_color(g.text_faint),
                        )
                        .child(
                            div()
                                .text_size(TEXT_SM)
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(g.text)
                                .child("Abrir Pasta de Projeto"),
                        )
                        .child(
                            div()
                                .text_size(TEXT_XS)
                                .text_color(g.text_muted)
                                .child("Navegue entre todos os arquivos .dbml de forma integrada."),
                        )
                        .child(
                            div().mt(px(4.)).child(
                                Button::new("banner-open-folder")
                                    .primary()
                                    .small()
                                    .icon(IconName::FolderOpen)
                                    .label("Abrir pasta…")
                                    .on_click(|_, window, cx| {
                                        window.dispatch_action(Box::new(OpenFolder), cx)
                                    }),
                            ),
                        ),
                )
            })
    }

    fn schema_pane(&self, g: &Glass, cx: &mut Context<Self>) -> Div {
        let Some(doc) = self.doc() else { return div() };
        let d = doc.diagram.read(cx);
        let sel = d.selection();
        let schema = d.schema().clone();
        let row = |t: &schema::Table, cx: &mut Context<Self>| {
            let on = sel.contains(&t.name);
            let name = t.name.clone();
            let tint = t.color.as_deref().and_then(hex);
            Self::row(g, SharedString::from(format!("side-{}", t.name)), on)
                .aria_label(format!("Tabela {}", t.name))
                .on_click(cx.listener(move |this, _, _, cx| {
                    let n = name.clone();
                    this.with_diagram(cx, |d| d.focus_table(&n));
                    this.inspector_open = true;
                    this.save_prefs();
                    cx.notify();
                }))
                .child(
                    Icon::new(IconName::Table2)
                        .xsmall()
                        .text_color(tint.unwrap_or(if on { g.accent } else { g.text_faint })),
                )
                .child(div().flex_1().truncate().child(t.name.clone()))
                .child(
                    div()
                        .text_size(TEXT_XS)
                        .text_color(g.text_faint)
                        .child(t.columns.len().to_string()),
                )
        };
        let mut body = div().flex().flex_col().gap(px(1.));
        for gr in &schema.groups {
            let c = gr
                .color
                .as_deref()
                .and_then(hex)
                .unwrap_or_else(|| crate::canvas::group_color(&schema, &gr.name));
            body = body.child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .px(SP_MD)
                    .pt(SP_SM)
                    .pb(px(2.))
                    .text_size(TEXT_XS)
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(g.text_muted)
                    .child(div().size(px(7.)).rounded_full().bg(c))
                    .child(div().flex_1().child(gr.name.clone()))
                    .child(
                        div()
                            .font_weight(FontWeight::NORMAL)
                            .text_color(g.text_faint)
                            .child(gr.tables.len().to_string()),
                    ),
            );
            for t in gr.tables.iter().filter_map(|n| schema.table(n)) {
                body = body.child(row(t, cx).pl(px(22.)));
            }
        }
        let loose: Vec<_> = schema
            .tables
            .iter()
            .filter(|t| schema.group_of(&t.name).is_none())
            .collect();
        if !schema.groups.is_empty() && !loose.is_empty() {
            body = body.child(
                div()
                    .px(SP_MD)
                    .pt(SP_SM)
                    .pb(px(2.))
                    .text_size(TEXT_XS)
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(g.text_faint)
                    .child("Sem grupo"),
            );
        }
        for t in loose {
            body = body.child(row(t, cx));
        }
        let mut refs = div().flex().flex_col().gap(px(2.)).px(SP_MD);
        for r in &schema.refs {
            refs = refs.child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .h(px(22.))
                    .text_size(TEXT_XS)
                    .text_color(g.text_muted)
                    .child(Icon::new(IconName::Link2).xsmall().text_color(g.text_faint))
                    .child(
                        div()
                            .truncate()
                            .child(format!("{} → {}", r.from.table, r.to.table)),
                    ),
            );
        }
        div()
            .flex()
            .flex_col()
            .child(Self::section(g, "TABELAS", Some(schema.tables.len())))
            .child(body)
            .child(Self::section(g, "RELAÇÕES", Some(schema.refs.len())))
            .child(refs)
    }

    fn problems_pane(&self, g: &Glass, cx: &mut Context<Self>) -> Div {
        let Some(doc) = self.doc() else { return div() };
        let text = doc.editor.read(cx).value().to_string();
        let mut list = div().flex().flex_col().gap(px(1.));
        for (i, d) in doc.diags.iter().enumerate() {
            let (icon, color) = match d.severity {
                Severity::Error => (IconName::CircleX, g.danger),
                Severity::Warning => (IconName::TriangleAlert, g.warning),
                Severity::Info => (IconName::Info, g.text_faint),
            };
            let line = text[..d.range.start.min(text.len())].matches('\n').count() + 1;
            let at = d.range.start;
            list = list.child(
                Self::row(g, ("problem", i), false)
                    .h_auto()
                    .py(px(5.))
                    .items_start()
                    .aria_label(d.message.clone())
                    .on_click(cx.listener(move |this, _, window, cx| this.reveal(at, window, cx)))
                    .child(
                        div()
                            .pt(px(2.))
                            .child(Icon::new(icon).xsmall().text_color(color)),
                    )
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .child(div().text_color(g.text).child(d.message.clone()))
                            .child(
                                div()
                                    .text_size(TEXT_XS)
                                    .text_color(g.text_faint)
                                    .child(format!("{} · linha {line}", doc.title)),
                            ),
                    ),
            );
        }
        let body = if doc.diags.is_empty() {
            div()
                .p(SP_LG)
                .flex()
                .flex_col()
                .items_center()
                .gap(SP_SM)
                .text_size(TEXT_SM)
                .text_color(g.text_muted)
                .child(Icon::new(IconName::CircleCheck).text_color(g.ok))
                .child("Nenhum problema encontrado.")
        } else {
            list
        };
        div()
            .flex()
            .flex_col()
            .child(Self::section(g, "PROBLEMAS", Some(doc.diags.len())))
            .child(body)
    }

    fn sidebar(&self, g: &Glass, cx: &mut Context<Self>) -> Stateful<Div> {
        let content = match self.pane {
            Pane::Files => self.files_pane(g, cx),
            Pane::Schema => self.schema_pane(g, cx),
            Pane::Problems => self.problems_pane(g, cx),
        };
        div()
            .id("sidebar")
            .role(Role::Region)
            .aria_label("Barra lateral")
            .size_full()
            .bg(g.sidebar)
            .overflow_y_scroll()
            .child(content)
    }

    fn inspector(&self, g: &Glass, window: &Window, cx: &mut Context<Self>) -> Stateful<Div> {
        let tag = |text: &'static str| {
            div()
                .px(px(5.))
                .rounded(px(4.))
                .border_1()
                .border_color(g.edge)
                .text_size(TEXT_XS)
                .text_color(g.text_muted)
                .child(text)
        };
        let (selection, selected, group) = match self.doc() {
            Some(doc) => {
                let d = doc.diagram.read(cx);
                let sel = d.selection();
                let one = d.selected().map(|t| {
                    (
                        t.clone(),
                        d.refs_of(&t.name).into_iter().cloned().collect::<Vec<_>>(),
                    )
                });
                let group = one
                    .as_ref()
                    .and_then(|(t, _)| d.schema().group_of(&t.name).cloned());
                (sel, one, group)
            }
            None => (Vec::new(), None, None),
        };
        let header = |eyebrow: &'static str, title: String, sub: String| {
            div()
                .flex()
                .flex_col()
                .gap(px(2.))
                .p(SP_MD)
                .border_b_1()
                .border_color(g.edge)
                .child(
                    div()
                        .text_size(TEXT_XS)
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(g.text_faint)
                        .child(eyebrow),
                )
                .child(
                    div()
                        .text_size(TEXT_LG)
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(g.text)
                        .child(title),
                )
                .child(div().text_size(TEXT_SM).text_color(g.text_muted).child(sub))
        };
        let body = if selection.len() > 1 {
            // várias tabelas: resumo + agrupar
            let mut list = div().flex().flex_col().gap(px(2.)).px(SP_MD);
            for n in &selection {
                list = list.child(
                    div()
                        .flex()
                        .items_center()
                        .gap(SP_SM)
                        .h(px(26.))
                        .text_size(TEXT_SM)
                        .child(
                            Icon::new(IconName::Table2)
                                .xsmall()
                                .text_color(g.text_faint),
                        )
                        .child(n.clone()),
                );
            }
            div()
                .flex()
                .flex_col()
                .child(header(
                    "SELEÇÃO",
                    format!("{} tabelas", selection.len()),
                    "Shift+clique adiciona ou remove".into(),
                ))
                .child(Self::section(g, "TABELAS", None))
                .child(list)
                .child(
                    div().p(SP_MD).flex().flex_col().gap(SP_SM).child(
                        Button::new("insp-group")
                            .primary()
                            .small()
                            .w_full()
                            .icon(IconName::LayoutGrid)
                            .label("Agrupar seleção")
                            .tooltip_with_action(
                                "Agrupar tabelas selecionadas",
                                &GroupSelection,
                                Some(actions::CTX),
                            )
                            .on_click(|_, window, cx| {
                                window.dispatch_action(Box::new(GroupSelection), cx)
                            }),
                    ),
                )
        } else if let Some((t, refs)) = selected {
            // edição em linha: clique no texto abre o campo; Enter ou sair confirma
            let editing = self
                .editing
                .as_ref()
                .map(|(t, st, _)| (t.clone(), st.clone()));
            let field = |target: EditTarget,
                         text: String,
                         muted: bool,
                         cx: &mut Context<Self>|
             -> AnyElement {
                match &editing {
                    Some((t, st)) if *t == target => Input::new(st).small().into_any_element(),
                    _ => {
                        let id = SharedString::from(format!("edit-{target:?}"));
                        let value = text.clone();
                        div()
                            .id(id)
                            .role(Role::Button)
                            .aria_label(format!("Editar {text}"))
                            .w_full()
                            .px(px(4.))
                            .rounded(px(4.))
                            .truncate()
                            .text_color(if muted { g.text_faint } else { g.text })
                            .cursor_text()
                            .hover({
                                let h = g.hover;
                                move |st| st.bg(h)
                            })
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.start_edit(target.clone(), value.clone(), window, cx)
                            }))
                            .child(text)
                            .into_any_element()
                    }
                }
            };
            let col_type_w = self.col_type_w;
            let mut cols = div().flex().flex_col();
            let col_header = div()
                .flex()
                .items_center()
                .gap(px(6.))
                .px(SP_MD)
                .py(px(4.))
                .border_b_1()
                .border_color(g.edge_soft)
                .text_size(px(10.))
                .font_weight(FontWeight::MEDIUM)
                .text_color(g.text_faint)
                .child(div().flex_1().min_w(px(60.)).child("NOME"))
                .child(
                    div()
                        .id("col-splitter-hdr")
                        .w(px(8.))
                        .h(px(14.))
                        .mx(px(-2.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .cursor(CursorStyle::ResizeLeftRight)
                        .tooltip(|window, cx| {
                            Tooltip::new(
                                "Arrastar para redimensionar colunas (duplo-clique restaura)",
                            )
                            .build(window, cx)
                        })
                        .hover(|s| s.bg(g.hover).rounded(px(2.)))
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, e: &MouseDownEvent, _, cx| {
                                if e.click_count >= 2 {
                                    this.col_type_w = 110.0;
                                    this.save_prefs();
                                    cx.notify();
                                } else {
                                    this.col_resize_start = Some((e.position.x, this.col_type_w));
                                }
                            }),
                        )
                        .child(div().w(px(1.5)).h(px(10.)).bg(g.edge)),
                )
                .child(div().flex_none().w(px(col_type_w)).child("TIPO"))
                .child(div().flex_none().w(px(62.)).child("FLAGS"))
                .child(div().w(px(16.)));
            cols = cols.child(col_header);
            for c in &t.columns {
                let chip = |id: String,
                            label: &'static str,
                            flag: &'static str,
                            on: bool,
                            cx: &mut Context<Self>| {
                    let (table, col) = (t.name.clone(), c.name.clone());
                    div()
                        .id(SharedString::from(id))
                        .role(Role::Switch)
                        .aria_label(format!("{label} em {}", c.name))
                        .tooltip(move |window, cx| {
                            Tooltip::new(match flag {
                                "pk" => "Chave primária",
                                "not null" => "Obrigatória (not null)",
                                _ => "Valor único",
                            })
                            .build(window, cx)
                        })
                        .px(px(4.))
                        .rounded(px(4.))
                        .border_1()
                        .text_size(px(10.))
                        .when(on, |d| {
                            d.bg(g.selected)
                                .border_color(g.accent.opacity(0.5))
                                .text_color(g.accent)
                        })
                        .when(!on, |d| {
                            d.border_color(g.edge_soft).text_color(g.text_faint)
                        })
                        .hover({
                            let e = g.edge;
                            move |st| st.border_color(e)
                        })
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.toggle_flag(table.clone(), col.clone(), flag, !on, window, cx)
                        }))
                        .child(label)
                };
                let name_el = field(
                    EditTarget::ColName(t.name.clone(), c.name.clone()),
                    c.name.clone(),
                    false,
                    cx,
                );
                let type_el = field(
                    EditTarget::ColType(t.name.clone(), c.name.clone()),
                    c.ty.clone(),
                    true,
                    cx,
                );
                let (table, col) = (t.name.clone(), c.name.clone());
                let is_editing_this_type = matches!(
                    &editing,
                    Some((EditTarget::ColType(tbl, col), _)) if tbl == &t.name && col == &c.name
                );
                cols = cols.child(
                    div()
                        .group("insp-col")
                        .flex()
                        .items_center()
                        .gap(px(6.))
                        .px(SP_MD)
                        .h(px(32.))
                        .border_b_1()
                        .border_color(g.edge_soft)
                        .text_size(TEXT_SM)
                        .child(
                            div()
                                .flex_1()
                                .min_w(px(60.))
                                .overflow_hidden()
                                .child(name_el),
                        )
                        .child(
                            div()
                                .w(px(4.))
                                .h(px(14.))
                                .mx(px(-2.))
                                .flex_none()
                                .cursor(CursorStyle::ResizeLeftRight)
                                .hover(|s| s.bg(g.accent.opacity(0.4)).rounded(px(1.)))
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(move |this, e: &MouseDownEvent, _, cx| {
                                        if e.click_count >= 2 {
                                            this.col_type_w = 110.0;
                                            this.save_prefs();
                                            cx.notify();
                                        } else {
                                            this.col_resize_start =
                                                Some((e.position.x, this.col_type_w));
                                        }
                                    }),
                                ),
                        )
                        .child(
                            div()
                                .flex_none()
                                .w(px(col_type_w))
                                .overflow_hidden()
                                .text_size(TEXT_XS)
                                .child(type_el),
                        )
                        .child(chip(format!("pk-{}", c.name), "PK", "pk", c.pk, cx))
                        .child(chip(
                            format!("nn-{}", c.name),
                            "NN",
                            "not null",
                            c.not_null,
                            cx,
                        ))
                        .child(chip(format!("uq-{}", c.name), "UQ", "unique", c.unique, cx))
                        .child(
                            div()
                                .invisible()
                                .group_hover("insp-col", |st| st.visible())
                                .child(
                                    Button::new(SharedString::from(format!("del-{}", c.name)))
                                        .ghost()
                                        .xsmall()
                                        .icon(IconName::Trash)
                                        .tooltip("Excluir coluna")
                                        .on_click(cx.listener(move |this, _, window, cx| {
                                            let (t2, c2) = (table.clone(), col.clone());
                                            this.edit_text(false, window, cx, |text| {
                                                dbml_edit::delete_column(text, &t2, &c2)
                                            });
                                        })),
                                ),
                        ),
                );

                if is_editing_this_type {
                    let query = if let Some((_, st)) = &editing {
                        st.read(cx).value().trim().to_ascii_lowercase()
                    } else {
                        String::new()
                    };

                    let doc_text = self
                        .doc()
                        .map(|d| d.editor.read(cx).value().to_string())
                        .unwrap_or_default();
                    let enums = extract_enums_from_dbml(&doc_text);

                    let mut suggestions: Vec<(&str, &str)> = Vec::new();
                    for &(ty, desc) in STANDARD_TYPES {
                        if query.is_empty() || ty.to_ascii_lowercase().contains(&query) {
                            suggestions.push((ty, desc));
                        }
                    }
                    for en in &enums {
                        if query.is_empty() || en.to_ascii_lowercase().contains(&query) {
                            suggestions.push((en.as_str(), "enumeração"));
                        }
                    }
                    suggestions.truncate(8);

                    if !suggestions.is_empty() {
                        let mut sugg_box = div()
                            .flex()
                            .flex_col()
                            .gap(px(2.))
                            .p(px(6.))
                            .mx(SP_MD)
                            .my(px(2.))
                            .rounded(px(6.))
                            .bg(g.island)
                            .border_1()
                            .border_color(g.edge)
                            .shadow_md();

                        sugg_box = sugg_box.child(
                            div()
                                .flex()
                                .justify_between()
                                .px(px(4.))
                                .pb(px(2.))
                                .border_b_1()
                                .border_color(g.edge_soft)
                                .text_size(px(10.))
                                .text_color(g.text_faint)
                                .child("Sugestões de tipo (clique para aplicar)")
                                .child("Esc fecha"),
                        );

                        for (s_type, s_desc) in suggestions {
                            let opt_type = s_type.to_string();
                            let (tbl, cl) = (t.name.clone(), c.name.clone());
                            sugg_box = sugg_box.child(
                                div()
                                    .id(SharedString::from(format!(
                                        "type-opt-{}-{}-{}",
                                        tbl, cl, opt_type
                                    )))
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .px(px(6.))
                                    .py(px(3.))
                                    .rounded(px(4.))
                                    .cursor_pointer()
                                    .hover(|s| s.bg(g.hover))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(move |this, _, window, cx| {
                                            this.apply_column_type(
                                                tbl.clone(),
                                                cl.clone(),
                                                opt_type.clone(),
                                                window,
                                                cx,
                                            );
                                        }),
                                    )
                                    .child(
                                        div()
                                            .text_size(TEXT_XS)
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(g.text)
                                            .child(s_type.to_string()),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(10.))
                                            .text_color(g.text_faint)
                                            .child(s_desc.to_string()),
                                    ),
                            );
                        }
                        cols = cols.child(sugg_box);
                    }
                }
            }
            let add_table = t.name.clone();
            cols = cols
                .child(
                    div().px(SP_MD).py(SP_SM).child(
                        Button::new("insp-add-col")
                            .ghost()
                            .small()
                            .icon(IconName::Plus)
                            .label("Adicionar coluna")
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.add_column_to(add_table.clone(), window, cx)
                            })),
                    ),
                )
                .on_mouse_move(cx.listener(|this, e: &MouseMoveEvent, _, cx| {
                    if let Some((start_x, start_w)) = this.col_resize_start {
                        let delta = f32::from(e.position.x - start_x);
                        let new_w = (start_w - delta).clamp(60.0, 360.0);
                        if (new_w - this.col_type_w).abs() >= 0.5 {
                            this.col_type_w = new_w;
                            cx.notify();
                        }
                    }
                }))
                .on_mouse_up(
                    MouseButton::Left,
                    cx.listener(|this, _, _, cx| {
                        if this.col_resize_start.take().is_some() {
                            this.save_prefs();
                            cx.notify();
                        }
                    }),
                );
            let title_el = field(EditTarget::Table(t.name.clone()), t.name.clone(), false, cx);
            let mut rels = div().flex().flex_col().gap(px(4.)).px(SP_MD);
            for r in refs {
                let card = match r.kind {
                    schema::RefKind::ManyToOne => "N:1",
                    schema::RefKind::OneToMany => "1:N",
                    schema::RefKind::OneToOne => "1:1",
                    schema::RefKind::ManyToMany => "N:N",
                };
                rels = rels.child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.))
                        .text_size(TEXT_SM)
                        .text_color(g.text_muted)
                        .child(Icon::new(IconName::Link2).xsmall().text_color(g.text_faint))
                        .child(div().flex_1().truncate().child(format!(
                            "{}.{} → {}.{}",
                            r.from.table, r.from.column, r.to.table, r.to.column
                        )))
                        .child(tag(card)),
                );
            }
            // cor: amostras clicáveis (grava headercolor no DBML)
            let current = t.color.clone().unwrap_or_default().to_ascii_lowercase();
            let mut swatches = div().flex().flex_wrap().gap(px(6.)).px(SP_MD);
            for (i, (label, hexv)) in TABLE_COLORS.iter().enumerate() {
                let on = current == *hexv;
                let table = t.name.clone();
                let color = hex(hexv).unwrap_or_default();
                swatches = swatches.child(
                    div()
                        .id(("swatch", i))
                        .role(Role::Button)
                        .aria_label(format!("Cor {label}"))
                        .tooltip(move |window, cx| Tooltip::new(*label).build(window, cx))
                        .size(px(22.))
                        .rounded_full()
                        .bg(color)
                        .border_2()
                        .border_color(if on {
                            g.text
                        } else {
                            gpui_kit::transparent_black()
                        })
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.set_color(table.clone(), Some(hexv), window, cx)
                        })),
                );
            }
            let table = t.name.clone();
            swatches = swatches.child(
                div()
                    .id("swatch-none")
                    .role(Role::Button)
                    .aria_label("Sem cor")
                    .tooltip(|window, cx| Tooltip::new("Sem cor").build(window, cx))
                    .size(px(22.))
                    .rounded_full()
                    .border_1()
                    .border_color(g.edge)
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(Icon::new(IconName::X).xsmall().text_color(g.text_faint))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.set_color(table.clone(), None, window, cx)
                    })),
            );
            let group_row = match group {
                Some(gr) => div()
                    .flex()
                    .items_center()
                    .gap(SP_SM)
                    .px(SP_MD)
                    .child(
                        Icon::new(IconName::LayoutGrid)
                            .xsmall()
                            .text_color(g.text_faint),
                    )
                    .child(
                        div()
                            .flex_1()
                            .text_size(TEXT_SM)
                            .text_color(g.text)
                            .child(gr.name.clone()),
                    )
                    .child(
                        Button::new("insp-ungroup")
                            .ghost()
                            .xsmall()
                            .label("Desagrupar")
                            .tooltip_with_action(
                                "Tirar a tabela do grupo",
                                &UngroupSelection,
                                Some(actions::CTX),
                            )
                            .on_click(|_, window, cx| {
                                window.dispatch_action(Box::new(UngroupSelection), cx)
                            }),
                    ),
                None => div()
                    .px(SP_MD)
                    .text_size(TEXT_SM)
                    .text_color(g.text_faint)
                    .child("Sem grupo · Shift+clique em outras tabelas e Ctrl+G para agrupar"),
            };
            div()
                .flex()
                .flex_col()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(2.))
                        .p(SP_MD)
                        .border_b_1()
                        .border_color(g.edge)
                        .child(
                            div()
                                .text_size(TEXT_XS)
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(g.text_faint)
                                .child("TABELA"),
                        )
                        .child(
                            div()
                                .text_size(TEXT_LG)
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(title_el),
                        )
                        .child(
                            div()
                                .text_size(TEXT_SM)
                                .text_color(g.text_muted)
                                .child(format!("{} colunas · F2 renomeia", t.columns.len())),
                        ),
                )
                .child(Self::section(g, "COR", None))
                .child(swatches)
                .child(Self::section(g, "GRUPO", None))
                .child(group_row)
                .child(Self::section(g, "COLUNAS", None))
                .child(cols)
                .child(Self::section(g, "RELAÇÕES", None))
                .child(rels)
                .child(
                    div()
                        .p(SP_MD)
                        .mt(SP_SM)
                        .border_t_1()
                        .border_color(g.edge_soft)
                        .child(
                            Button::new("insp-del-table")
                                .ghost()
                                .small()
                                .icon(IconName::Trash)
                                .label("Excluir tabela")
                                .tooltip_with_action(
                                    "Excluir tabela",
                                    &DeleteSelection,
                                    Some(actions::CANVAS),
                                )
                                .on_click(|_, window, cx| {
                                    window.dispatch_action(Box::new(DeleteSelection), cx)
                                }),
                        ),
                )
        } else {
            let tip = |action: &dyn Action, ctx: &str, text: &'static str| {
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .h(px(26.))
                    .text_size(TEXT_SM)
                    .text_color(g.text_muted)
                    .child(text)
                    .children(Kbd::binding_for_action(action, Some(ctx), window))
            };
            div()
                .flex()
                .flex_col()
                .gap(px(6.))
                .p(SP_LG)
                .child(
                    div()
                        .flex()
                        .justify_between()
                        .items_center()
                        .child(
                            div()
                                .text_size(TEXT_BASE)
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(g.text)
                                .child("Nada selecionado"),
                        )
                        .child(
                            Button::new("btn-hide-inspector")
                                .ghost()
                                .xsmall()
                                .icon(IconName::PanelRightClose)
                                .tooltip("Ocultar inspector (Ctrl+I)")
                                .on_click(|_, window, cx| {
                                    window.dispatch_action(Box::new(ToggleInspector), cx)
                                }),
                        ),
                )
                .child(
                    div()
                        .text_size(TEXT_SM)
                        .text_color(g.text_muted)
                        .child("Clique numa tabela para ver colunas, cor, grupo e relações. Shift+clique seleciona várias."),
                )
                .child(div().h(px(1.)).my(SP_SM).bg(g.edge))
                .child(tip(&ToolSelect, actions::CANVAS, "Selecionar"))
                .child(tip(&ToolHand, actions::CANVAS, "Mover a vista"))
                .child(tip(&FitView, actions::CANVAS, "Ajustar à tela"))
                .child(tip(&GroupSelection, actions::CTX, "Agrupar seleção"))
                .child(tip(&ToggleEditor, actions::CTX, "Editor DBML"))
                .child(tip(&OpenPalette, actions::CTX, "Todos os comandos"))
        };
        div()
            .id("inspector")
            .role(Role::Region)
            .aria_label("Inspector")
            .size_full()
            .bg(g.sidebar)
            .border_l_1()
            .border_color(g.edge)
            .overflow_y_scroll()
            .child(body)
    }

    fn editor_panel(&self, g: &Glass) -> Div {
        let Some(doc) = self.doc() else { return div() };
        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(g.surface)
            .border_r_1()
            .border_color(g.edge)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .h(px(32.))
                    .px(SP_MD)
                    .border_b_1()
                    .border_color(g.edge)
                    .text_size(TEXT_XS)
                    .text_color(g.text_muted)
                    .child(Icon::new(IconName::Braces).xsmall())
                    .child(div().flex_1().child(doc.title.clone()))
                    .child(
                        Button::new("editor-format")
                            .ghost()
                            .xsmall()
                            .icon(IconName::Sparkles)
                            .tooltip_with_action(
                                "Formatar documento",
                                &FormatDocument,
                                Some(actions::CTX),
                            )
                            .on_click(|_, window, cx| {
                                window.dispatch_action(Box::new(FormatDocument), cx)
                            }),
                    )
                    .child(
                        Button::new("editor-close")
                            .ghost()
                            .xsmall()
                            .icon(IconName::X)
                            .tooltip_with_action("Fechar editor", &ToggleEditor, Some(actions::CTX))
                            .on_click(|_, window, cx| {
                                window.dispatch_action(Box::new(ToggleEditor), cx)
                            }),
                    ),
            )
            .child(
                div().flex_1().min_h_0().child(
                    Editor::new(&doc.editor)
                        .appearance(false)
                        .bordered(false)
                        .h_full(),
                ),
            )
    }

    fn empty_state(&self, g: &Glass) -> Div {
        div()
            .size_full()
            .bg(g.surface)
            .flex()
            .items_center()
            .justify_center()
            .child(crate::widgets::empty_state(
                g,
                IconName::Database,
                "Nenhum arquivo aberto",
                "Abra uma pasta de projeto ou comece um esquema novo.",
                crate::widgets::EmptyAction {
                    id: "empty-folder",
                    icon: IconName::FolderOpen,
                    label: "Abrir pasta",
                    keys: "Ctrl+Shift+O",
                    action: Box::new(OpenFolder),
                },
                crate::widgets::EmptyAction {
                    id: "empty-new",
                    icon: IconName::Plus,
                    label: "Novo arquivo",
                    keys: "Ctrl+N",
                    action: Box::new(NewFile),
                },
            ))
    }

    fn statusbar(&self, g: &Glass, compact: bool, cx: &App) -> Div {
        let mut bar = div()
            .h(STATUSBAR_H)
            .w_full()
            .flex_none()
            .flex()
            .items_center()
            .gap(SP_LG)
            .px(SP_MD)
            .bg(g.chrome)
            .border_t_1()
            .border_color(g.edge)
            .text_size(TEXT_XS)
            .text_color(g.text_muted);
        if let Some(doc) = self.doc() {
            let d = doc.diagram.read(cx);
            let s = d.schema();
            let (e, w) = doc.counts();
            bar = bar
                .child(
                    div()
                        .id("status-problems")
                        .flex()
                        .items_center()
                        .gap(px(5.))
                        .on_click(|_, window, cx| {
                            window.dispatch_action(Box::new(ShowProblems), cx)
                        })
                        .child(Icon::new(IconName::CircleX).xsmall().text_color(if e > 0 {
                            g.danger
                        } else {
                            g.text_faint
                        }))
                        .child(e.to_string())
                        .child(
                            Icon::new(IconName::TriangleAlert)
                                .xsmall()
                                .text_color(if w > 0 { g.warning } else { g.text_faint }),
                        )
                        .child(w.to_string()),
                )
                .child(format!(
                    "{} tabelas · {} relações",
                    s.tables.len(),
                    s.refs.len()
                ))
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .justify_center()
                        .text_color(g.text_faint)
                        .truncate()
                        .when(!compact, |e| e.child(d.hint())),
                )
                .child(format!("Zoom {}%", (d.zoom() * 100.).round() as i32));
        } else {
            bar = bar.child(div().flex_1());
        }
        bar.child(format!("v{}", env!("CARGO_PKG_VERSION")))
            .child("DBML")
            .child(if self.mode == Mode::Dark {
                "Escuro"
            } else {
                "Claro"
            })
    }
}

/// Ações disparadas de camadas fora do shell (paleta de comandos, menus, diálogos)
/// não passam pelos handlers do elemento; estes handlers globais as encaminham ao shell.
macro_rules! forward_actions {
    ($cx:ident, $win:ident, $shell:ident; $($a:ident => $m:ident),* $(,)?) => {
        $({
            let shell = $shell.clone();
            $cx.on_action(move |a: &$a, cx| {
                let shell = shell.clone();
                $win.update(cx, |_, window, cx| shell.update(cx, |s, cx| s.$m(a, window, cx))).ok();
            });
        })*
    };
}

pub fn register_global_actions(win: AnyWindowHandle, shell: Entity<Shell>, cx: &mut App) {
    forward_actions!(cx, win, shell;
        NewFile => new_file,
        OpenFile => open_file,
        OpenFolder => open_folder,
        SaveFile => save_file,
        CloseTab => close_tab,
        NextTab => next_tab,
        PrevTab => prev_tab,
        ShowFiles => show_files,
        ShowSchema => show_schema,
        ShowProblems => show_problems,
        Quit => quit,
        FormatDocument => format_document,
        MoveLineUp => move_line_up,
        MoveLineDown => move_line_down,
        OpenPalette => open_palette,
        ShowShortcuts => show_shortcuts,
        ToggleTheme => toggle_theme,
        ToggleCardTransparency => toggle_card_transparency,
        ToggleEditor => toggle_editor,
        ToggleSidebar => toggle_sidebar,
        ToggleInspector => toggle_inspector,
        ToggleLegend => toggle_legend,
        FitView => fit,
        Relayout => relayout,
        ZoomIn => zoom_in,
        ZoomOut => zoom_out,
        ToolSelect => tool_select,
        ToolHand => tool_hand,
        Deselect => deselect,
        GroupSelection => group_selection,
        NewTable => new_table,
        DeleteSelection => delete_selection,
        RenameSelected => rename_selected,
        ResetPanels => reset_panels,
        UngroupSelection => ungroup_selection,
        ZoomInUi => zoom_in_ui,
        ZoomOutUi => zoom_out_ui,
        ResetUiZoom => reset_ui_zoom,
        ToggleAutoCollapseInspector => toggle_auto_collapse_inspector,
    );
}

impl Render for Shell {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let g = Glass::of_scaled(self.mode, self.translucent, self.ui_scale);
        // responsivo: painéis colapsam apenas em larguras extremamente reduzidas
        let width = f32::from(window.viewport_size().width);
        let has_selection = self.doc().is_some_and(|d| {
            let diag = d.diagram.read(cx);
            diag.selected().is_some() || !diag.selection().is_empty()
        });
        let show_sidebar = self.sidebar_open && !self.editor_open && width >= 500.;
        let show_editor = self.editor_open && width >= 500.;
        let show_inspector =
            self.inspector_open && (!self.inspector_auto_hide || has_selection) && width >= 600.;
        let titlebar = self.titlebar(&g, window.is_maximized(), width < 1000., cx);
        let rail = self.rail(&g);
        let right_rail = self.right_rail(&g, cx);
        // o canvas primeiro: ele aplica o ajuste à tela que a barra de status exibe
        let translucent_cards = self.translucent_cards;
        let canvas = self.doc().map(|d| d.diagram.clone()).inspect(|e| {
            e.update(cx, |d, cx| {
                if d.glass.map(|x| x.mode) != Some(g.mode)
                    || d.translucent_cards != translucent_cards
                {
                    d.glass = Some(g);
                    d.translucent_cards = translucent_cards;
                    cx.notify();
                }
            });
        });
        let sidebar = self.sidebar(&g, cx);
        let inspector = self.inspector(&g, window, cx);
        let editor = self.editor_panel(&g);
        let statusbar = self.statusbar(&g, width < 1000., cx);
        let work: AnyElement = match canvas {
            Some(canvas) => {
                let panels_id = SharedString::from(format!(
                    "main-panels-{}-{}-{}-{}",
                    self.panels_gen, show_sidebar, show_editor, show_inspector
                ));
                let mut resizable = h_resizable(panels_id);

                let has_sb = show_sidebar;
                let has_ed = show_editor;
                let has_insp = show_inspector;
                let weak = cx.entity().downgrade();
                resizable = resizable.on_resize(move |state, _, cx| {
                    let sizes = state.read(cx).sizes().clone();
                    weak.update(cx, |s, _| {
                        let mut ix = 0;
                        if has_sb && sizes.len() > ix {
                            let w = f32::from(sizes[ix]);
                            if w >= 120. {
                                s.widths.0 = w;
                            }
                            ix += 1;
                        }
                        if has_ed && sizes.len() > ix {
                            let w = f32::from(sizes[ix]);
                            if w >= 120. {
                                s.widths.1 = w;
                            }
                            ix += 1;
                        }
                        // ix é o canvas
                        ix += 1;
                        if has_insp && sizes.len() > ix {
                            let w = f32::from(sizes[ix]);
                            if w >= 120. {
                                s.widths.2 = w;
                            }
                        }
                        s.save_prefs();
                    })
                    .ok();
                });

                if show_sidebar {
                    resizable = resizable.child(
                        resizable_panel()
                            .size(px(self.widths.0))
                            .size_range(px(180.)..px(440.))
                            .child(sidebar),
                    );
                }

                if show_editor {
                    resizable = resizable.child(
                        resizable_panel()
                            .size(px(self.widths.1))
                            .size_range(px(280.)..px(900.))
                            .child(editor),
                    );
                }

                resizable = resizable.child(
                    resizable_panel()
                        .child(div().id("canvas").size_full().bg(g.surface).child(canvas)),
                );

                if show_inspector {
                    resizable = resizable.child(
                        resizable_panel()
                            .size(px(self.widths.2))
                            .size_range(px(240.)..px(800.))
                            .child(inspector),
                    );
                }

                resizable.into_any_element()
            }
            None => div()
                .size_full()
                .flex()
                .when(show_sidebar, |d| {
                    d.child(
                        div()
                            .w(px(self.widths.0))
                            .h_full()
                            .border_r_1()
                            .border_color(g.edge)
                            .child(sidebar),
                    )
                })
                .when(show_editor, |d| {
                    d.child(
                        div()
                            .w(px(self.widths.1))
                            .h_full()
                            .border_r_1()
                            .border_color(g.edge)
                            .child(editor),
                    )
                })
                .child(self.empty_state(&g))
                .into_any_element(),
        };
        div()
            .id("shell")
            .key_context(actions::CTX)
            .track_focus(&self.focus)
            .on_action(cx.listener(Self::new_file))
            .on_action(cx.listener(Self::open_file))
            .on_action(cx.listener(Self::open_folder))
            .on_action(cx.listener(Self::save_file))
            .on_action(cx.listener(Self::close_tab))
            .on_action(cx.listener(Self::next_tab))
            .on_action(cx.listener(Self::prev_tab))
            .on_action(cx.listener(Self::show_files))
            .on_action(cx.listener(Self::show_schema))
            .on_action(cx.listener(Self::show_problems))
            .on_action(cx.listener(Self::quit))
            .on_action(cx.listener(Self::format_document))
            .on_action(cx.listener(Self::move_line_up))
            .on_action(cx.listener(Self::move_line_down))
            .on_action(cx.listener(Self::open_palette))
            .on_action(cx.listener(Self::show_shortcuts))
            .on_action(cx.listener(Self::toggle_theme))
            .on_action(cx.listener(Self::toggle_card_transparency))
            .on_action(cx.listener(Self::toggle_editor))
            .on_action(cx.listener(Self::toggle_sidebar))
            .on_action(cx.listener(Self::toggle_inspector))
            .on_action(cx.listener(Self::toggle_legend))
            .on_action(cx.listener(Self::fit))
            .on_action(cx.listener(Self::relayout))
            .on_action(cx.listener(Self::zoom_in))
            .on_action(cx.listener(Self::zoom_out))
            .on_action(cx.listener(Self::tool_select))
            .on_action(cx.listener(Self::tool_hand))
            .on_action(cx.listener(Self::deselect))
            .on_action(cx.listener(Self::new_table))
            .on_action(cx.listener(Self::delete_selection))
            .on_action(cx.listener(Self::rename_selected))
            .on_action(cx.listener(Self::reset_panels))
            .on_action(cx.listener(Self::group_selection))
            .on_action(cx.listener(Self::ungroup_selection))
            .on_action(cx.listener(Self::zoom_in_ui))
            .on_action(cx.listener(Self::zoom_out_ui))
            .on_action(cx.listener(Self::reset_ui_zoom))
            .on_action(cx.listener(Self::toggle_auto_collapse_inspector))
            .size_full()
            .font_family(FONT)
            .text_color(g.text)
            .bg(g.chrome)
            .flex()
            .flex_col()
            .child(titlebar)
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .child(rail)
                    .child(div().flex_1().min_w_0().h_full().child(work))
                    .child(right_rail),
            )
            .child(statusbar)
    }
}
