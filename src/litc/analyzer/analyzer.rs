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


                    let expr_type = infer_type(&program.expr_arena, v.expr_id, &declared);
                    check_compat(&v._type, &expr_type, &v.name);

                    declared.insert(v.name.clone(), v._type.clone());
                }

                Stmt::Println(expr) => {
                    infer_type(&program.expr_arena, *expr, &declared);
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
    let expr_node = arena.get(id);
    let expr = &expr_node.expr;
    
    match expr {
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
        
        Expr::Unary { op, expr } => {
            let expr_type = infer_type(arena, *expr, declared);
            
            let resolved = resolve_unary_type(&expr_type);
            
            resolved.unwrap_or_else(|| {
                generate_error!("Cannot apply unary operator `{}` for type: `{}`", op, expr_type.get_type());
            })
        }
    }
}

fn resolve_unary_type(expr: &TypeSource) -> Option<TypeSource> {
    match expr {
        TypeSource::Var(a) => {
            if is_num_type(a) {
                Some(TypeSource::Var(a.clone()))
            } else {
                None
            }
        }
        
        TypeSource::Lit(a) => {
            if is_num_type(a) {
                Some(TypeSource::Lit(a.clone()))
            } else {
                None
            }
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
    matches!(a, Type::Unt | Type::Int | Type::Float)
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