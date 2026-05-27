use crate::ast::*;
use crate::lexer::Token;
use crate::lexer::TokenKind;
use crate::lexer::Span;
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
        self.peek().kind.is_eof()
    }

    fn expect(&mut self, expected: TokenKind) {
        let got = self.advance().kind;

        if got != expected {
            generate_error!("Parse error: expected `{}`, but got `{}`", expected, got);
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
        self.expect(TokenKind::Fun);

        let name = match self.advance().kind {
            TokenKind::Ident(n) => n,

            other => generate_error!("Expected function name after `fun`, but got `{}`", other),
        };

        self.expect(TokenKind::LParen);
        self.expect(TokenKind::RParen);
        self.expect(TokenKind::LBrace);

        let mut body = Vec::new();

        while self.peek().kind != TokenKind::RBrace && !self.is_eof() {
            body.push(self.parse_stmt());
        }

        self.expect(TokenKind::RBrace);

        FuncDef { name, body }
    }

    fn parse_stmt(&mut self) -> Stmt {
        let stmt = match self.peek().clone().kind {
            t if t.is_primitive_type() => {
                Stmt::VarDecl(self.parse_vardecl())
            }

            TokenKind::Ident(name) if name == "println" => {
                Stmt::Println(self.parse_println())
            },

            other => generate_error!("Parse error: unknown statement starting with `{}`", other),
        };

        self.expect(TokenKind::Semicolon);

        stmt
    }

    fn parse_println(&mut self) -> ExprId {
        self.scroll();
        self.expect(TokenKind::LParen);

        let arg = self.parse_expr();

        self.expect(TokenKind::RParen);

        arg
    }

    fn parse_vardecl(&mut self) -> VarDecl {
        let _type = match self.advance().kind {
            TokenKind::Unt => Type::Unt,
            TokenKind::Int => Type::Int,
            TokenKind::Float => Type::Float,
            TokenKind::Bool => Type::Bool,
            TokenKind::Str => Type::Str,

            other => generate_error!("Unknown type: `{}`", other),
        };

        let name = match self.advance().kind {
            TokenKind::Ident(name) => name,

            other => generate_error!("Expected variable name after type, but got `{}`", other),
        };

        self.expect(TokenKind::Assign);

        let expr_id = self.parse_expr();

        VarDecl { _type, name, expr_id }
    }

    fn make_binary(&mut self, op: BinaryOp, left: ExprId, right: ExprId) -> ExprId {
        let start = self.expr_arena.get(left).span.start;
        let end = self.expr_arena.get(right).span.end;
        
        self.expr_arena.add(
            ExprNode::new(
                Expr::Binary { op, left, right },
                Span::new(start, end),
            )
        )
    }

    fn parse_expr(&mut self) -> ExprId {
        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> ExprId {
        let mut expr = self.parse_logical();

        loop {
            let op = match self.peek().kind {
                TokenKind::EqEq => BinaryOp::EqEq,
                TokenKind::NotEq => BinaryOp::NotEq,
                TokenKind::Lt => BinaryOp::Lt,
                TokenKind::Gt => BinaryOp::Gt,
                TokenKind::LtEq => BinaryOp::LtEq,
                TokenKind::GtEq => BinaryOp::GtEq,
                _ => break,
            };

            self.scroll();

            let right = self.parse_logical();

            expr = self.make_binary(op, expr, right);
        }

        expr
    }

    fn parse_logical(&mut self) -> ExprId {
        let mut expr = self.parse_additive();

        loop {
            let op = match self.peek().kind {
                TokenKind::AndAnd => BinaryOp::And,
                TokenKind::OrOr => BinaryOp::Or,
                TokenKind::XorXor => BinaryOp::Xor,
                _ => break,
            };

            self.scroll();

            let right = self.parse_additive();

            expr = self.make_binary(op, expr, right);
        }

        expr
    }

    fn parse_additive(&mut self) -> ExprId {
        let mut expr = self.parse_term();

        loop {
            let op = match self.peek().kind {
                TokenKind::Plus => BinaryOp::Add,
                TokenKind::Minus => BinaryOp::Sub,
                _ => break,
            };
            
            self.scroll();
            
            let right = self.parse_term();
            
            expr = self.make_binary(op, expr, right);
        }

        expr
    }

    fn parse_term(&mut self) -> ExprId {
        let mut expr = self.parse_factor();

        loop {
            let op = match self.peek().kind {
                TokenKind::Star => BinaryOp::Mul,
                TokenKind::Slash => BinaryOp::Div,
                TokenKind::Percent => BinaryOp::Mod,
                _ => break,
            };
            
            self.scroll();
            
            let right = self.parse_factor();
            
            expr = self.make_binary(op, expr, right);
        }

        expr
    }

    fn parse_factor(&mut self) -> ExprId {
        match self.peek().kind.clone() {
            TokenKind::Minus => {
                let start = self.peek().span.start;

                self.scroll();

                match self.peek().kind {
                    TokenKind::NumLit(n) => {
                        let end = self.peek().span.end;

                        self.scroll();

                        self.expr_arena.add(ExprNode::new(
                            Expr::Lit(Lit::Int(-(n as i64))),
                            Span::new(start, end),
                        ))
                    }

                    TokenKind::FloatLit(f) => {
                        let end = self.peek().span.end;

                        self.scroll();

                        self.expr_arena.add(ExprNode::new(
                            Expr::Lit(Lit::Float(-f)),
                            Span::new(start, end),
                        ))
                    }

                    _ => {
                        let expr = self.parse_primary();
                        let end = self.expr_arena.get(expr).span.end;

                        self.expr_arena.add(ExprNode::new(
                            Expr::Unary {
                                op: UnaryOp::Minus,
                                expr,
                            },
                            Span::new(start, end),
                        ))
                    }
                }
            }
            
            TokenKind::Bang | TokenKind::Not => {
                let start = self.peek().span.start;
                
                self.scroll();
                
                let expr = self.parse_primary();
                
                let end = self.expr_arena.get(expr).span.end;

                self.expr_arena.add(ExprNode::new(
                    Expr::Unary {
                        op: UnaryOp::Not,
                        expr,
                    },
                    Span::new(start, end),
                ))
            }

            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> ExprId {
        match self.peek().kind.clone() {
            TokenKind::NumLit(n) => {
                let start = self.peek().span.start;
                let end = self.peek().span.end;

                self.scroll();

                self.expr_arena.add(ExprNode::new(
                    Expr::Lit(Lit::Unt(n)),
                    Span::new(start, end),
                ))
            }

            TokenKind::FloatLit(f) => {
                let start = self.peek().span.start;
                let end = self.peek().span.end;

                self.scroll();

                self.expr_arena.add(ExprNode::new(
                    Expr::Lit(Lit::Float(f)),
                    Span::new(start, end),
                ))
            }

            TokenKind::BoolLit(b) => {
                let start = self.peek().span.start;
                let end = self.peek().span.end;

                self.scroll();

                self.expr_arena.add(ExprNode::new(
                    Expr::Lit(Lit::Bool(b)),
                    Span::new(start, end),
                ))
            }

            TokenKind::StringLit(s) => {
                let start = self.peek().span.start;
                let end = self.peek().span.end;

                self.scroll();

                self.expr_arena.add(ExprNode::new(
                    Expr::Lit(Lit::Str(s)),
                    Span::new(start, end),
                ))
            }

            TokenKind::Ident(name) => {
                let span = self.peek().span.clone();

                self.scroll();

                self.expr_arena.add(ExprNode::new(
                    Expr::Var(name),
                    span,
                ))
            }

            TokenKind::LParen => {
                self.scroll();

                let expr = self.parse_expr();

                self.expect(TokenKind::RParen);

                expr
            }

            other => generate_error!("Invalid expression: `{}`", other),
        }
    }
}