use std::collections::HashMap;
use std::process;

use crate::analyzer::TypeSource as TS;
use crate::ast::*;
use crate::generate_error;

#[derive(Debug, Clone)]
enum TypeSource {
    Lit(Type),
    Var(Type),
}

impl TypeSource {
    fn get_type(&self) -> &Type {
        match self {
            TS::Lit(t) => t,
            TS::Var(t) => t,
        }
    }
}

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

fn infer_type(
    arena: &ExprArena,
    id: ExprId,
    declared: &HashMap<String, Type>
) -> TypeSource {
    match arena.get(id) {
        Expr::Unt(_) => TS::Lit(Type::Unt),
        Expr::Int(_) => TS::Lit(Type::Int),
        Expr::Float(_) => TS::Lit(Type::Float),
        Expr::Bool(_) => TS::Lit(Type::Bool),
        Expr::Str(_) => TS::Lit(Type::Str),

        Expr::Var(name) => {
            let _type = declared.get(name).unwrap_or_else(|| {
                generate_error!("Variable '{}' is not declared", name);
            }).clone();
            
            TS::Var(_type)
        }

        Expr::Binary { left, op, right} => {
            let left = infer_type(arena, *left, declared);
            let right = infer_type(arena, *right, declared);

            let resolved = resolve_binary_type(&left, &right);

            resolved.unwrap_or_else(|| {
                generate_error!("Cannot apply operator '{}' for '{}' and '{}'", op, left.get_type(), right.get_type());
            })
        }
    }
}

fn resolve_binary_type(left: &TypeSource, right: &TypeSource) -> Option<TypeSource> {
    match (left, right) {
        (TS::Var(a), TS::Var(b)) => {
            if a == b {
                Some(TS::Var(a.clone()))
            } else {
                None
            }
        }

        (TS::Var(a), TS::Lit(b)) |
        (TS::Lit(b), TS::Var(a))
            => {
            if does_literal_num_fits_in(b, a) {
                Some(TS::Var(a.clone()))
            } else {
                None
            }
        }

        (TS::Lit(a), TS::Lit(b)) => {
            numeric_tower(a, b).map(TS::Lit)
        }
    }
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
        (a, b) if a == b => Some(a.clone()),

        (Type::Int, Type::Unt) | (Type::Unt, Type::Int)
            => Some(Type::Int),

        (Type::Unt, Type::Float) | (Type::Float, Type::Unt) |
        (Type::Int, Type::Float) | (Type::Float, Type::Int)
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
        (a, TS::Var(b)) if a == b => true,
        (a, TS::Lit(b)) if a == b => true,

        (Type::Int, TS::Lit(Type::Unt)) => true,
        (Type::Float, TS::Lit(Type::Unt)) => true,
        (Type::Float, TS::Lit(Type::Int)) => true,

        _ => false,
    };

    if !ok {
        generate_error!(
            "Cannot assign '{}' to variable '{}' of type '{}'",
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