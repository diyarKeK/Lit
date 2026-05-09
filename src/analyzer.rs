use std::collections::HashMap;
use std::process;

use crate::ast::*;
use crate::generate_error;

pub fn analyze(program: &Program) {
    for func in &program.funcs {
        let mut declared: HashMap<String, Type> = HashMap::new();

        for stmt in &func.body {
            match stmt {
                Stmt::VarDecl(v) => {
                    if declared.contains_key(&v.name) {
                        generate_error!("Variable '{}' is already declared", v.name);
                    }

                    let expr_type = infer_type(&program.expr_arena, v.value, &declared);
                    check_compat(&v._type, &expr_type, &v.name);

                    declared.insert(v.name.clone(), v._type.clone());
                }

                Stmt::Println(expr) => {
                    check_vars_declared(&program.expr_arena, *expr, &declared);
                }
            }
        }
    }
}

fn infer_type(arena: &ExprArena, expr_id: ExprId, declared: &HashMap<String, Type>) -> Type {
    match arena.get(expr_id) {
        Expr::Unt(_) => Type::Unt,
        Expr::Int(_) => Type::Int,
        Expr::Float(_) => Type::Float,
        Expr::Bool(_) => Type::Bool,
        Expr::Str(_) => Type::Str,

        Expr::Var(name) => {
            declared.get(name).unwrap_or_else(|| {
                generate_error!("Variable '{}' is not declared", name);
            }).clone()
        }

        Expr::Binary { left, op, right} => {
            let left = infer_type(arena, *left, declared);
            let right = infer_type(arena, *right, declared);

            let resolved = resolve_binary_type(&left, &right);

            resolved.unwrap_or_else(|| {
                generate_error!("Cannot apply operator '{}' for '{}' and '{}'", op, left, right);
            })
        }
    }
}

fn resolve_binary_type(left_type: &Type, right_type: &Type) -> Option<Type> {
    match (left_type, right_type) {
        (a, b) if a == b => Some(a.clone()),

        (Type::Unt, Type::Int) | (Type::Int, Type::Unt)
            => Some(Type::Int),

        (Type::Unt, Type::Float) | (Type::Float, Type::Unt) |
        (Type::Int, Type::Float) | (Type::Float, Type::Int)
            => Some(Type::Float),

        _ => None,
    }
}

fn check_compat(var_type: &Type, expr_type: &Type, var_name: &String) {
    let ok = match (var_type, expr_type) {
        (a, b) if a == b => true,

        (Type::Int, Type::Unt) => true,
        (Type::Float, Type::Unt) => true,
        (Type::Float, Type::Int) => true,

        _ => false,
    };

    if !ok {
        generate_error!(
            "Cannot assign '{}' to variable '{}' of type '{}'",
            expr_type, var_name, var_type
        )
    }
}

fn check_vars_declared(arena: &ExprArena, expr_id: ExprId, declared: &HashMap<String, Type>) {
    match arena.get(expr_id) {
        Expr::Var(name) => {
            if !declared.contains_key(name) {
                generate_error!("Variable '{}' is not declared", name)
            }
        }

        Expr::Binary { left, op: _, right} => {
            check_vars_declared(arena, *left, declared);
            check_vars_declared(arena, *right, declared);
        }

        _ => {}
    }
}