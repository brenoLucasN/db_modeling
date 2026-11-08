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

/// Catálogo de tipos de dados SQL/DBML reconhecidos.
const KNOWN_TYPES: &[&str] = &[
    "int", "integer", "tinyint", "smallint", "mediumint", "bigint", "int2", "int4", "int8",
    "serial", "smallserial", "bigserial", "decimal", "numeric", "real", "float", "float4",
    "float8", "double", "money", "varchar", "char", "character", "nvarchar", "nchar", "text",
    "tinytext", "mediumtext", "longtext", "clob", "uuid", "boolean", "bool", "bit", "date",
    "datetime", "datetime2", "smalldatetime", "time", "timetz", "timestamp", "timestamptz",
    "interval", "year", "json", "jsonb", "xml", "bytea", "blob", "binary", "varbinary", "inet",
    "cidr", "macaddr", "point", "line", "polygon", "geometry", "geography", "tsvector", "tsquery",
    "array", "enum", "set",
];

fn levenshtein(a: &str, b: &str) -> usize {
    let b_chars: Vec<char> = b.chars().collect();
    let mut costs: Vec<usize> = (0..=b_chars.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut last = i;
        costs[0] = i + 1;
        for (j, &cb) in b_chars.iter().enumerate() {
            let old = costs[j + 1];
            costs[j + 1] = if ca == cb {
                last
            } else {
                1 + last.min(old).min(costs[j])
            };
            last = old;
        }
    }
    costs[b_chars.len()]
}

fn suggest_type(input: &str) -> Option<&'static str> {
    let low = input.to_ascii_lowercase();
    KNOWN_TYPES
        .iter()
        .copied()
        .filter_map(|t| {
            let d = levenshtein(&low, t);
            (d <= 2 || (low.len() >= 4 && t.starts_with(&low))).then_some((d, t))
        })
        .min_by_key(|(d, _)| *d)
        .map(|(_, t)| t)
}

fn collect_enums(text: &str) -> HashSet<String> {
    let mut enums = HashSet::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed
            .strip_prefix("Enum ")
            .or_else(|| trimmed.strip_prefix("enum "))
        {
            let name = rest
                .split(|c: char| c.is_whitespace() || c == '{')
                .next()
                .unwrap_or("")
                .trim();
            if !name.is_empty() {
                enums.insert(unquote(name).to_ascii_lowercase());
            }
        }
    }
    enums
}

fn check_column_type(
    cname: &str,
    ty_toks: &[SyntaxToken],
    enums: &HashSet<String>,
    out: &mut Vec<Diagnostic>,
) {
    let base_tok = &ty_toks[0];
    let base = unquote(base_tok.text()).to_ascii_lowercase();
    let is_known = KNOWN_TYPES.contains(&base.as_str()) || enums.contains(&base);
    if !is_known {
        let msg = match suggest_type(&base) {
            Some(sug) => format!(
                "Tipo desconhecido `{}` na coluna `{cname}` (você quis dizer `{sug}`?)",
                base_tok.text()
            ),
            None => format!(
                "Tipo desconhecido `{}` na coluna `{cname}`",
                base_tok.text()
            ),
        };
        out.push(diag(range(base_tok), Severity::Error, msg));
    }

    // Valida parênteses de precisão/tamanho e tokens extras antes de `[`
    let rest = &ty_toks[1..];
    if !rest.is_empty() {
        let lparen = rest.iter().position(|k| k.kind() == SyntaxKind::LParen);
        let rparen = rest.iter().position(|k| k.kind() == SyntaxKind::RParen);
        match (lparen, rparen) {
            (Some(0), Some(r)) => {
                let inner = &rest[1..r];
                if inner.is_empty() {
                    out.push(diag(
                        range(&rest[0]),
                        Severity::Error,
                        format!("Parâmetros vazios `()` no tipo `{}`", base_tok.text()),
                    ));
                } else if inner
                    .iter()
                    .any(|k| !matches!(k.kind(), SyntaxKind::Number | SyntaxKind::Comma))
                {
                    out.push(diag(
                        range(&inner[0]),
                        Severity::Error,
                        format!(
                            "Tamanho/precisão inválido para o tipo `{}` (esperado número)",
                            base_tok.text()
                        ),
                    ));
                }
                if let Some(extra) = rest.get(r + 1) {
                    out.push(diag(
                        range(extra),
                        Severity::Error,
                        format!("Token inesperado `{}` após o tipo da coluna `{cname}`", extra.text()),
                    ));
                }
            }
            (Some(l), None) => {
                out.push(diag(
                    range(&rest[l]),
                    Severity::Error,
                    format!("Falta `)` nos parâmetros do tipo `{}`", base_tok.text()),
                ));
            }
            (None, Some(r)) => {
                out.push(diag(
                    range(&rest[r]),
                    Severity::Error,
                    format!("`)` inesperado no tipo da coluna `{cname}`"),
                ));
            }
            _ => {
                out.push(diag(
                    range(&rest[0]),
                    Severity::Error,
                    format!(
                        "Token inesperado `{}` após o tipo `{}` na coluna `{cname}`",
                        rest[0].text(),
                        base_tok.text()
                    ),
                ));
            }
        }
    }
}

