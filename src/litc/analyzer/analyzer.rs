use std::collections::HashMap;

use crate::ast::*;
use crate::generate_error;

pub fn analyze(program: &Program) {
    for func in &program.funcs {
        let mut declared: HashMap<String, Type> = HashMap::new();

        for stmt in &func.body {
            match stmt {
                Stmt::VarDecl(v) => {
                    if declared.contains_key(&v.name) {
                        generate_error!("Variable `{}` is already declared", v.name);
                    }


                    let expr_type = infer_type(&program.expr_arena, v.expr_id, &declared);
                    check_compat(&v._type, &expr_type, &v.name, has_variables(&program.expr_arena, v.expr_id));

                    declared.insert(v.name.clone(), v._type.clone());
                }

                Stmt::Println(expr) => {
                    infer_type(&program.expr_arena, *expr, &declared);
                }
            }
        }
    }
}

fn has_variables(arena: &ExprArena, id: ExprId) -> bool {
    let expr_node = arena.get(id);
    let expr = &expr_node.expr;

    match expr {
        Expr::Lit(_) => false,
        Expr::Var(_) => true,
        Expr::Binary { left, right, .. } => has_variables(arena, *left) || has_variables(arena, *right),
        Expr::Unary { expr, .. } => has_variables(arena, *expr),
    }
}

fn infer_type(
    arena: &ExprArena,
    id: ExprId,
    declared: &HashMap<String, Type>,
) -> Type {
    let expr_node = arena.get(id);
    let expr = &expr_node.expr;
    
    match expr {
        Expr::Lit(Lit::Unt(_)) => Type::Unt,
        Expr::Lit(Lit::Int(_)) => Type::Int,
        Expr::Lit(Lit::Float(_)) => Type::Float,
        Expr::Lit(Lit::Bool(_)) => Type::Bool,
        Expr::Lit(Lit::Str(_)) => Type::Str,

        Expr::Var(name) => {
            declared.get(name).unwrap_or_else(|| {
                generate_error!("Variable `{}` is not declared", name);
            }).clone()
        }

        Expr::Binary { op, left, right} => {
            let left = infer_type(arena, *left, declared);
            let right = infer_type(arena, *right, declared);

            let resolved = resolve_binary_type(op, &left, &right);

            println!("Left: {}, Right: {}, New_type: {:?}", left, right, resolved);

            resolved.unwrap_or_else(|| {
                generate_error!("Cannot apply operator `{}` for types: `{}` and `{}`", op, left, right);
            })
        }

        Expr::Unary { op, expr } => {
            let expr_type = infer_type(arena, *expr, declared);
            
            let resolved = resolve_unary_type(&expr_type);

            resolved.unwrap_or_else(|| {
                generate_error!("Cannot apply unary operator `{}` for type: `{}`", op, expr_type);
            })
        }
    }
}

fn resolve_binary_type(op: &BinaryOp, left: &Type, right: &Type) -> Option<Type> {
    match op {
        op if op.is_comparison() => match_comparison(left, right),
        op if op.is_arranging() => match_arranging(left, right),
        op if op.is_logical() => match_logical(left, right),
        op if op.is_arithmetic() => match_arithmetic(left, right),

        _ => unreachable!(),
    }
}

fn match_comparison(left: &Type, right: &Type) -> Option<Type> {
    if left == right {
        Some(Type::Bool)
    } else {
        None
    }
}

fn match_arranging(left: &Type, right: &Type) -> Option<Type> {
    if left.is_num_type() && right.is_num_type() && left == right {
        Some(Type::Bool)
    } else {
        None
    }
}

fn match_logical(left: &Type, right: &Type) -> Option<Type> {
    match (left, right) {
        (Type::Bool, Type::Bool) => Some(Type::Bool),
        _ => None
    }
}

fn match_arithmetic(left: &Type, right: &Type) -> Option<Type> {
    let expr_type = numeric_tower(left, right);

    if expr_type.is_some() {
        Some(expr_type.unwrap())
    } else {
        None
    }
}

fn resolve_unary_type(expr: &Type) -> Option<Type> {
    if expr.is_num_type() || *expr == Type::Bool {
        Some(expr.clone())
    } else {
        None
    }
}

#[allow(dead_code)]
fn does_literal_num_fits_in(lit: &Type, var: &Type) -> bool {
    matches!(
        (var, lit),
        (Type::Unt, Type::Unt) |
        (Type::Int, Type::Unt) |
        (Type::Int, Type::Int) |
        (Type::Float, Type::Unt) |
        (Type::Float, Type::Int) |
        (Type::Float, Type::Float)
    )
}

fn numeric_tower(a: &Type, b: &Type) -> Option<Type> {
    match (a, b) {
        (Type::Unt, Type::Unt) => Some(Type::Unt),

        (Type::Int, Type::Unt) | (Type::Unt, Type::Int) |
        (Type::Int, Type::Int)
            => Some(Type::Int),

        (Type::Unt, Type::Float) | (Type::Float, Type::Unt) |
        (Type::Int, Type::Float) | (Type::Float, Type::Int) |
        (Type::Float, Type::Float)
            => Some(Type::Float),

        _ => None,
    }
}

fn check_compat(
    var_type: &Type,
    expr_type: &Type,
    var_name: &String,
    contains_var: bool,
) {
    let ok = if contains_var {
        var_type == expr_type
    } else {
        match (var_type, expr_type) {
            (a, b) if a == b => true,

            (Type::Int, Type::Unt) => true,
            (Type::Float, Type::Unt) => true,
            (Type::Float, Type::Int) => true,

            _ => false,
        }
    };

    if !ok {
        generate_error!(
            "Cannot assign {} value to variable `{}` of type `{}`",
            expr_type, var_name, var_type
        )
    }
}