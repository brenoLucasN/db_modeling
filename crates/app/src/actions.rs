//! Ações do app. Toda ação tem atalho e aparece na paleta de comandos e nos menus.

use gpui_kit::*;

actions!(
    app,
    [
        NewFile,
        OpenFile,
        OpenFolder,
        CloseTab,
        NextTab,
        PrevTab,
        ShowFiles,
        ShowSchema,
        ShowProblems,
        SaveFile,
        Quit,
        OpenPalette,
        ShowShortcuts,
        ToggleTheme,
        ToggleEditor,
        ToggleSidebar,
        ToggleInspector,
        ToggleLegend,
        FitView,
        Relayout,
        ZoomIn,
        ZoomOut,
        ToolSelect,
        ToolHand,
        Deselect,
        ToggleCardTransparency,
        GroupSelection,
        NewTable,
        DeleteSelection,
        RenameSelected,
        ResetPanels,
        UngroupSelection,
        FormatDocument,
        MoveLineUp,
        MoveLineDown,
        ZoomInUi,
        ZoomOutUi,
        ResetUiZoom,
        ToggleAutoCollapseInspector,
    ]
);

/// Contexto de teclado do shell (atalhos com modificador valem na janela toda).
pub const CTX: &str = "Shell";
/// Contexto do canvas focado: atalhos de uma tecla só valem aqui, para não
/// disparar enquanto se digita no editor.
pub const CANVAS: &str = "Canvas";

pub struct Entry {
    pub action: Box<dyn Action>,
    pub group: &'static str,
    pub label: &'static str,
    pub context: &'static str,
    pub binding: Option<KeyBinding>,
}

/// Fonte única para keybindings, paleta de comandos, menus e diálogo de atalhos.
pub fn catalog() -> Vec<Entry> {
    fn e<A: Action + Clone>(
        key: &'static str,
        action: A,
        group: &'static str,
        label: &'static str,
    ) -> Entry {
        // atalhos sem Ctrl só valem com o canvas focado (não disparam ao digitar no editor)
        let context = if key.starts_with("ctrl") || key.starts_with("alt") {
            CTX
        } else {
            CANVAS
        };
        Entry {
            binding: (!key.is_empty()).then(|| KeyBinding::new(key, action.clone(), Some(context))),
            action: Box::new(action),
            group,
            label,
            context,
        }
    }
    vec![
        e("ctrl-n", NewFile, "Arquivo", "Novo arquivo"),
        e("ctrl-o", OpenFile, "Arquivo", "Abrir arquivo .dbml…"),
        e(
            "ctrl-shift-o",
            OpenFolder,
            "Arquivo",
            "Abrir pasta de projeto…",
        ),
        e("ctrl-w", CloseTab, "Arquivo", "Fechar aba"),
        e("ctrl-tab", NextTab, "Exibir", "Próxima aba"),
        e("ctrl-shift-tab", PrevTab, "Exibir", "Aba anterior"),
        e(
            "ctrl-shift-e",
            ShowFiles,
            "Exibir",
            "Mostrar arquivos do projeto",
        ),
        e(
            "ctrl-shift-d",
            ShowSchema,
            "Exibir",
            "Mostrar esquema (tabelas)",
        ),
        e("ctrl-shift-m", ShowProblems, "Exibir", "Mostrar problemas"),
        e("ctrl-s", SaveFile, "Arquivo", "Salvar"),
        e("ctrl-q", Quit, "Arquivo", "Sair"),
        e(
            "ctrl-t",
            ToggleTheme,
            "Exibir",
            "Alternar tema claro/escuro",
        ),
        e(
            "ctrl-e",
            ToggleEditor,
            "Exibir",
            "Mostrar/ocultar editor DBML",
        ),
        e(
            "ctrl-b",
            ToggleSidebar,
            "Exibir",
            "Mostrar/ocultar barra lateral",
        ),
        e(
            "ctrl-i",
            ToggleInspector,
            "Exibir",
            "Mostrar/ocultar inspector",
        ),
        e(
            "ctrl-shift-l",
            ToggleLegend,
            "Exibir",
            "Mostrar/ocultar legenda",
        ),
        e("v", ToolSelect, "Canvas", "Ferramenta Selecionar"),
        e("h", ToolHand, "Canvas", "Ferramenta Mão (mover a vista)"),
        e("shift-1", FitView, "Canvas", "Ajustar à tela"),
        e("ctrl-l", Relayout, "Canvas", "Reorganizar automaticamente"),
        e("ctrl-=", ZoomIn, "Canvas", "Ampliar"),
        e("ctrl--", ZoomOut, "Canvas", "Reduzir"),
        e("escape", Deselect, "Canvas", "Limpar seleção"),
        e("t", NewTable, "Canvas", "Nova tabela"),
        e(
            "delete",
            DeleteSelection,
            "Canvas",
            "Excluir tabelas selecionadas",
        ),
        e("f2", RenameSelected, "Canvas", "Renomear tabela"),
        e(
            "ctrl-g",
            GroupSelection,
            "Canvas",
            "Agrupar tabelas selecionadas",
        ),
        e(
            "ctrl-shift-g",
            UngroupSelection,
            "Canvas",
            "Desagrupar tabela",
        ),
        e(
            "",
            ToggleCardTransparency,
            "Aparência",
            "Cartões translúcidos",
        ),
        e(
            "ctrl-shift-=",
            ZoomInUi,
            "Aparência",
            "Aumentar tamanho da interface (fontes e ícones)",
        ),
        e(
            "ctrl-shift--",
            ZoomOutUi,
            "Aparência",
            "Diminuir tamanho da interface (fontes e ícones)",
        ),
        e(
            "ctrl-shift-0",
            ResetUiZoom,
            "Aparência",
            "Tamanho padrão da interface",
        ),
        e(
            "ctrl-shift-i",
            ToggleAutoCollapseInspector,
            "Exibir",
            "Recolher inspector quando nada selecionado",
        ),
        e(
            "alt-shift-f",
            FormatDocument,
            "Editor",
            "Formatar documento",
        ),
        e("alt-up", MoveLineUp, "Editor", "Mover linha para cima"),
        e("alt-down", MoveLineDown, "Editor", "Mover linha para baixo"),
        e("ctrl-k", OpenPalette, "Ajuda", "Paleta de comandos"),
        e("ctrl-/", ShowShortcuts, "Ajuda", "Atalhos de teclado"),
    ]
}