fn check_column_settings(
    cname: &str,
    ct: &[SyntaxToken],
    open_idx: usize,
    out: &mut Vec<Diagnostic>,
) {
    let Some(close_rel) = ct[open_idx..].iter().position(|k| k.kind() == SyntaxKind::RBracket) else {
        out.push(diag(
            range(&ct[open_idx]),
            Severity::Error,
            "Falta `]` para fechar as configurações",
        ));
        return;
    };
    let close_idx = open_idx + close_rel;
    if let Some(extra) = ct.get(close_idx + 1) {
        out.push(diag(
            range(extra),
            Severity::Error,
            format!(
                "Token inesperado `{}` após `]` na coluna `{cname}`",
                extra.text()
            ),
        ));
    }
    let inner = &ct[open_idx + 1..close_idx];
    if inner.is_empty() {
        out.push(diag(
            range(&ct[open_idx]),
            Severity::Warning,
            format!("Bloco de configurações `[]` vazio na coluna `{cname}`"),
        ));
        return;
    }

    let mut has_null = false;
    let mut has_not_null = false;
    let mut has_pk = false;

    for part in inner.split(|k| k.kind() == SyntaxKind::Comma) {
        if part.is_empty() {
            continue;
        }
        let first = &part[0];
        let kw = first.text().to_ascii_lowercase();
        match kw.as_str() {
            "pk" | "fk" | "unique" | "increment" => {
                if kw == "pk" {
                    has_pk = true;
                }
                if let Some(extra) = part.get(1) {
                    out.push(diag(
                        range(extra),
                        Severity::Error,
                        format!("Token inesperado `{}` após `{kw}` (use vírgula para separar opções)", extra.text()),
                    ));
                }
            }
            "primary" => {
                has_pk = true;
                let ok = part
                    .get(1)
                    .is_some_and(|k| k.text().eq_ignore_ascii_case("key"))
                    && part.len() == 2;
                if !ok {
                    out.push(diag(
                        range(first),
                        Severity::Error,
                        "Configuração incompleta: esperado `primary key` ou `pk`",
                    ));
                }
            }
            "null" => {
                has_null = true;
                if let Some(extra) = part.get(1) {
                    out.push(diag(
                        range(extra),
                        Severity::Error,
                        format!("Token inesperado `{}` após `null`", extra.text()),
                    ));
                }
            }
            "not" => {
                has_not_null = true;
                let ok = part
                    .get(1)
                    .is_some_and(|k| k.text().eq_ignore_ascii_case("null"))
                    && part.len() == 2;
                if !ok {
                    out.push(diag(
                        range(first),
                        Severity::Error,
                        "Configuração incompleta: esperado `not null`",
                    ));
                }
            }
            "default" | "note" | "ref" => {
                let has_colon = part.get(1).is_some_and(|k| k.kind() == SyntaxKind::Colon);
                let has_val = part.len() >= 3;
                if !has_colon || !has_val {
                    out.push(diag(
                        range(first),
                        Severity::Error,
                        format!("A configuração `{kw}:` exige um valor na coluna `{cname}`"),
                    ));
                }
            }
            _ => {
                out.push(diag(
                    range(first),
                    Severity::Error,
                    format!(
                        "Configuração desconhecida `{}` na coluna `{cname}` (esperado: pk, fk, not null, unique, increment, default, note, ref)",
                        first.text()
                    ),
                ));
            }
        }
    }

    if has_null && (has_not_null || has_pk) {
        out.push(diag(
            range(&ct[open_idx]),
            Severity::Error,
            format!("Restrições conflitantes na coluna `{cname}`: não pode ser `null` e `not null`/`pk` simultaneamente"),
        ));
    }
}

