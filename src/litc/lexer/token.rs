use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Fun,

    Unt,
    Int,
    Float,
    Bool,
    Str,

    Ident(String),
    StringLit(String),
    UntLit(u64),
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

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Fun => write!(f, "fun"),
            Token::Unt => write!(f, "unt"),
            Token::Int => write!(f, "int"),
            Token::Float => write!(f, "float"),
            Token::Bool => write!(f, "bool"),
            Token::Str => write!(f, "str"),
            Token::Ident(name) => write!(f, "{}", name),
            Token::StringLit(s) => write!(f, "\"{}\"", s),
            Token::UntLit(n) => write!(f, "{}", n),
            Token::FloatLit(n) => write!(f, "{}", n),
            Token::BoolLit(b) => write!(f, "{}", b),
            Token::Equal => write!(f, "="),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Mul => write!(f, "*"),
            Token::Div => write!(f, "/"),
            Token::Rem => write!(f, "%"),
            Token::And => write!(f, "&"),
            Token::Or => write!(f, "|"),
            Token::Xor => write!(f, "^"),
            Token::Bang => write!(f, "!"),
            Token::Gt => write!(f, ">"),
            Token::Lt => write!(f, "<"),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::LBrace => write!(f, "{{"),
            Token::RBrace => write!(f, "}}"),
            Token::Semicolon => write!(f, ";"),
            Token::Eof => write!(f, "`End_Of_File`"),
        }
    }
}