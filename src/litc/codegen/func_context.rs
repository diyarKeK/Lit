use std::collections::HashMap;

use super::LlvmType;
use crate::ast::*;
use crate::codegen::codegen::infer_llvm_type;

pub struct FuncCtx {
    str_consts: Vec<String>,
    num_fmts: Vec<String>,
    var_types: HashMap<String, Type>,
}

impl FuncCtx {
    pub fn build(func: &FuncDef, arena: &ExprArena) -> FuncCtx {
        let mut str_consts: Vec<String> = Vec::new();
        let mut num_fmts = Vec::new();
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

        for stmt in &func.body {
            if let Stmt::Println(expr_id) = stmt {
                let _type = infer_llvm_type(arena, *expr_id, &var_types);

                let fmt = match _type {
                    LlvmType::I64Unsigned => Some("%llu\n"),
                    LlvmType::I64Signed => Some("%lld\n"),
                    LlvmType::Double => Some("%g\n"),
                    _ => None,
                };

                if let Some(f) = fmt {
                    num_fmts.push(f.to_string());
                }
            }
        }

        FuncCtx { str_consts, num_fmts, var_types }
    }

    pub fn get_string_consts(&self) -> &[String] {
        &self.str_consts
    }

    pub fn get_num_fmts(&self) -> &[String] {
        &self.num_fmts
    }

    pub fn get_var_types(&self) -> &HashMap<String, Type> {
        &self.var_types
    }
}