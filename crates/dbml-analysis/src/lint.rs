//! Diagnósticos (lint) com intervalos de bytes, para o editor sublinhar.

use dbml_syntax::{SyntaxKind, SyntaxNode, SyntaxToken, parse};
use std::collections::{HashMap, HashSet};
use std::ops::Range;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub range: Range<usize>,
    pub severity: Severity,
    pub message: String,
}

fn diag(range: Range<usize>, severity: Severity, message: impl Into<String>) -> Diagnostic {
    Diagnostic {
        range,
        severity,
        message: message.into(),
    }
}

fn range(t: &SyntaxToken) -> Range<usize> {
    usize::from(t.text_range().start())..usize::from(t.text_range().end())
}

fn sig_tokens(n: &SyntaxNode) -> Vec<SyntaxToken> {
    n.descendants_with_tokens()
        .filter_map(|e| e.into_token())
        .filter(|t| !t.kind().is_trivia())
        .collect()
}

fn unquote(s: &str) -> &str {
    s.trim_matches(|c| c == '"' || c == '\'' || c == '`')
}

/// Palavras de topo que o parser ainda não modela, mas que são DBML válido.
const KNOWN_TOP: [&str; 5] = ["enum", "project", "tablegroup", "note", "records"];

pub fn lint(text: &str) -> Vec<Diagnostic> {
    let root = parse(text).syntax();
    let mut out = Vec::new();
    let mut tables: HashMap<String, HashSet<String>> = HashMap::new();

    // 1ª passada: tabelas, colunas, duplicatas, estrutura
    for t in root.children().filter(|n| n.kind() == SyntaxKind::Table) {
        let toks = sig_tokens(&t);
        let head: Vec<_> = t
            .children_with_tokens()
            .filter_map(|e| e.into_token())
            .filter(|k| !k.kind().is_trivia())
            .collect();
        let Some(name_tok) = head
            .iter()
            .find(|k| k.kind() != SyntaxKind::Ident || !k.text().eq_ignore_ascii_case("table"))
            .filter(|k| matches!(k.kind(), SyntaxKind::Ident | SyntaxKind::Str))
        else {
            out.push(diag(range(&toks[0]), Severity::Error, "Tabela sem nome"));
            continue;
        };
        let name = unquote(name_tok.text()).to_string();
        if !head.iter().any(|k| k.kind() == SyntaxKind::LBrace) {
            out.push(diag(
                range(name_tok),
                Severity::Error,
                format!("Falta `{{` depois de `Table {name}`"),
            ));
        } else if !head.iter().any(|k| k.kind() == SyntaxKind::RBrace) {
            out.push(diag(
                range(name_tok),
                Severity::Error,
                format!("Tabela `{name}` não foi fechada com `}}`"),
            ));
        }
        if tables.contains_key(&name) {
            out.push(diag(
                range(name_tok),
                Severity::Error,
                format!("Tabela `{name}` duplicada"),
            ));
        }
        let mut cols = HashSet::new();
        let mut has_pk = false;
        for c in t.children().filter(|n| n.kind() == SyntaxKind::Column) {
            let ct = sig_tokens(&c);
            let Some(first) = ct.first() else { continue };
            let cname = unquote(first.text()).to_string();
            let has_type = ct.get(1).is_some_and(|k| k.kind() != SyntaxKind::LBracket);
            if !has_type {
                out.push(diag(
                    range(first),
                    Severity::Error,
                    format!("Coluna `{cname}` sem tipo"),
                ));
            }
            if !cols.insert(cname.clone()) {
                out.push(diag(
                    range(first),
                    Severity::Error,
                    format!("Coluna `{cname}` duplicada em `{name}`"),
                ));
            }
            if ct.iter().any(|k| {
                k.text().eq_ignore_ascii_case("pk") || k.text().eq_ignore_ascii_case("primary")
            }) {
                has_pk = true;
            }
            if let Some(open) = ct.iter().position(|k| k.kind() == SyntaxKind::LBracket)
                && !ct[open..].iter().any(|k| k.kind() == SyntaxKind::RBracket)
            {
                out.push(diag(
                    range(&ct[open]),
                    Severity::Error,
                    "Falta `]` para fechar as configurações",
                ));
            }
        }
        if !has_pk && !cols.is_empty() {
            out.push(diag(
                range(name_tok),
                Severity::Info,
                format!("`{name}` não tem chave primária"),
            ));
        }
        tables.insert(name, cols);
    }

    // 2ª passada: referências (topo e inline)
    let check = |ts: &[SyntaxToken], out: &mut Vec<Diagnostic>| {
        // procura padrões `tabela . coluna`
        for w in ts.windows(3) {
            if w[1].text() == "." && matches!(w[0].kind(), SyntaxKind::Ident | SyntaxKind::Str) {
                let (tn, cn) = (unquote(w[0].text()), unquote(w[2].text()));
                match tables.get(tn) {
                    None => out.push(diag(
                        range(&w[0]),
                        Severity::Error,
                        format!("Tabela `{tn}` não existe"),
                    )),
                    Some(cols) if !cols.contains(cn) => out.push(diag(
                        range(&w[2]),
                        Severity::Error,
                        format!("Coluna `{cn}` não existe em `{tn}`"),
                    )),
                    _ => {}
                }
            }
        }
    };
    for r in root.children().filter(|n| n.kind() == SyntaxKind::Ref) {
        let ts = sig_tokens(&r);
        if !ts.iter().any(|k| k.text() == ":") {
            out.push(diag(
                range(&ts[0]),
                Severity::Error,
                "Esperado `Ref: tabela.coluna > tabela.coluna`",
            ));
        }
        check(&ts, &mut out);
    }
    for c in root
        .descendants()
        .filter(|n| n.kind() == SyntaxKind::Column)
    {
        let ts = sig_tokens(&c);
        if let Some(i) = ts.iter().position(|k| k.text().eq_ignore_ascii_case("ref")) {
            check(&ts[i..], &mut out);
        }
    }

    // 3ª passada: texto solto no topo que não é construção conhecida
    let mut line_start = true;
    for e in root.children_with_tokens() {
        match e {
            rowan::NodeOrToken::Token(t) => {
                if t.kind() == SyntaxKind::Newline {
                    line_start = true;
                } else if !t.kind().is_trivia() {
                    if line_start
                        && !(t.kind() == SyntaxKind::Ident
                            && KNOWN_TOP.contains(&t.text().to_ascii_lowercase().as_str()))
                    {
                        out.push(diag(
                            range(&t),
                            Severity::Warning,
                            format!("Construção não reconhecida: `{}`", t.text()),
                        ));
                    }
                    line_start = false;
                }
            }
            rowan::NodeOrToken::Node(_) => line_start = false,
        }
    }
    out.sort_by_key(|d| d.range.start);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn msgs(src: &str) -> Vec<String> {
        lint(src).into_iter().map(|d| d.message).collect()
    }

    #[test]
    fn clean_schema_has_only_infos() {
        let src = "Table a {\n  id int [pk]\n}\nTable b {\n  id int [pk]\n  a_id int [ref: > a.id]\n}\nRef: b.a_id > a.id\n";
        assert!(lint(src).is_empty(), "{:?}", lint(src));
    }

    #[test]
    fn unknown_ref_targets() {
        let m = msgs("Table a {\n  id int [pk]\n}\nRef: a.id > x.id\nRef: a.nope > a.id\n");
        assert!(m.contains(&"Tabela `x` não existe".to_string()), "{m:?}");
        assert!(
            m.contains(&"Coluna `nope` não existe em `a`".to_string()),
            "{m:?}"
        );
    }

    #[test]
    fn structural_errors() {
        let m = msgs("Table a {\n  id\n  id int [pk\n");
        assert!(m.iter().any(|s| s.contains("não foi fechada")), "{m:?}");
        assert!(m.iter().any(|s| s.contains("sem tipo")), "{m:?}");
        assert!(m.iter().any(|s| s.contains("duplicada")), "{m:?}");
        assert!(m.iter().any(|s| s.contains("Falta `]`")), "{m:?}");
    }

    #[test]
    fn stray_text_and_missing_pk() {
        let m = msgs("blah\nEnum status {\n}\nTable a {\n  x int\n}\n");
        assert!(
            m.iter().any(|s| s.contains("não reconhecida: `blah`")),
            "{m:?}"
        );
        assert!(!m.iter().any(|s| s.contains("`Enum`")), "{m:?}");
        assert!(
            m.iter().any(|s| s.contains("não tem chave primária")),
            "{m:?}"
        );
    }

    #[test]
    fn ranges_point_at_tokens() {
        let src = "Ref: a.id > x.id\n";
        let d = lint(src);
        let r = d.iter().find(|d| d.message.contains("`a`")).unwrap();
        assert_eq!(&src[r.range.clone()], "a");
    }
}
