//! Modelo semântico puro (tabelas, colunas, refs). Projeção derivada do CST.

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Schema {
    pub tables: Vec<Table>,
    pub refs: Vec<Ref>,
    pub groups: Vec<Group>,
}

/// `TableGroup nome [color: #hex] { tabela ... }`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Group {
    pub name: String,
    pub tables: Vec<String>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    pub name: String,
    pub columns: Vec<Column>,
    /// `[headercolor: #hex]`
    pub color: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Column {
    pub name: String,
    pub ty: String,
    pub pk: bool,
    pub not_null: bool,
    pub unique: bool,
}

/// Aresta entre duas colunas. `kind` segue a seta do DBML (`>` muitos-para-um etc.).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ref {
    pub from: Endpoint,
    pub to: Endpoint,
    pub kind: RefKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Endpoint {
    pub table: String,
    pub column: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefKind {
    ManyToOne,
    OneToMany,
    OneToOne,
    ManyToMany,
}

impl Schema {
    pub fn table(&self, name: &str) -> Option<&Table> {
        self.tables.iter().find(|t| t.name == name)
    }

    pub fn group_of(&self, table: &str) -> Option<&Group> {
        self.groups
            .iter()
            .find(|g| g.tables.iter().any(|t| t == table))
    }
}
