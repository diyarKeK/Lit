use std::collections::HashMap;
use crate::ast::*;

pub fn desugar(program: &mut Program) {
    let mut desugar = Desugar::new(&mut program.expr_arena);

    for func in &mut program.funcs {
        desugar.desugar_func(func);
    }
}

struct Desugar<'a> {
    arena: &'a mut ExprArena,
    declared: HashMap<String, Type>,
}

impl<'a> Desugar<'a> {
    fn new(expr_arena: &'a mut ExprArena) -> Desugar<'a> {
        Desugar {
            arena: expr_arena,
            declared: HashMap::new(),
        }
    }

    fn desugar_func(&mut self, func: &mut FuncDef) {
        self.declared.clear();

        for stmt in &mut func.body {
            match stmt {
                Stmt::VarDecl(v) => {
                    let (_, expr_type) = self.desugar_expr(v.expr_id);

                    if expr_type == Type::Unt && v._type != Type::Unt {
                        self.coerce_node_to(v.expr_id, &v._type);
                    }

                    self.declared.insert(v.name.clone(), v._type.clone());
                }
                Stmt::Println(expr_id) => {
                    self.desugar_expr(*expr_id);
                }
            }
        }
    }

    fn desugar_expr(&mut self, id: ExprId) -> (Expr, Type) {
        let expr_node = self.arena.get(id);
        let expr = expr_node.expr.clone();

        use Lit::*;
        let (new_expr, current_type) = match expr {
            Expr::Lit(Unt(u)) => (Expr::Lit(Unt(u)), Type::Unt),
            Expr::Lit(Int(u)) => (Expr::Lit(Int(u)), Type::Int),
            Expr::Lit(Float(u)) => (Expr::Lit(Float(u)), Type::Float),
            Expr::Lit(Bool(u)) => (Expr::Lit(Bool(u)), Type::Bool),
            Expr::Lit(Str(s)) => (Expr::Lit(Str(s)), Type::Str),

            Expr::Var(ref name) => {
                let ty = self.declared.get(name).cloned().unwrap_or(Type::Unt);
                (Expr::Var(name.clone()), ty)
            }

            Expr::Binary { op, left, right } => {
                let (_, left_ty) = self.desugar_expr(left);
                let (_, right_ty) = self.desugar_expr(right);

                if left_ty.is_num_type() && right_ty.is_num_type() {
                    if let Some(target_ty) = Desugar::numeric_tower(&left_ty, &right_ty) {

                        if left_ty != target_ty { self.coerce_node_to(left, &target_ty); }
                        if right_ty != target_ty { self.coerce_node_to(right, &target_ty); }

                        let res_ty = if op.is_comparison() || op.is_arranging() {
                            Type::Bool
                        } else {
                            target_ty
                        };

                        (Expr::Binary { op, left, right }, res_ty)
                    } else {
                        (Expr::Binary { op, left, right }, Type::Unt)
                    }
                } else {
                    let res_ty = if op.is_comparison() {
                        Type::Bool
                    } else {
                        left_ty
                    };
                    (Expr::Binary { op, left, right }, res_ty)
                }
            }

            Expr::Unary { op, expr } => {
                let (_, inner_ty) = self.desugar_expr(expr);
                (Expr::Unary { op, expr }, inner_ty)
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
            Expr::Binary { op, left, right }
                if op.is_arithmetic() || op.is_logical() => {
                self.coerce_node_to(*left, target);
                self.coerce_node_to(*right, target);
            }
            Expr::Unary { expr, .. } => {
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