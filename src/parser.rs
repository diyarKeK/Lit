use std::process;

use crate::ast::*;
use crate::lexer::Token;
use crate::generate_error;

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

    fn peek(&self) -> &Token {
        &self.tokens.get(self.pos).unwrap()
    }

    fn scroll(&mut self) {
        self.pos += 1;
    }

    fn advance(&mut self) -> Token {
        let kind = self.tokens.get(self.pos).unwrap().clone();
        self.pos += 1;
        kind
    }

    fn is_eof(&self) -> bool {
        self.peek() == &Token::Eof
    }

    fn expect(&mut self, expected: Token) {
        let got = self.advance();

        if got != expected {
            generate_error!("Parse error: expected '{}' but got '{}'", expected, got);
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
        self.expect(Token::Fun);

        let name = match self.advance() {
            Token::Ident(n) => n,

            other => {
                generate_error!("Expected function name after 'fun', but got {:?}", other);
            }
        };

        self.expect(Token::LParen);
        self.expect(Token::RParen);
        self.expect(Token::LBrace);

        let mut body = Vec::new();

        while self.peek() != &Token::RBrace && !self.is_eof() {
            body.push(self.parse_stmt());
        }

        self.expect(Token::RBrace);

        FuncDef { name, body }
    }

    fn parse_stmt(&mut self) -> Stmt {
        let stmt = match self.peek() {
            Token::Unt | Token::Int | Token::Float | Token::Bool | Token::Str => {
                Stmt::VarDecl(self.parse_vardecl())
            }

            Token::Ident(name) if name == "println" => {
                Stmt::Println(self.parse_println())
            },

            other => {
                generate_error!("Parse error: unknown statement starting with `{}`", other);
            }
        };

        self.expect(Token::Semicolon);

        stmt
    }

    fn parse_vardecl(&mut self) -> VarDecl {
        let _type = match self.advance() {
            Token::Unt => Type::Unt,
            Token::Int => Type::Int,
            Token::Float => Type::Float,
            Token::Bool => Type::Bool,
            Token::Str => Type::Str,

            other => {
                generate_error!("Parse error: unknown type: `{}`", other);
            }
        };

        let name = match self.advance() {
            Token::Ident(name) => name,
            other => {
                generate_error!("Expected variable name after type, but got `{}`", other);
            }
        };

        self.expect(Token::Equal);

        let value = self.parse_value(&_type);

        VarDecl { _type, name, value }
    }

    fn parse_value(&mut self, _type: &Type) -> Value {
        match (_type, self.advance()) {
            (Type::Unt, Token::UntLit(n)) => Value::Unt(n),

            (Type::Int, Token::UntLit(n)) => Value::Int(n as i64),
            (Type::Int, Token::Minus) => {
                match self.advance() {
                    Token::UntLit(n) => Value::Int(-(n as i64)),
                    other => {
                        generate_error!("Incorrect value: value '{}' does not match to type '{:?}'", other, _type);
                    }
                }
            }

            (Type::Float, Token::FloatLit(n)) => Value::Float(n),
            (Type::Float, Token::UntLit(n)) => Value::Float(n as f64),
            (Type::Float, Token::Minus) => {
                match self.advance() {
                    Token::FloatLit(n) => Value::Float(-n),
                    Token::UntLit(n) => Value::Float(-(n as f64)),
                    other => {
                        generate_error!("Incorrect value: value '{}' does not match to type '{:?}'", other, _type);
                    }
                }
            }

            (Type::Bool, Token::BoolLit(b)) => Value::Bool(b),

            (Type::Str, Token::StringLit(s)) => Value::Str(s),

            (_type, other) => {
                generate_error!("Incorrect value: value '{}' does not match to type '{:?}'", other, _type)
            }
        }
    }

    fn parse_println(&mut self) -> PrintlnArg {
        self.scroll();
        self.expect(Token::LParen);

        let arg = match self.advance() {
            Token::StringLit(s) => PrintlnArg::StringLit(s),
            Token::Ident(var) => PrintlnArg::Var(var),

            other => {
                generate_error!("Expected literal string or variable as an argument for println(), but got {:?}", other);
            }
        };

        self.expect(Token::RParen);

        arg
    }
}