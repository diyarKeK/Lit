use super::Span;

use std::fmt;

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    #[inline]
    pub fn new(kind: TokenKind, span: Span) -> Token {
        Token { kind, span }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenKind {
    Fun,

    Unt,
    Int,
    Float,
    Bool,
    Str,

    Ident(String),
    StringLit(String),
    NumLit(u64),
    FloatLit(f64),
    BoolLit(bool),

    Equal,
    Plus,
    Minus,
    Mul,
    Div,
    Rem,

    And,
    Or,
    Xor,
    Bang,

    Gt,
    Lt,

    LParen,
    RParen,
    LBrace,
    RBrace,
    Semicolon,

    Eof,
}

impl TokenKind {
    pub fn is_primitive_type(&self) -> bool {
        match self {
            TokenKind::Unt |
            TokenKind::Int |
            TokenKind::Float |
            TokenKind::Bool |
            TokenKind::Str => true,
            _ => false,
        }
    }

    pub fn is_eof(&self) -> bool {
        match self {
            TokenKind::Eof => true,
            _ => false,
        }
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TokenKind::Fun => write!(f, "fun"),
            TokenKind::Unt => write!(f, "unt"),
            TokenKind::Int => write!(f, "int"),
            TokenKind::Float => write!(f, "float"),
            TokenKind::Bool => write!(f, "bool"),
            TokenKind::Str => write!(f, "str"),
            TokenKind::Ident(name) => write!(f, "{}", name),
            TokenKind::StringLit(s) => write!(f, "\"{}\"", s),
            TokenKind::NumLit(n) => write!(f, "{}", n),
            TokenKind::FloatLit(n) => write!(f, "{}", n),
            TokenKind::BoolLit(b) => write!(f, "{}", b),
            TokenKind::Equal => write!(f, "="),
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Mul => write!(f, "*"),
            TokenKind::Div => write!(f, "/"),
            TokenKind::Rem => write!(f, "%"),
            TokenKind::And => write!(f, "&"),
            TokenKind::Or => write!(f, "|"),
            TokenKind::Xor => write!(f, "^"),
            TokenKind::Bang => write!(f, "!"),
            TokenKind::Gt => write!(f, ">"),
            TokenKind::Lt => write!(f, "<"),
            TokenKind::LParen => write!(f, "("),
            TokenKind::RParen => write!(f, ")"),
            TokenKind::LBrace => write!(f, "{{"),
            TokenKind::RBrace => write!(f, "}}"),
            TokenKind::Semicolon => write!(f, ";"),
            TokenKind::Eof => write!(f, "`End_Of_File`"),
        }
    }
}