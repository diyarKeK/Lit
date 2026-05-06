use std::process;

use crate::ast::*;
use crate::lexer::{Token, TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens,
            pos: 0
        }
    }

    fn peek(&self) -> &TokenKind {
        &self.tokens.get(self.pos).unwrap().kind
    }

    fn scroll(&mut self) {
        self.pos += 1;
    }

    fn advance(&mut self) -> TokenKind {
        let kind = self.tokens.get(self.pos).unwrap().kind.clone();
        self.pos += 1;
        kind
    }

    fn is_eof(&self) -> bool {
        self.peek() == &TokenKind::Eof
    }

    fn expect(&mut self, expected: TokenKind) {
        let got = self.advance();

        if got != expected {
            eprintln!("Parse error: expected '{}' but got '{}'", expected, got);
            process::exit(1);
        }
    }

    pub fn parse(&mut self) -> Program {
        let mut funcs = Vec::new();

        while !self.is_eof() {
            funcs.push(self.parse_func());
        }

        Program {
            funcs,
        }
    }

    fn parse_func(&mut self) -> FuncDef {
        self.expect(TokenKind::Fun);

        let name = match self.advance() {
            TokenKind::Ident(n) => n,

            other => {
                eprintln!("Expected function name after 'fun', but got {:?}", other);
                process::exit(1);
            }
        };

        self.expect(TokenKind::LParen);
        self.expect(TokenKind::RParen);
        self.expect(TokenKind::LBrace);

        let mut body = Vec::new();

        while self.peek() != &TokenKind::RBrace && !self.is_eof() {
            body.push(self.parse_stmt());
        }

        self.expect(TokenKind::RBrace);

        FuncDef { name, body }
    }

    fn parse_stmt(&mut self) -> Stmt {
        match self.peek() {
            TokenKind::Unt | TokenKind::Int | TokenKind::Float | TokenKind::Bool | TokenKind::Str => {
                Stmt::VarDecl(self.parse_vardecl())
            }

            TokenKind::Ident(name) if name == "println" => {
                Stmt::Println(self.parse_println())
            },

            other => {
                eprintln!("Parse error: unknown statement starting with `{}`", other);
                process::exit(1);
            }
        }
    }

    fn parse_vardecl(&mut self) -> VarDecl {
        let _type = match self.advance() {
            TokenKind::Unt => Type::Unt,
            TokenKind::Int => Type::Int,
            TokenKind::Float => Type::Float,
            TokenKind::Bool => Type::Bool,
            TokenKind::Str => Type::Str,

            other => {
                eprintln!("Parse error: unknown type: `{}`", other);
                process::exit(1);
            }
        };

        let name = match self.advance() {
            TokenKind::Ident(name) => name,
            other => {
                eprintln!("Expected variable name after type, but got `{}`", other);
                process::exit(1);
            }
        };

        self.expect(TokenKind::Equal);

        let value = self.parse_value(&_type);

        self.expect(TokenKind::Semicolon);

        VarDecl { _type, name, value }
    }

    fn parse_value(&mut self, _type: &Type) -> Value {
        match (_type, self.advance()) {
            (Type::Unt, TokenKind::UntLit(n)) => Value::Unt(n),

            (Type::Int, TokenKind::UntLit(n)) => Value::Int(n as i64),
            (Type::Int, TokenKind::Minus) => {
                match self.advance() {
                    TokenKind::UntLit(n) => Value::Int(-(n as i64)),
                    other => {
                        eprintln!("Incorrect value: value '{}' does not match to type '{:?}'", other, _type);
                        process::exit(1);
                    }
                }
            }

            (Type::Float, TokenKind::FloatLit(n)) => Value::Float(n),
            (Type::Float, TokenKind::UntLit(n)) => Value::Float(n as f64),
            (Type::Float, TokenKind::Minus) => {
                match self.advance() {
                    TokenKind::FloatLit(n) => Value::Float(-n),
                    TokenKind::UntLit(n) => Value::Float(-(n as f64)),
                    other => {
                        eprintln!("Incorrect value: value '{}' does not match to type '{:?}'", other, _type);
                        process::exit(1);
                    }
                }
            }

            (Type::Bool, TokenKind::BoolLit(b)) => Value::Bool(b),

            (Type::Str, TokenKind::StringLit(s)) => Value::Str(s),

            (_type, other) => {
                eprintln!("Incorrect value: value '{}' does not match to type '{:?}'", other, _type);
                process::exit(1);
            }
        }
    }

    fn parse_println(&mut self) -> PrintlnArg {
        self.scroll();
        self.expect(TokenKind::LParen);

        let arg = match self.advance() {
            TokenKind::StringLit(s) => PrintlnArg::StringLit(s),
            TokenKind::Ident(var) => PrintlnArg::Var(var),

            other => {
                eprintln!("Expected literal string or variable for println(), but got {:?}", other);
                process::exit(1);
            }
        };

        self.expect(TokenKind::RParen);
        self.expect(TokenKind::Semicolon);

        arg
    }
}