pub fn lint(text: &str) -> Vec<Diagnostic> {
    let root = parse(text).syntax();
    let mut out = Vec::new();
    let mut tables: HashMap<String, HashSet<String>> = HashMap::new();
    let mut col_types: HashMap<(String, String), String> = HashMap::new();
    let enums = collect_enums(text);

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
            let bracket_pos = ct.iter().position(|k| k.kind() == SyntaxKind::LBracket);
            let ty_toks = match bracket_pos {
                Some(b) => &ct[1..b],
                None => &ct[1..],
            };
            if ty_toks.is_empty() {
                out.push(diag(
                    range(first),
                    Severity::Error,
                    format!("Coluna `{cname}` sem tipo"),
                ));
            } else {
                check_column_type(&cname, ty_toks, &enums, &mut out);
                col_types.insert(
                    (name.clone(), cname.clone()),
                    unquote(ty_toks[0].text()).to_ascii_lowercase(),
                );
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
            if let Some(open) = bracket_pos {
                check_column_settings(&cname, &ct, open, &mut out);
            } else if let Some(unopened_close) =
                ct.iter().position(|k| k.kind() == SyntaxKind::RBracket)
            {
                out.push(diag(
                    range(&ct[unopened_close]),
                    Severity::Error,
                    "Colchete `]` sem `[` correspondente",
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
        let Some(colon) = ts.iter().position(|k| k.text() == ":") else {
            out.push(diag(
                range(&ts[0]),
                Severity::Error,
                "Esperado `Ref: tabela.coluna > tabela.coluna`",
            ));
            continue;
        };
        check(&ts, &mut out);
        let body = &ts[colon + 1..];
        let has_dot1 = body.get(1).is_some_and(|k| k.text() == ".");
        let mut op_idx = 3;
        while op_idx < body.len() && matches!(body[op_idx].text(), "<" | ">" | "-") {
            op_idx += 1;
        }
        let has_op = op_idx > 3;
        let has_dot2 = body.get(op_idx + 1).is_some_and(|k| k.text() == ".")
            && body.get(op_idx + 2).is_some();
        if !has_dot1 || !has_op || !has_dot2 {
            out.push(diag(
                range(&ts[0]),
                Severity::Error,
                "Relação incompleta: esperado `Ref: tabela.coluna > tabela.coluna`",
            ));
        } else {
            let (ft, fc) = (unquote(body[0].text()), unquote(body[2].text()));
            let (tt, tc) = (
                unquote(body[op_idx].text()),
                unquote(body[op_idx + 2].text()),
            );
            if ft == tt && fc == tc {
                out.push(diag(
                    range(&body[0]),
                    Severity::Error,
                    format!("Uma coluna não pode referenciar a si mesma (`{ft}.{fc}`)"),
                ));
            } else if let (Some(t1), Some(t2)) = (
                col_types.get(&(ft.to_string(), fc.to_string())),
                col_types.get(&(tt.to_string(), tc.to_string())),
            ) && t1 != t2
            {
                out.push(diag(
                    range(&body[2]),
                    Severity::Warning,
                    format!(
                        "Tipos incompatíveis na relação: `{ft}.{fc}` é `{t1}`, mas `{tt}.{tc}` é `{t2}`"
                    ),
                ));
            }
        }
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

    // Validação de TableGroup (tabelas inexistentes ou cabeçalho malformado)
    for g in root
        .children()
        .filter(|n| n.kind() == SyntaxKind::TableGroup)
    {
        let toks = sig_tokens(&g);
        let head: Vec<_> = g
            .children_with_tokens()
            .filter_map(|e| e.into_token())
            .filter(|k| !k.kind().is_trivia())
            .collect();
        let Some(gname_tok) = head
            .iter()
            .find(|k| {
                k.kind() != SyntaxKind::Ident || !k.text().eq_ignore_ascii_case("tablegroup")
            })
            .filter(|k| matches!(k.kind(), SyntaxKind::Ident | SyntaxKind::Str))
        else {
            out.push(diag(range(&toks[0]), Severity::Error, "Grupo (`TableGroup`) sem nome"));
            continue;
        };
        let gname = unquote(gname_tok.text());
        if !head.iter().any(|k| k.kind() == SyntaxKind::RBrace) {
            out.push(diag(
                range(gname_tok),
                Severity::Error,
                format!("Grupo `{gname}` não foi fechado com `}}`"),
            ));
        }
        for item in g.children().filter(|n| n.kind() == SyntaxKind::Column) {
            let it = sig_tokens(&item);
            if let Some(first) = it.first() {
                let tname = unquote(first.text());
                if !tables.contains_key(tname) {
                    out.push(diag(
                        range(first),
                        Severity::Error,
                        format!("Tabela `{tname}` listada no grupo `{gname}` não existe"),
                    ));
                }
            }
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

    #[test]
    fn detects_unknown_column_types_and_suggests_fix() {
        let src = "Table users {\n  id int [pk]\n  created_at timestam}\n";
        let m = msgs(src);
        assert!(
            m.iter()
                .any(|s| s.contains("Tipo desconhecido `timestam`") && s.contains("`timestamp`")),
            "Deve acusar erro em timestam e sugerir timestamp: {m:?}"
        );
    }

    #[test]
    fn detects_malformed_type_params_and_settings() {
        let src = "Table users {\n  id int [pk, null]\n  email varchar() [uniq]\n  bio text [default:]\n}\n";
        let m = msgs(src);
        assert!(m.iter().any(|s| s.contains("conflitantes")), "{m:?}");
        assert!(m.iter().any(|s| s.contains("Parâmetros vazios")), "{m:?}");
        assert!(m.iter().any(|s| s.contains("Configuração desconhecida `uniq`")), "{m:?}");
        assert!(m.iter().any(|s| s.contains("exige um valor")), "{m:?}");
    }

    #[test]
    fn detects_ref_type_mismatch_and_self_ref() {
        let src = "Table a {\n  id int [pk]\n}\nTable b {\n  id int [pk]\n  a_id text\n}\nRef: b.a_id > a.id\nRef: a.id > a.id\n";
        let m = msgs(src);
        assert!(m.iter().any(|s| s.contains("Tipos incompatíveis")), "{m:?}");
        assert!(m.iter().any(|s| s.contains("referenciar a si mesma")), "{m:?}");
    }
}
