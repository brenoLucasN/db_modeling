/**
 * Catálogo unificado de ações e atalhos de teclado.
 * Portado 1:1 de `crates/app/src/actions.rs`.
 */

export type ActionId =
  | 'NewFile'
  | 'ShowFiles'
  | 'ShowSchema'
  | 'ShowProblems'
  | 'ToggleTheme'
  | 'ToggleEditor'
  | 'ToggleSidebar'
  | 'ToggleInspector'
  | 'ToggleLegend'
  | 'FitView'
  | 'Relayout'
  | 'ZoomIn'
  | 'ZoomOut'
  | 'ToolSelect'
  | 'ToolHand'
  | 'Deselect'
  | 'ToggleCardTransparency'
  | 'GroupSelection'
  | 'UngroupSelection'
  | 'NewTable'
  | 'DeleteSelection'
  | 'RenameSelected'
  | 'FormatDocument'
  | 'MoveLineUp'
  | 'MoveLineDown'
  | 'ZoomInUi'
  | 'ZoomOutUi'
  | 'ResetUiZoom'
  | 'ToggleAutoCollapseInspector'
  | 'OpenPalette'
  | 'ShowShortcuts'
  | 'ExportSvg'
  | 'ExportPng'
  | 'ExportJpeg'
  | 'ExportMermaid';

export type KeyContext = 'Shell' | 'Canvas';

export interface ActionEntry {
  id: ActionId;
  key: string;
  displayKeys: string[];
  group: string;
  label: string;
  context: KeyContext;
}

export function catalog(): ActionEntry[] {
  const e = (
    key: string,
    displayKeys: string[],
    id: ActionId,
    group: string,
    label: string
  ): ActionEntry => {
    const context: KeyContext =
      key.startsWith('ctrl') || key.startsWith('alt') ? 'Shell' : 'Canvas';
    return { id, key, displayKeys, group, label, context };
  };

  return [
    e('ctrl-n', ['Ctrl', 'N'], 'NewFile', 'Arquivo', 'Novo arquivo .dbml'),
    e('ctrl-shift-e', ['Ctrl', 'Shift', 'E'], 'ShowFiles', 'Exibir', 'Mostrar arquivos do projeto'),
    e('ctrl-shift-d', ['Ctrl', 'Shift', 'D'], 'ShowSchema', 'Exibir', 'Mostrar esquema (tabelas)'),
    e('ctrl-shift-m', ['Ctrl', 'Shift', 'M'], 'ShowProblems', 'Exibir', 'Mostrar problemas'),
    e('ctrl-t', ['Ctrl', 'T'], 'ToggleTheme', 'Exibir', 'Alternar tema claro/escuro'),
    e('ctrl-e', ['Ctrl', 'E'], 'ToggleEditor', 'Exibir', 'Mostrar/ocultar editor DBML'),
    e('ctrl-b', ['Ctrl', 'B'], 'ToggleSidebar', 'Exibir', 'Mostrar/ocultar barra lateral'),
    e('ctrl-i', ['Ctrl', 'I'], 'ToggleInspector', 'Exibir', 'Mostrar/ocultar inspector'),
    e('ctrl-shift-l', ['Ctrl', 'Shift', 'L'], 'ToggleLegend', 'Exibir', 'Mostrar/ocultar legenda'),
    e('v', ['V'], 'ToolSelect', 'Canvas', 'Ferramenta Selecionar'),
    e('h', ['H'], 'ToolHand', 'Canvas', 'Ferramenta Mão (mover a vista)'),
    e('shift-1', ['Shift', '1'], 'FitView', 'Canvas', 'Ajustar à tela'),
    e('ctrl-l', ['Ctrl', 'L'], 'Relayout', 'Canvas', 'Reorganizar automaticamente'),
    e('ctrl-=', ['Ctrl', '='], 'ZoomIn', 'Canvas', 'Ampliar'),
    e('ctrl--', ['Ctrl', '-'], 'ZoomOut', 'Canvas', 'Reduzir'),
    e('escape', ['Esc'], 'Deselect', 'Canvas', 'Limpar seleção'),
    e('t', ['T'], 'NewTable', 'Canvas', 'Nova tabela'),
    e('delete', ['Del'], 'DeleteSelection', 'Canvas', 'Excluir tabelas selecionadas'),
    e('f2', ['F2'], 'RenameSelected', 'Canvas', 'Renomear tabela'),
    e('ctrl-g', ['Ctrl', 'G'], 'GroupSelection', 'Canvas', 'Agrupar tabelas selecionadas'),
    e('ctrl-shift-g', ['Ctrl', 'Shift', 'G'], 'UngroupSelection', 'Canvas', 'Desagrupar tabela'),
    e('', [], 'ToggleCardTransparency', 'Aparência', 'Cartões translúcidos'),
    e(
      'ctrl-shift-=',
      ['Ctrl', 'Shift', '='],
      'ZoomInUi',
      'Aparência',
      'Aumentar tamanho da interface (fontes e ícones)'
    ),
    e(
      'ctrl-shift--',
      ['Ctrl', 'Shift', '-'],
      'ZoomOutUi',
      'Aparência',
      'Diminuir tamanho da interface (fontes e ícones)'
    ),
    e('ctrl-shift-0', ['Ctrl', 'Shift', '0'], 'ResetUiZoom', 'Aparência', 'Tamanho padrão da interface'),
    e(
      'ctrl-shift-i',
      ['Ctrl', 'Shift', 'I'],
      'ToggleAutoCollapseInspector',
      'Exibir',
      'Recolher inspector quando nada selecionado'
    ),
    e('alt-shift-f', ['Alt', 'Shift', 'F'], 'FormatDocument', 'Editor', 'Formatar documento'),
    e('alt-up', ['Alt', '↑'], 'MoveLineUp', 'Editor', 'Mover linha para cima'),
    e('alt-down', ['Alt', '↓'], 'MoveLineDown', 'Editor', 'Mover linha para baixo'),
    e('', [], 'ExportSvg', 'Exportar', 'Exportar diagrama como SVG vetorial'),
    e('', [], 'ExportPng', 'Exportar', 'Exportar diagrama como imagem PNG'),
    e('', [], 'ExportJpeg', 'Exportar', 'Exportar diagrama como imagem JPEG'),
    e('', [], 'ExportMermaid', 'Exportar', 'Exportar diagrama como Mermaid erDiagram'),
    e('ctrl-k', ['Ctrl', 'K'], 'OpenPalette', 'Ajuda', 'Paleta de comandos'),
    e('ctrl-/', ['Ctrl', '/'], 'ShowShortcuts', 'Ajuda', 'Atalhos de teclado'),
  ];
}
