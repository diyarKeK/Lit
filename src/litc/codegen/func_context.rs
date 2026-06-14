use std::collections::HashMap;

use crate::ast::*;

pub struct FuncCtx {
    str_consts: Vec<String>,
    var_types: HashMap<String, Type>,
}

impl FuncCtx {
    pub fn build(func: &FuncDef, arena: &ExprArena) -> FuncCtx {
        let mut str_consts: Vec<String> = Vec::new();
        let mut var_types = HashMap::new();

        for stmt in &func.body {
            match stmt {
                Stmt::VarDecl(v) => {
                    var_types.insert(v.name.clone(), v._type.clone());

                    if let Expr::Lit(Lit::Str(s)) = arena.get(v.expr_id).expr.clone() {
                        str_consts.push(s.clone());
                    }
                }
                Stmt::Println(id) => {
                    if let Expr::Lit(Lit::Str(s)) = arena.get(*id).expr.clone() {
                        str_consts.push(s.clone());
                    }
                }
            }
        }

        FuncCtx { str_consts, var_types }
    }

    pub fn get_string_consts(&self) -> &[String] {
        &self.str_consts
    }

    pub fn get_var_types(&self) -> &HashMap<String, Type> {
        &self.var_types
    }
}