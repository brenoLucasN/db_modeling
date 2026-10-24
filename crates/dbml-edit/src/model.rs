//! Edições estruturais feitas pela UI (canvas/inspector). Cada função devolve o texto
//! novo tocando só o necessário; comentários e formatação do resto ficam intactos.

use dbml_syntax::{SyntaxKind, SyntaxNode, SyntaxToken, parse};
use std::ops::Range;

fn unquote(s: &str) -> &str {
    s.trim_matches(|c| c == '"' || c == '\'' || c == '`')
}

fn sig(n: &SyntaxNode) -> Vec<SyntaxToken> {
    n.descendants_with_tokens()
        .filter_map(|e| e.into_token())
        .filter(|t| !t.kind().is_trivia())
        .collect()
}

fn range(t: &SyntaxToken) -> Range<usize> {
    usize::from(t.text_range().start())..usize::from(t.text_range().end())
}

fn tables(root: &SyntaxNode) -> impl Iterator<Item = SyntaxNode> {
    root.children().filter(|n| n.kind() == SyntaxKind::Table)
}

/// Token do nome no cabeçalho `Table nome ...`.
fn name_token(t: &SyntaxNode) -> Option<SyntaxToken> {
    t.children_with_tokens()
        .filter_map(|e| e.into_token())
        .filter(|k| matches!(k.kind(), SyntaxKind::Ident | SyntaxKind::Str))
        .nth(1)
}

fn find_table(root: &SyntaxNode, name: &str) -> Option<SyntaxNode> {
    tables(root).find(|t| name_token(t).is_some_and(|k| unquote(k.text()) == name))
}

fn find_column(t: &SyntaxNode, col: &str) -> Option<SyntaxNode> {
    t.children()
        .filter(|n| n.kind() == SyntaxKind::Column)
        .find(|c| sig(c).first().is_some_and(|k| unquote(k.text()) == col))
}

/// Aplica substituições (intervalo → texto), da última para a primeira.
fn apply(text: &str, mut edits: Vec<(Range<usize>, String)>) -> String {
    edits.sort_by_key(|(r, _)| std::cmp::Reverse(r.start));
    edits.dedup_by_key(|(r, _)| r.start);
    let mut out = text.to_string();
    for (r, s) in edits {
        out.replace_range(r, &s);
    }
    out
}

/// Linha inteira (com o `\n`) que contém o intervalo.
fn line_of(text: &str, r: Range<usize>) -> Range<usize> {
    let start = text[..r.start].rfind('\n').map_or(0, |i| i + 1);
    let end = text[r.end..]
        .find('\n')
        .map_or(text.len(), |i| r.end + i + 1);
    start..end
}

fn append_block(text: &str, block: &str) -> String {
    let mut out = text.trim_end_matches('\n').to_string();
    if !out.is_empty() {
        out.push_str("\n\n");
    }
    out.push_str(block);
    out
}

/// Primeiro nome livre `base`, `base_2`, `base_3`...
pub fn unique_table_name(text: &str, base: &str) -> String {
    let root = parse(text).syntax();
    let taken: Vec<String> = tables(&root)
        .filter_map(|t| name_token(&t))
        .map(|k| unquote(k.text()).to_string())
        .collect();
    std::iter::once(base.to_string())
        .chain((2..).map(|i| format!("{base}_{i}")))
        .find(|n| !taken.contains(n))
        .unwrap()
}

/// Primeiro nome de coluna livre na tabela.
pub fn unique_column_name(text: &str, table: &str, base: &str) -> String {
    let root = parse(text).syntax();
    let taken: Vec<String> = find_table(&root, table)
        .map(|t| {
            t.children()
                .filter(|n| n.kind() == SyntaxKind::Column)
                .filter_map(|c| sig(&c).first().map(|k| unquote(k.text()).to_string()))
                .collect()
        })
        .unwrap_or_default();
    std::iter::once(base.to_string())
        .chain((2..).map(|i| format!("{base}_{i}")))
        .find(|n| !taken.contains(n))
        .unwrap()
}

pub fn add_table(text: &str, name: &str) -> String {
    append_block(text, &format!("Table {name} {{\n  id int [pk]\n}}\n"))
}

/// Remove a tabela, as `Ref:` de topo que a citam e a sua linha em grupos.
pub fn delete_table(text: &str, name: &str) -> String {
    let root = parse(text).syntax();
    let mut edits = Vec::new();
    if let Some(t) = find_table(&root, name) {
        let mut r = line_of(
            text,
            usize::from(t.text_range().start())..usize::from(t.text_range().end()),
        );
        if text[..r.start].ends_with("\n\n") {
            r.start -= 1;
        }
        edits.push((r, String::new()));
    }
    for r in root.children().filter(|n| n.kind() == SyntaxKind::Ref) {
        let toks = sig(&r);
        if toks
            .windows(2)
            .any(|w| unquote(w[0].text()) == name && w[1].text() == ".")
        {
            edits.push((
                line_of(
                    text,
                    usize::from(r.text_range().start())..usize::from(r.text_range().end()),
                ),
                String::new(),
            ));
        }
    }
    let out = apply(text, edits);
    crate::ungroup_table(&out, name)
}

