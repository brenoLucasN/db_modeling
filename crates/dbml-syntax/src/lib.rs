//! Lexer (logos) e parser tolerante que produz um CST rowan sem perdas.
//!
//! Invariante: `parse(text).syntax().text() == text` para qualquer entrada.

mod lexer;
mod parser;

pub use lexer::lex;
pub use parser::{Parse, parse};

use rowan::Language;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u16)]
pub enum SyntaxKind {
    // tokens
    Whitespace,
    Newline,
    Comment,
    Ident,
    Str,
    Number,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    LParen,
    RParen,
    Comma,
    Colon,
    Other,
    // nós
    Root,
    Table,
    TableGroup,
    Ref,
    Column,
    Block,
    Error,
}

impl SyntaxKind {
    pub fn is_trivia(self) -> bool {
        matches!(self, Self::Whitespace | Self::Newline | Self::Comment)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DbmlLang {}

impl Language for DbmlLang {
    type Kind = SyntaxKind;
    fn kind_from_raw(raw: rowan::SyntaxKind) -> SyntaxKind {
        assert!(raw.0 <= SyntaxKind::Error as u16);
        // SAFETY: repr(u16), valor validado acima e variantes contíguas.
        unsafe { std::mem::transmute::<u16, SyntaxKind>(raw.0) }
    }
    fn kind_to_raw(kind: SyntaxKind) -> rowan::SyntaxKind {
        rowan::SyntaxKind(kind as u16)
    }
}

pub type SyntaxNode = rowan::SyntaxNode<DbmlLang>;
pub type SyntaxToken = rowan::SyntaxToken<DbmlLang>;
