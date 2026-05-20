use crate::generate_error;
use super::token::Token;
use super::token::TokenKind;

use std::process;
use crate::lexer::Span;

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
            let is_eof = token.kind.is_eof();

            tokens.push(token);

            if is_eof { break; }
        }

        tokens
    }

    fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        
        let start = self.pos;

        let kind = match self.advance() {
            None => TokenKind::Eof,
            Some('(') => TokenKind::LParen,
            Some(')') => TokenKind::RParen,
            Some('{') => TokenKind::LBrace,
            Some('}') => TokenKind::RBrace,
            Some(';') => TokenKind::Semicolon,
            Some('=') => TokenKind::Equal,
            Some('+') => TokenKind::Plus,
            Some('-') => TokenKind::Minus,
            Some('*') => TokenKind::Mul,
            Some('/') => TokenKind::Div,
            Some('%') => TokenKind::Rem,
            Some('&') => TokenKind::And,
            Some('|') => TokenKind::Or,
            Some('^') => TokenKind::Xor,
            Some('!') => TokenKind::Bang,
            Some('>') => TokenKind::Gt,
            Some('<') => TokenKind::Lt,

            Some(q @ '"') => {
                let mut s = String::new();

                while let Some(c) = self.peek() {
                    if c == '\\' {
                        self.scroll();
                        let unicode = match self.advance().unwrap() {
                            'n' => '\n',
                            'r' => '\r',
                            't' => '\t',
                            'c' => '\x1B',
                            '"' => '"',
                            '\\' => '\\',
                            other => {
                                generate_error!("Cannot resolve: `\\{}` unicode", other);
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
                    TokenKind::NumLit(num.parse().unwrap())
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

            Some(other) => generate_error!("Unrecognized character: `{}`", other),
        };

        let span = Span::new(start, self.pos);
        Token::new(kind, span)
    }
}