/// Renomeia a tabela no cabeçalho, em `tabela.coluna` (refs de topo e inline) e nos grupos.
pub fn rename_table(text: &str, old: &str, new: &str) -> String {
    let root = parse(text).syntax();
    let mut edits = Vec::new();
    if let Some(k) = find_table(&root, old).and_then(|t| name_token(&t)) {
        edits.push((range(&k), new.to_string()));
    }
    let toks: Vec<_> = root
        .descendants_with_tokens()
        .filter_map(|e| e.into_token())
        .filter(|t| !t.kind().is_trivia())
        .collect();
    for w in toks.windows(2) {
        if unquote(w[0].text()) == old && w[1].text() == "." {
            edits.push((range(&w[0]), new.to_string()));
        }
    }
    for g in root
        .children()
        .filter(|n| n.kind() == SyntaxKind::TableGroup)
    {
        for line in g.children().filter(|n| n.kind() == SyntaxKind::Column) {
            if let Some(k) = sig(&line).first().filter(|k| unquote(k.text()) == old) {
                edits.push((range(k), new.to_string()));
            }
        }
    }
    apply(text, edits)
}

pub fn add_column(text: &str, table: &str, name: &str, ty: &str) -> Option<String> {
    let cmd = crate::Command::AddColumn {
        table: table.into(),
        name: name.into(),
        ty: ty.into(),
    };
    crate::edit(text, &cmd).ok().map(|p| p.apply(text))
}

pub fn delete_column(text: &str, table: &str, col: &str) -> Option<String> {
    let root = parse(text).syntax();
    let c = find_column(&find_table(&root, table)?, col)?;
    let r = line_of(
        text,
        usize::from(c.text_range().start())..usize::from(c.text_range().end()),
    );
    Some(apply(text, vec![(r, String::new())]))
}

pub fn rename_column(text: &str, table: &str, old: &str, new: &str) -> Option<String> {
    let root = parse(text).syntax();
    let c = find_column(&find_table(&root, table)?, old)?;
    let mut edits = vec![(range(sig(&c).first()?), new.to_string())];
    // `tabela.coluna` em qualquer lugar
    let toks: Vec<_> = root
        .descendants_with_tokens()
        .filter_map(|e| e.into_token())
        .filter(|t| !t.kind().is_trivia())
        .collect();
    for w in toks.windows(3) {
        if unquote(w[0].text()) == table && w[1].text() == "." && unquote(w[2].text()) == old {
            edits.push((range(&w[2]), new.to_string()));
        }
    }
    Some(apply(text, edits))
}

pub fn set_column_type(text: &str, table: &str, col: &str, ty: &str) -> Option<String> {
    let root = parse(text).syntax();
    let c = find_column(&find_table(&root, table)?, col)?;
    let toks = sig(&c);
    let ty_toks: Vec<_> = toks[1..]
        .iter()
        .take_while(|k| k.kind() != SyntaxKind::LBracket)
        .collect();
    let edit = match (ty_toks.first(), ty_toks.last()) {
        (Some(a), Some(b)) => (
            usize::from(a.text_range().start())..usize::from(b.text_range().end()),
            ty.to_string(),
        ),
        _ => (range(&toks[0]).end..range(&toks[0]).end, format!(" {ty}")),
    };
    Some(apply(text, vec![edit]))
}

/// Liga/desliga `pk`, `not null` ou `unique` nas opções `[...]` da coluna.
pub fn set_column_flag(text: &str, table: &str, col: &str, flag: &str, on: bool) -> Option<String> {
    let root = parse(text).syntax();
    let c = find_column(&find_table(&root, table)?, col)?;
    let toks = sig(&c);
    let open = toks.iter().position(|k| k.kind() == SyntaxKind::LBracket);
    let close = toks.iter().position(|k| k.kind() == SyntaxKind::RBracket);
    let is_flag = |p: &str| {
        let p = p.trim().to_ascii_lowercase();
        p == flag || (flag == "pk" && p == "primary key") || (flag == "not null" && p == "null")
    };
    match (open, close) {
        (Some(o), Some(cl)) if o < cl => {
            let inner =
                usize::from(toks[o].text_range().end())..usize::from(toks[cl].text_range().start());
            let mut parts: Vec<String> = text[inner.clone()]
                .split(',')
                .map(|p| p.trim().to_string())
                .filter(|p| !p.is_empty() && !is_flag(p))
                .collect();
            if on {
                parts.insert(0, flag.to_string());
            }
            let whole =
                usize::from(toks[o].text_range().start())..usize::from(toks[cl].text_range().end());
            let edit = if parts.is_empty() {
                (
                    text[..whole.start].trim_end().len()..whole.end,
                    String::new(),
                )
            } else {
                (whole, format!("[{}]", parts.join(", ")))
            };
            Some(apply(text, vec![edit]))
        }
        _ if on => {
            let end = usize::from(toks.last()?.text_range().end());
            Some(apply(text, vec![(end..end, format!(" [{flag}]"))]))
        }
        _ => Some(text.to_string()),
    }
}

