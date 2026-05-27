use std::fmt;

use super::Span;


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

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} : {}..{}", self.kind, self.span.start, self.span.end)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenKind {
    // Keywords
    Fun,                // `fun`

    Unt,                // `unt`
    Int,                // `int`
    Float,              // `float`
    Bool,               // `bool`
    Str,                // `str`

    // Identifier
    Ident(String),      // identifier
    
    // Literals
    StringLit(String),
    NumLit(u64),
    FloatLit(f64),
    BoolLit(bool),

    // Operators
    Assign,             // `=`
    Plus,               // `+`
    Minus,              // `-`
    Star,               // `*`
    Slash,              // `/`
    Percent,            // `%`

    // Logical
    And,                // `&`
    Or,                 // `|`
    Caret,                // `^`
    Not,                // `~`
    
    AndAnd,             // `&&`
    OrOr,               // `||`
    XorXor,             // `^^`
    Bang,               // `!`
    
    // Comparison
    EqEq,               // `==`
    NotEq,              // `!=`
    Gt,                 // `>`
    Lt,                 // `<`
    GtEq,               // `>=`
    LtEq,               // `<=`

    // Symbols
    LParen,             // `(`
    RParen,             // `)`
    LBrace,             // `{`
    RBrace,             // `}`
    Semicolon,          // `;`

    // End Of File
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
            TokenKind::Assign => write!(f, "="),
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Star => write!(f, "*"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::Percent => write!(f, "%"),
            TokenKind::And => write!(f, "&"),
            TokenKind::Or => write!(f, "|"),
            TokenKind::Caret => write!(f, "^"),
            TokenKind::Not => write!(f, "~"),
            TokenKind::AndAnd => write!(f, "&&"),
            TokenKind::OrOr => write!(f, "||"),
            TokenKind::XorXor => write!(f, "^^"),
            TokenKind::Bang => write!(f, "!"),
            TokenKind::EqEq => write!(f, "=="),
            TokenKind::NotEq => write!(f, "!="),
            TokenKind::Gt => write!(f, ">"),
            TokenKind::Lt => write!(f, "<"),
            TokenKind::GtEq => write!(f, ">="),
            TokenKind::LtEq => write!(f, "<="),
            TokenKind::LParen => write!(f, "("),
            TokenKind::RParen => write!(f, ")"),
            TokenKind::LBrace => write!(f, "{{"),
            TokenKind::RBrace => write!(f, "}}"),
            TokenKind::Semicolon => write!(f, ";"),
            TokenKind::Eof => write!(f, "`End_Of_File`"),
        }
    }
}