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
    IntLit(i64),
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

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.chars.get(self.pos).copied();
        self.pos += 1;
        c
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Some(c) if c.is_whitespace()) {
            self.advance();
        }
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
                    if c == q {
                        self.advance();
                        break;
                    }

                    s.push(c);
                    self.advance();
                }
                TokenKind::StringLit(s)
            }

            Some(c) if c.is_ascii_digit() => {
                let mut num = String::from(c);
                let mut is_float = false;

                while let Some(nc) = self.peek() {
                    if nc.is_ascii_digit() {
                        num.push(nc);
                        self.advance();
                    } else if nc == '.' && !is_float {
                        is_float = true;
                        num.push(nc);
                        self.advance();
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
                        self.advance();
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