/// `Ref: a.b > c.d` (muitos-para-um) no fim do texto.
pub fn add_ref(text: &str, from: (&str, &str), to: (&str, &str)) -> String {
    let line = format!("Ref: {}.{} > {}.{}\n", from.0, from.1, to.0, to.1);
    let mut out = text.trim_end_matches('\n').to_string();
    out.push_str(if out.is_empty() { "" } else { "\n" });
    // junta com outras Refs se o texto já termina com uma; senão separa por linha em branco
    let last = out
        .lines()
        .last()
        .unwrap_or("")
        .trim_start()
        .to_ascii_lowercase();
    if !out.is_empty() && !last.starts_with("ref") {
        out.push('\n');
    }
    out.push_str(&line);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const S: &str = "// loja\nTable users {\n  id int [pk]\n  email text\n}\n\nTable posts {\n  id int [pk]\n  user_id int [ref: > users.id]\n}\n\nRef: posts.user_id > users.id\n\nTableGroup g {\n  users\n  posts\n}\n";

    #[test]
    fn table_lifecycle() {
        let n = unique_table_name(S, "users");
        assert_eq!(n, "users_2");
        let added = add_table(S, "tags");
        assert!(
            added.ends_with("\n\nTable tags {\n  id int [pk]\n}\n"),
            "{added}"
        );
        let renamed = rename_table(S, "users", "people");
        assert!(renamed.contains("Table people {"));
        assert!(renamed.contains("[ref: > people.id]"));
        assert!(renamed.contains("Ref: posts.user_id > people.id"));
        assert!(renamed.contains("  people\n  posts"));
        assert!(renamed.starts_with("// loja\n"));
        let deleted = delete_table(S, "users");
        assert!(!deleted.contains("Table users"), "{deleted}");
        assert!(!deleted.contains("Ref: posts.user_id"), "{deleted}");
        assert!(deleted.contains("TableGroup g {\n  posts\n}"), "{deleted}");
    }

    #[test]
    fn column_edits() {
        let r = rename_column(S, "users", "id", "uid").unwrap();
        assert!(
            r.contains("  uid int [pk]")
                && r.contains("> users.uid]")
                && r.contains("> users.uid\n"),
            "{r}"
        );
        assert!(
            r.contains("Table posts {\n  id int [pk]"),
            "não mexe em posts.id: {r}"
        );
        let t = set_column_type(S, "users", "email", "varchar(255)").unwrap();
        assert!(t.contains("  email varchar(255)\n"), "{t}");
        let d = delete_column(S, "users", "email").unwrap();
        assert!(d.contains("Table users {\n  id int [pk]\n}"), "{d}");
        assert_eq!(unique_column_name(S, "users", "email"), "email_2");
        let a = add_column(S, "users", "name", "text").unwrap();
        assert!(a.contains("  email text\n  name text\n"), "{a}");
    }

    #[test]
    fn flags() {
        let on = set_column_flag(S, "users", "email", "unique", true).unwrap();
        assert!(on.contains("  email text [unique]\n"), "{on}");
        let both = set_column_flag(&on, "users", "email", "not null", true).unwrap();
        assert!(both.contains("[not null, unique]"), "{both}");
        let off = set_column_flag(&both, "users", "email", "unique", false).unwrap();
        assert!(off.contains("[not null]"), "{off}");
        let none = set_column_flag(&off, "users", "email", "not null", false).unwrap();
        assert!(none.contains("  email text\n"), "{none}");
        let pk_off = set_column_flag(S, "users", "id", "pk", false).unwrap();
        assert!(pk_off.contains("Table users {\n  id int\n"), "{pk_off}");
    }

    #[test]
    fn refs() {
        let r = add_ref(S, ("posts", "id"), ("users", "id"));
        assert!(r.ends_with("}\n\nRef: posts.id > users.id\n"), "{r}");
        let only = add_ref("Table a {\n}\nRef: a.x > b.y\n", ("a", "z"), ("b", "w"));
        assert!(only.ends_with("Ref: a.x > b.y\nRef: a.z > b.w\n"), "{only}");
    }
}
