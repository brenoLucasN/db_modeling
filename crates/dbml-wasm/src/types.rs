//! Tipos serializáveis de entrada e saída para a API WebAssembly.

use crate::line_index::Range;
use schema::Schema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Severidade de um diagnóstico emitida para o editor / webview.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
}

/// Diagnóstico de sintaxe ou semântica com posição exata compatível com LSP e UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WasmDiagnostic {
    pub range: Range,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub line: usize,
    pub byte_offset: usize,
}

/// Resultado retornado por `parse_and_analyze`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub schema: Schema,
    pub diagnostics: Vec<WasmDiagnostic>,
    pub enums: Vec<String>,
}

/// Patch textual mínimo resultante de uma operação semântica + metadados de UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextPatch {
    /// Intervalo de linha/coluna no VS Code a ser substituído
    pub range: Range,
    /// Offset de byte inicial (útil para substituição direta em buffer)
    pub byte_start: usize,
    /// Offset de byte final
    pub byte_end: usize,
    /// Novo texto que substitui o intervalo
    pub new_text: String,
    /// Texto completo atualizado (para sincronia imediata no cliente sem eco)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_source: Option<String>,
    /// Nome da tabela recém-criada (`place_new` e `start_edit`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_table: Option<String>,
    /// `(tabela, coluna)` recém-criada (`start_edit`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_column: Option<(String, String)>,
    /// `(antigo, novo)` quando uma tabela é renomeada (`d.rename`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub renamed_table: Option<(String, String)>,
    /// Se a operação exige relayout imediato (ex.: agrupar/desagrupar)
    #[serde(default)]
    pub should_relayout: bool,
}

/// Comandos semânticos emitidos pelo Canvas ou Inspector.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", content = "payload", rename_all = "snake_case")]
pub enum Command {
    AddTable {
        name: Option<String>,
    },
    DeleteTable {
        name: String,
    },
    DeleteTables {
        names: Vec<String>,
    },
    RenameTable {
        old_name: String,
        new_name: String,
    },
    AddColumn {
        table: String,
        name: Option<String>,
        ty: Option<String>,
    },
    DeleteColumn {
        table: String,
        column: String,
    },
    RenameColumn {
        table: String,
        old_name: String,
        new_name: String,
    },
    SetColumnType {
        table: String,
        column: String,
        ty: String,
    },
    SetColumnFlag {
        table: String,
        column: String,
        flag: String,
        value: bool,
    },
    AddRef {
        from_table: String,
        from_col: String,
        to_table: String,
        to_col: String,
    },
    GroupTables {
        name: Option<String>,
        tables: Vec<String>,
    },
    UngroupTable {
        table: String,
    },
    UngroupTables {
        tables: Vec<String>,
    },
    SetTableColor {
        table: String,
        color: Option<String>,
    },
}

/// Dimensão de um nó de tabela no canvas para cálculo de auto-layout.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableDimension {
    pub name: String,
    pub width: f32,
    pub height: f32,
}

/// Parâmetros de entrada opcionais para cálculo de layout.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LayoutParams {
    #[serde(default)]
    pub tables: Vec<TableDimension>,
    pub gap_x: Option<f32>,
    pub gap_y: Option<f32>,
    pub margin: Option<f32>,
}

/// Coordenada calculada de um nó.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NodeCoordinate {
    pub x: f32,
    pub y: f32,
}

/// Saída do algoritmo de auto-layout.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutResult {
    pub positions: HashMap<String, NodeCoordinate>,
}

/// Aresta roteada ortogonalmente em coordenadas de mundo (`z = 1.0`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutedEdge {
    pub from_table: String,
    pub from_col: String,
    pub to_table: String,
    pub to_col: String,
    pub ax: f32,
    pub ay: f32,
    pub bx: f32,
    pub by: f32,
    pub sa: f32,
    pub sb: f32,
    pub points: Vec<NodeCoordinate>,
    /// String de path SVG (`M ... L ... Q ...`) com cantos arredondados de 8px
    pub svg_path: String,
    pub start_label: String,
    pub start_label_pos: NodeCoordinate,
    pub end_label: String,
    pub end_label_pos: NodeCoordinate,
    pub cardinality: String,
}

/// Caixa delimitadora de um `TableGroup` em coordenadas de mundo.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupBox {
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub color: String,
    pub count: usize,
    pub tables: Vec<String>,
}

/// Metadados visuais calculados para uma tabela (altura, cor `tint` resolvida, colunas FK).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableVisual {
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub tint: Option<String>,
    pub group: Option<String>,
    pub fk_columns: Vec<String>,
}

/// Cena geométrica completa calculada pelo Rust a partir de `crates/app/src/canvas.rs`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneGeometry {
    pub positions: HashMap<String, NodeCoordinate>,
    pub tables: Vec<TableVisual>,
    pub groups: Vec<GroupBox>,
    pub edges: Vec<RoutedEdge>,
}
