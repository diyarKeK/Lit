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
    As,                 // `as`

    // Primitive types
    Unt,                // `unt`
    Int,                // `int`
    Float,              // `float`
    Bool,               // `bool`
    Char,               // `char`
    Str,                // `str`

    // Identifier
    Ident(String),      // identifier
    
    // Literals
    NumLit(u64),
    FloatLit(f64),
    BoolLit(bool),
    CharLit(char),
    StringLit(String),

    // Operators
    Assign,             // `=`
    Plus,               // `+`
    Minus,              // `-`
    Star,               // `*`
    Slash,              // `/`
    Percent,            // `%`

    // Comparison
    EqEq,               // `==`
    NotEq,              // `!=`
    Gt,                 // `>`
    Lt,                 // `<`
    GtEq,               // `>=`
    LtEq,               // `<=`
    
    // Logical
    And,                // `&`
    Or,                 // `|`
    Caret,              // `^`
    Tilde,                // `~`
    
    AndAnd,             // `&&`
    OrOr,               // `||`
    CaretCaret,         // `^^`
    Bang,               // `!`
    
    // Bitwise
    LShift,             // `<<`
    RShift,             // `>>`

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
            TokenKind::Char |
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
            TokenKind::As => write!(f, "as"),
            TokenKind::Unt => write!(f, "unt"),
            TokenKind::Int => write!(f, "int"),
            TokenKind::Float => write!(f, "float"),
            TokenKind::Bool => write!(f, "bool"),
            TokenKind::Char => write!(f, "char"),
            TokenKind::Str => write!(f, "str"),
            TokenKind::Ident(name) => write!(f, "{}", name),
            TokenKind::StringLit(s) => write!(f, "\"{}\"", s),
            TokenKind::NumLit(n) => write!(f, "{}", n),
            TokenKind::FloatLit(n) => write!(f, "{}", n),
            TokenKind::BoolLit(b) => write!(f, "{}", b),
            TokenKind::CharLit(c) => write!(f, "{}", c),
            TokenKind::Assign => write!(f, "="),
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Star => write!(f, "*"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::Percent => write!(f, "%"),
            TokenKind::And => write!(f, "&"),
            TokenKind::Or => write!(f, "|"),
            TokenKind::Caret => write!(f, "^"),
            TokenKind::Tilde => write!(f, "~"),
            TokenKind::AndAnd => write!(f, "&&"),
            TokenKind::OrOr => write!(f, "||"),
            TokenKind::CaretCaret => write!(f, "^^"),
            TokenKind::Bang => write!(f, "!"),
            TokenKind::EqEq => write!(f, "=="),
            TokenKind::NotEq => write!(f, "!="),
            TokenKind::Gt => write!(f, ">"),
            TokenKind::Lt => write!(f, "<"),
            TokenKind::GtEq => write!(f, ">="),
            TokenKind::LtEq => write!(f, "<="),
            TokenKind::LShift => write!(f, "<<"),
            TokenKind::RShift => write!(f, ">>"),
            TokenKind::LParen => write!(f, "("),
            TokenKind::RParen => write!(f, ")"),
            TokenKind::LBrace => write!(f, "{{"),
            TokenKind::RBrace => write!(f, "}}"),
            TokenKind::Semicolon => write!(f, ";"),
            TokenKind::Eof => write!(f, "End_Of_File"),
        }
    }
}