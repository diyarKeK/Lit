use std::collections::HashMap;

use crate::ast::*;
use crate::generate_plain_error;

pub fn analyze(program: &Program) {
    let mut analyzer = Analyzer::new(&program.expr_arena);

    for func in &program.funcs {
        analyzer.analyze_func(func);
    }
}

struct Analyzer<'a> {
    arena: &'a ExprArena,
    declared: HashMap<String, Type>,
}

impl<'a> Analyzer<'a> {
    fn new(expr_arena: &'a ExprArena) -> Analyzer<'a> {
        Analyzer {
            arena: expr_arena,
            declared: HashMap::new(),
        }
    }
    fn analyze_func(&mut self, func: &FuncDef) {
        self.declared.clear();

        for stmt in func.body.stmts() {
            match stmt {
                Stmt::VarDecl(v) => {
                    if self.declared.contains_key(&v.name) {
                        generate_plain_error!("Variable `{}` is already declared", v.name);
                    }

                    let expr_type = self.infer_type(v.expr_id);

                    if expr_type != v._type {
                        generate_plain_error!(
                            "Cannot assign {} value to variable `{}` of type `{}`",
                            expr_type, v.name, v._type
                        );
                    }

                    self.declared.insert(v.name.clone(), v._type.clone());
                }
                Stmt::Println(expr_id) => {
                    self.infer_type(*expr_id);
                }
                _ => {}
            }
        }
    }

    fn infer_type(&self, id: ExprId) -> Type {
        let expr_node = self.arena.get(id);
        let expr = &expr_node.expr;

        use Lit::*;
        match expr {
            Expr::Lit(Unt(_)) => Type::Unt,
            Expr::Lit(Int(_)) => Type::Int,
            Expr::Lit(Float(_)) => Type::Float,
            Expr::Lit(Bool(_)) => Type::Bool,
            Expr::Lit(Char(_)) => Type::Char,
            Expr::Lit(Str(_)) => Type::Str,

            Expr::Var(name) => {
                self.declared.get(name).unwrap_or_else(|| {
                    generate_plain_error!("Variable `{}` is not declared", name)
                }).clone()
            }

            Expr::Binary (op, left, right) => {
                let left_ty = self.infer_type(*left);
                let right_ty = self.infer_type(*right);

                if left_ty != right_ty {
                    generate_plain_error!(
                        "Cannot apply operator `{op}` for types: `{left}` and `{right}`",
                        op = op, left = left_ty, right = right_ty
                    )
                }

                if op.is_comparison() {
                    Type::Bool

                } else if
                    op.is_arranging()
                    && left_ty.is_num_type()
                    && right_ty.is_num_type()
                {
                    Type::Bool

                } else if
                    op.is_arithmetic()
                    && left_ty.is_num_type()
                    && right_ty.is_num_type()
                {
                    left_ty

                } else if
                    op.is_logical()
                    && left_ty.is_logical_type()
                    && right_ty.is_logical_type()
                {
                    left_ty

                } else if
                    op.is_bitwise()
                    && left_ty.is_integer_type()
                    && right_ty.is_integer_type()
                {
                    left_ty

                } else {
                    generate_plain_error!(
                        "Cannot apply operator `{op}` for types: `{left}` and `{right}`",
                        op = op, left = left_ty, right = right_ty
                    )
                }
            }

            Expr::Unary (op, expr) => {
                let expr_ty = self.infer_type(*expr);

                if let UnaryOp::Minus = op && (expr_ty == Type::Int || expr_ty == Type::Float) {
                    expr_ty
                } else if let UnaryOp::Not = op && expr_ty.is_logical_type() {
                    expr_ty
                } else {
                    generate_plain_error!(
                        "Cannot apply unary operator `{op}` for type: `{_type}`",
                        op = op, _type = expr_ty
                    )
                }
            }

            Expr::Cast (to, expr) => {
                let expr_ty = self.infer_type(*expr);

                if !expr_ty.is_num_type() && expr_ty != Type::Char {
                    generate_plain_error!("Cannot cast {} to type `{}`", expr_ty, to);
                }

                if (expr_ty == Type::Char && *to == Type::Float) ||
                    (expr_ty == Type::Float && *to == Type::Char)
                {
                    generate_plain_error!("Cannot cast {} to type `{}`", expr_ty, to);
                }

                if !to.is_num_type() && *to != Type::Char {
                    generate_plain_error!("Cannot cast anything to type `{}`", to);
                }

                to.clone()
            }
        }
    }
}