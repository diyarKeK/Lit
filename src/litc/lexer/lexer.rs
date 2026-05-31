use crate::generate_error;
use super::token::Token;
use super::token::TokenKind;

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

            if self.peek() == Some('/') && let Some('/') = self.chars.get(self.pos + 1) {
                while matches!(self.peek(), Some(c) if c != '\n') {
                    self.scroll();
                }
                continue;
            }

            break;
        }
    }

    fn match_next(&mut self, next: char, yes: TokenKind, no: TokenKind) -> TokenKind {
        if self.peek() == Some(next) {
            self.scroll();
            yes
        } else {
            no
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
            Some('=') => self.match_next('=', TokenKind::EqEq, TokenKind::Assign),
            Some('+') => TokenKind::Plus,
            Some('-') => TokenKind::Minus,
            Some('*') => TokenKind::Star,
            Some('/') => TokenKind::Slash,
            Some('%') => TokenKind::Percent,
            Some('~') => TokenKind::Not,
            Some('&') => self.match_next('&', TokenKind::AndAnd, TokenKind::And),
            Some('|') => self.match_next('|', TokenKind::OrOr, TokenKind::Or),
            Some('^') => self.match_next('^', TokenKind::CaretCaret, TokenKind::Caret),
            Some('!') => self.match_next('=', TokenKind::NotEq, TokenKind::Bang),
            Some('>') => {
                match self.peek() {
                    Some('=') => { self.scroll(); TokenKind::GtEq },
                    Some('>') => { self.scroll(); TokenKind::RShift },
                    _ => TokenKind::Gt,
                }
            },
            Some('<') => {
                match self.peek() {
                    Some('=') => { self.scroll(); TokenKind::LtEq },
                    Some('<') => { self.scroll(); TokenKind::LShift },
                    _ => TokenKind::Lt,
                }
            },

            Some(q @ '"') => {
                let mut s = String::new();

                while let Some(c) = self.peek() {
                    if c == '\\' {
                        self.scroll();
                        let unicode = self.read_escape();

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

    fn read_escape(&mut self) -> char {
        match self.advance().unwrap() {
            'n' => '\n',
            'r' => '\r',
            't' => '\t',
            'c' => '\x1B',
            '"' => '"',
            '\\' => '\\',
            other => {
                generate_error!("Cannot resolve: `\\{}` unicode", other)
            },
        }
    }
}