use std::fmt;
use std::process;
use crate::generate_error;

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
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::LBrace => write!(f, "{{"),
            Token::RBrace => write!(f, "}}"),
            Token::Semicolon => write!(f, ";"),
            Token::Eof => write!(f, "'End_Of_File'"),
        }
    }
}


pub struct Lexer {
    chars: Vec<char>,
    pos: usize,
}

impl Lexer {
    pub fn new(src: &str) -> Lexer {
        Lexer {
            chars: src.chars().collect(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn scroll(&mut self) {
        self.pos += 1;
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.chars.get(self.pos).copied();
        self.pos += 1;
        c
    }

    fn skip_whitespace(&mut self) {
        loop {
            while matches!(self.peek(), Some(c) if c.is_whitespace()) {
                self.scroll();
            }

            if self.peek() == Some('#') {
                while matches!(self.peek(), Some(c) if c != '\n') {
                    self.scroll();
                }
                continue;
            }

            break;
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        loop {
            let token = self.next_token();
            let is_eof = token == Token::Eof;

            tokens.push(token);

            if is_eof { break; }
        }

        tokens
    }

    fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        let kind = match self.advance() {
            None => Token::Eof,
            Some('(') => Token::LParen,
            Some(')') => Token::RParen,
            Some('{') => Token::LBrace,
            Some('}') => Token::RBrace,
            Some(';') => Token::Semicolon,
            Some('=') => Token::Equal,
            Some('+') => Token::Plus,
            Some('-') => Token::Minus,
            Some('*') => Token::Mul,
            Some('/') => Token::Div,
            Some('%') => Token::Rem,

            Some(q @ '"') => {
                let mut s = String::new();

                while let Some(c) = self.peek() {
                    if c == '\\' {
                        self.scroll();
                        let unicode = match self.advance().unwrap() {
                            'n' => '\n',
                            'r' => '\r',
                            't' => '\t',
                            '"' => '"',
                            '\\' => '\\',
                            other => {
                                generate_error!("Cannot resolve: '\\{}' unicode", other);
                            },
                        };

                        s.push(unicode);
                        continue;
                    }

                    if c == q {
                        self.scroll();
                        break;
                    }

                    s.push(c);
                    self.scroll();
                }
                Token::StringLit(s)
            }

            Some(c) if c.is_ascii_digit() => {
                let mut num = String::from(c);
                let mut is_float = false;

                while let Some(nc) = self.peek() {
                    if nc.is_ascii_digit() {
                        num.push(nc);
                        self.scroll();
                    } else if nc == '.' && !is_float {
                        is_float = true;
                        num.push(nc);
                        self.scroll();
                    } else {
                        break;
                    }
                }

                if is_float {
                    Token::FloatLit(num.parse().unwrap())
                } else {
                    Token::UntLit(num.parse().unwrap())
                }
            }

            Some(c) if c.is_alphabetic() || c == '_' => {
                let mut word = String::from(c);
                while let Some(nc) = self.peek() {
                    if nc.is_alphanumeric() || nc == '_' {
                        word.push(nc);
                        self.scroll();
                    } else {
                        break;
                    }
                }

                match word.as_str() {
                    "fun" => Token::Fun,
                    "unt" => Token::Unt,
                    "int" => Token::Int,
                    "float" => Token::Float,
                    "bool" => Token::Bool,
                    "str" => Token::Str,
                    "true" => Token::BoolLit(true),
                    "false" => Token::BoolLit(false),

                    _ => Token::Ident(word),
                }
            }

            Some(other) => generate_error!("Unrecognized character: {}", other),
        };

        kind
    }
}