use std::collections::HashMap;
use std::process;

use super::TypeSource;
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

fn infer_type(
    arena: &ExprArena,
    id: ExprId,
    declared: &HashMap<String, Type>
) -> TypeSource {
    match arena.get(id) {
        Expr::Lit(Lit::Unt(_)) => TypeSource::Lit(Type::Unt),
        Expr::Lit(Lit::Int(_)) => TypeSource::Lit(Type::Int),
        Expr::Lit(Lit::Float(_)) => TypeSource::Lit(Type::Float),
        Expr::Lit(Lit::Bool(_)) => TypeSource::Lit(Type::Bool),
        Expr::Lit(Lit::Str(_)) => TypeSource::Lit(Type::Str),

        Expr::Var(name) => {
            let _type = declared.get(name).unwrap_or_else(|| {
                generate_error!("Variable `{}` is not declared", name);
            }).clone();
            
            TypeSource::Var(_type)
        }

        Expr::Binary { op, left, right} => {
            let left = infer_type(arena, *left, declared);
            let right = infer_type(arena, *right, declared);

            let resolved = resolve_binary_type(&left, &right);

            resolved.unwrap_or_else(|| {
                generate_error!("Cannot apply operator `{}` for types: `{}` and `{}`", op, left.get_type(), right.get_type());
            })
        }
    }
}

fn resolve_binary_type(left: &TypeSource, right: &TypeSource) -> Option<TypeSource> {
    match (left, right) {
        (TypeSource::Var(a), TypeSource::Var(b)) => {
            if a == b && is_num_type(a) && is_num_type(b) {
                Some(TypeSource::Var(a.clone()))
            } else {
                None
            }
        }

        (TypeSource::Var(a), TypeSource::Lit(b)) |
        (TypeSource::Lit(b), TypeSource::Var(a))
            => {
            if does_literal_num_fits_in(b, a) {
                Some(TypeSource::Var(a.clone()))
            } else {
                None
            }
        }

        (TypeSource::Lit(a), TypeSource::Lit(b)) => {
            numeric_tower(a, b).map(TypeSource::Lit)
        }
    }
}

fn is_num_type(a: &Type) -> bool {
    !matches!(a, Type::Bool | Type::Str)
}

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
    expr_ts: &TypeSource,
    var_name: &String
) {
    let ok = match (var_type, expr_ts) {
        (a, TypeSource::Var(b)) if a == b => true,
        (a, TypeSource::Lit(b)) if a == b => true,

        (Type::Int, TypeSource::Lit(Type::Unt)) => true,
        (Type::Float, TypeSource::Lit(Type::Unt)) => true,
        (Type::Float, TypeSource::Lit(Type::Int)) => true,

        _ => false,
    };

    if !ok {
        generate_error!(
            "Cannot assign {} value to variable `{}` of type `{}`",
            expr_ts.get_type(), var_name, var_type
        )
    }
}

fn check_vars_declared(
    arena: &ExprArena,
    expr_id: ExprId,
    declared: &HashMap<String, Type>
) {
    match arena.get(expr_id) {
        Expr::Var(name) => {
            if !declared.contains_key(name) {
                generate_error!("Variable `{}` is not declared", name)
            }
        }

        Expr::Binary { op: _, left, right} => {
            check_vars_declared(arena, *left, declared);
            check_vars_declared(arena, *right, declared);
        }

        _ => {}
    }
}