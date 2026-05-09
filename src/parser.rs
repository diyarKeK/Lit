use std::process;

use crate::ast::*;
use crate::lexer::Token;
use crate::generate_error;

pub struct Parser {
    tokens: Vec<Token>,
    expr_arena: ExprArena,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens,
            expr_arena: ExprArena::new(),
            pos: 0,
        }
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap()
    }

    fn scroll(&mut self) {
        self.pos += 1;
    }

    fn advance(&mut self) -> Token {
        let token = self.tokens.get(self.pos).unwrap().clone();
        self.pos += 1;
        token
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

    pub fn parse(mut self) -> Program {
        let mut funcs = Vec::new();

        while !self.is_eof() {
            funcs.push(self.parse_func());
        }

        Program {
            funcs,
            expr_arena: self.expr_arena,
        }
    }

    fn parse_func(&mut self) -> FuncDef {
        self.expect(Token::Fun);

        let name = match self.advance() {
            Token::Ident(n) => n,

            other => generate_error!("Expected function name after 'fun', but got {:?}", other),
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

            other => generate_error!("Parse error: unknown statement starting with `{}`", other),
        };

        self.expect(Token::Semicolon);

        stmt
    }

    fn parse_println(&mut self) -> ExprId {
        self.scroll();
        self.expect(Token::LParen);

        let arg = self.parse_expr();

        self.expect(Token::RParen);

        arg
    }

    fn parse_vardecl(&mut self) -> VarDecl {
        let _type = match self.advance() {
            Token::Unt => Type::Unt,
            Token::Int => Type::Int,
            Token::Float => Type::Float,
            Token::Bool => Type::Bool,
            Token::Str => Type::Str,

            other => generate_error!("Unknown type: `{}`", other),
        };

        let name = match self.advance() {
            Token::Ident(name) => name,

            other => generate_error!("Expected variable name after type, but got `{}`", other),
        };

        self.expect(Token::Equal);

        let value = self.parse_expr();

        VarDecl { _type, name, value }
    }


    fn parse_expr(&mut self) -> ExprId {
        let mut expr = self.parse_term();

        loop {
            match self.peek() {
                Token::Plus => {
                    self.scroll();
                    let right = self.parse_term();

                    expr = self.expr_arena.add(Expr::Binary {
                        left: expr,
                        op: Operand::Plus,
                        right,
                    });
                }

                Token::Minus => {
                    self.scroll();
                    let right = self.parse_term();

                    expr = self.expr_arena.add(Expr::Binary {
                        left: expr,
                        op: Operand::Minus,
                        right,
                    });
                }

                _ => break,
            }
        }

        expr
    }

    fn parse_term(&mut self) -> ExprId {
        let mut expr = self.parse_factor();

        loop {
            match self.peek() {
                Token::Mul => {
                    self.scroll();
                    let right = self.parse_factor();

                    expr = self.expr_arena.add(Expr::Binary {
                        left: expr,
                        op: Operand::Mul,
                        right,
                    });
                }

                Token::Div => {
                    self.scroll();
                    let right = self.parse_factor();

                    expr = self.expr_arena.add(Expr::Binary {
                        left: expr,
                        op: Operand::Div,
                        right,
                    });
                }

                Token::Rem => {
                    self.scroll();
                    let right = self.parse_factor();

                    expr = self.expr_arena.add(Expr::Binary {
                        left: expr,
                        op: Operand::Rem,
                        right,
                    });
                }

                _ => break,
            }
        }

        expr
    }

    fn parse_factor(&mut self) -> ExprId {
        match self.advance() {
            Token::UntLit(u) => self.expr_arena.add(Expr::Unt(u)),

            Token::FloatLit(f) => self.expr_arena.add(Expr::Float(f)),

            Token::BoolLit(b) => self.expr_arena.add(Expr::Bool(b)),

            Token::StringLit(s) => self.expr_arena.add(Expr::Str(s)),

            Token::Ident(name) => self.expr_arena.add(Expr::Var(name)),

            Token::Minus => {
                match self.advance() {
                    Token::UntLit(n) => self.expr_arena.add(Expr::Int(-(n as i64))),
                    Token::FloatLit(f) => self.expr_arena.add(Expr::Float(-f)),

                    other => generate_error!("Expected number after '-', got: `{}`", other),
                }
            }

            Token::LParen => {
                let expr_id = self.parse_expr();

                if self.advance() != Token::RParen {
                    generate_error!("Expected ')' to close left paren in expression")
                }

                expr_id
            },

            other => generate_error!("Invalid expression: {}", other),
        }
    }
}