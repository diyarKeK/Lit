use std::fmt;
use std::process;

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
    UntLit(u64),
    FloatLit(f64),
    BoolLit(bool),

    Equal,
    Minus,

    LParen,
    RParen,
    LBrace,
    RBrace,
    Semicolon,

    Eof,
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
            TokenKind::UntLit(n) => write!(f, "{}", n),
            TokenKind::FloatLit(n) => write!(f, "{}", n),
            TokenKind::BoolLit(b) => write!(f, "{}", b),
            TokenKind::Equal => write!(f, "="),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::LParen => write!(f, "("),
            TokenKind::RParen => write!(f, ")"),
            TokenKind::LBrace => write!(f, "{{"),
            TokenKind::RBrace => write!(f, "}}"),
            TokenKind::Semicolon => write!(f, ";"),
            TokenKind::Eof => write!(f, "'End_Of_File'"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
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
            let is_eof = token.kind == TokenKind::Eof;

            tokens.push(token);

            if is_eof { break; }
        }

        tokens
    }

    fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        let kind = match self.advance() {
            None => TokenKind::Eof,
            Some('(') => TokenKind::LParen,
            Some(')') => TokenKind::RParen,
            Some('{') => TokenKind::LBrace,
            Some('}') => TokenKind::RBrace,
            Some(';') => TokenKind::Semicolon,
            Some('=') => TokenKind::Equal,
            Some('-') => TokenKind::Minus,

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
                                eprintln!("Cannot resolve: '\\{}' unicode", other);
                                process::exit(1);
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
                TokenKind::StringLit(s)
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
                    TokenKind::FloatLit(num.parse().unwrap())
                } else {
                    TokenKind::UntLit(num.parse().unwrap())
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
                    "fun" => TokenKind::Fun,
                    "unt" => TokenKind::Unt,
                    "int" => TokenKind::Int,
                    "float" => TokenKind::Float,
                    "bool" => TokenKind::Bool,
                    "str" => TokenKind::Str,
                    "true" => TokenKind::BoolLit(true),
                    "false" => TokenKind::BoolLit(false),

                    _ => TokenKind::Ident(word),
                }
            }

            Some(other) => panic!("Unrecognized character: {}", other),
        };

        Token { kind }
    }
}