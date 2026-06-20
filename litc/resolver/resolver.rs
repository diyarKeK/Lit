use std::collections::HashMap;

use crate::ast::*;

pub fn resolve(program: &mut Program) {
    let mut resolver = Resolver::new(&mut program.expr_arena);

    for func in &mut program.funcs {
        resolver.resolve_func(func);
    }
}

struct Resolver<'a> {
    arena: &'a mut ExprArena,
    declared: HashMap<String, Type>,
}

impl<'a> Resolver<'a> {
    fn new(expr_arena: &'a mut ExprArena) -> Resolver<'a> {
        Resolver {
            arena: expr_arena,
            declared: HashMap::new(),
        }
    }

    fn resolve_func(&mut self, func: &mut FuncDef) {
        self.declared.clear();

        self.resolve_block(&mut func.body)
    }
    
    fn resolve_block(&mut self, block: &mut Block) {
        for stmt in block.mut_stmts() {
            match stmt {
                Stmt::VarDecl(v) => {
                    let (_, expr_type) = self.resolve_expr(v.expr_id);

                    if expr_type == Type::Unt && v._type != Type::Unt {
                        self.coerce_node_to(v.expr_id, &v._type);
                    }

                    self.declared.insert(v.name.clone(), v._type.clone());
                }
                Stmt::Println(expr_id) => {
                    self.resolve_expr(*expr_id);
                }
                _ => {}
            }
        }
    }

    fn resolve_expr(&mut self, id: ExprId) -> (Expr, Type) {
        let expr_node = self.arena.get(id);
        let expr = expr_node.expr.clone();

        use Lit::*;
        let (new_expr, current_type) = match expr {
            Expr::Lit(Unt(u)) => (Expr::Lit(Unt(u)), Type::Unt),
            Expr::Lit(Int(i)) => (Expr::Lit(Int(i)), Type::Int),
            Expr::Lit(Float(f)) => (Expr::Lit(Float(f)), Type::Float),
            Expr::Lit(Bool(b)) => (Expr::Lit(Bool(b)), Type::Bool),
            Expr::Lit(Char(c)) => (Expr::Lit(Char(c)), Type::Char),
            Expr::Lit(Str(s)) => (Expr::Lit(Str(s)), Type::Str),

            Expr::Var(ref name) => {
                let ty = self.declared.get(name).cloned().unwrap_or(Type::Unt);
                (Expr::Var(name.clone()), ty)
            }

            Expr::Binary (op, left, right) => {
                let (_, left_ty) = self.resolve_expr(left);
                let (_, right_ty) = self.resolve_expr(right);

                if left_ty.is_num_type() && right_ty.is_num_type() {
                    if let Some(target_ty) = Resolver::numeric_tower(&left_ty, &right_ty) {

                        if left_ty != target_ty { self.coerce_node_to(left, &target_ty); }
                        if right_ty != target_ty { self.coerce_node_to(right, &target_ty); }

                        let res_ty = if op.is_comparison() || op.is_arranging() {
                            Type::Bool
                        } else {
                            target_ty
                        };

                        (Expr::Binary(op, left, right), res_ty)
                    } else {
                        (Expr::Binary(op, left, right), Type::Unt)
                    }
                } else {
                    let res_ty = if op.is_comparison() {
                        Type::Bool
                    } else {
                        left_ty
                    };
                    (Expr::Binary(op, left, right), res_ty)
                }
            }

            Expr::Unary (op, expr) => {
                let (_, inner_ty) = self.resolve_expr(expr);

                if let UnaryOp::Minus = op && inner_ty == Type::Unt {
                    self.coerce_node_to(expr, &Type::Int);
                }

                (Expr::Unary(op, expr), inner_ty)
            }
            
            Expr::Cast (to, expr) => {
                self.resolve_expr(expr);
                
                // TODO: Хуйня, переделывай!
                
                /*if inner_ty == to {
                    let inner_node = self.arena.get(expr).clone();
                    self.arena.set(id, inner_node);

                    (inner_expr, inner_ty)
                } else {
                    if let Expr::Lit(lit) = &inner_expr {
                        let folded_expr = match (lit, &to) {
                            (Unt(u), Type::Int) => Some(Expr::Lit(Int(*u as i64))),
                            (Unt(u), Type::Float) => Some(Expr::Lit(Float(*u as f64))),
                            (Int(i), Type::Unt) => Some(Expr::Lit(Unt(*i as u64))),
                            (Int(i), Type::Float) => Some(Expr::Lit(Float(*i as f64))),
                            (Float(f), Type::Unt) => Some(Expr::Lit(Unt(*f as u64))),
                            (Float(f), Type::Int) => Some(Expr::Lit(Int(*f as i64))),
                            _ => None,
                        };

                        if let Some(new_lit) = folded_expr {
                            let mut node = self.arena.get(id).clone();
                            node.expr = new_lit.clone();
                            self.arena.set(id, node);

                            return (new_lit, to);
                        }
                    }
                }*/

                (Expr::Cast(to.clone(), expr), to)
            }
        };

        (new_expr, current_type)
    }

    fn coerce_node_to(&mut self, id: ExprId, target: &Type) {
        let mut node = self.arena.get(id).clone();

        match &mut node.expr {
            Expr::Lit(Lit::Unt(u)) => {
                if *target == Type::Int {
                    node.expr = Expr::Lit(Lit::Int(*u as i64));
                } else if *target == Type::Float {
                    node.expr = Expr::Lit(Lit::Float(*u as f64));
                }
            }
            Expr::Lit(Lit::Int(i)) => {
                if *target == Type::Float {
                    node.expr = Expr::Lit(Lit::Float(*i as f64));
                }
            }
            Expr::Binary (op, left, right)
                if op.is_arithmetic() || op.is_logical() || op.is_bitwise() => {
                self.coerce_node_to(*left, target);
                self.coerce_node_to(*right, target);
            }
            Expr::Unary (_, expr) => {
                self.coerce_node_to(*expr, target);
            }
            _ => {}
        }

        self.arena.set(id, node);
    }

    fn numeric_tower(a: &Type, b: &Type) -> Option<Type> {
        match (a, b) {
            (Type::Unt, Type::Unt) => Some(Type::Unt),

            (Type::Int, Type::Unt) | (Type::Unt, Type::Int)
            | (Type::Int, Type::Int)
            => Some(Type::Int),

            (Type::Unt, Type::Float) | (Type::Float, Type::Unt)
            | (Type::Int, Type::Float) | (Type::Float, Type::Int)
            | (Type::Float, Type::Float)
            => Some(Type::Float),

            _ => None,
        }
    }
}