use std::collections::HashMap;
use crate::ast::*;

use super::Parser;

impl Parser {
    fn desugar(&mut self, program: &mut Program) {
        for func in &mut program.funcs {
            self.desugar_func(func, &mut program.expr_arena);
        }
    }

    fn desugar_func(&mut self, func: &mut FuncDef, expr_arena: &mut ExprArena) {
        let mut declared: HashMap<String, Type> = HashMap::new();

        for stmt in &mut func.body {
            self.desugar_stmt(stmt, expr_arena, &mut declared);
        }
    }

    fn desugar_stmt(&mut self, stmt: &mut Stmt, expr_arena: &mut ExprArena, declared: &mut HashMap<String, Type>) {
        match stmt {
            Stmt::VarDecl(v) => {
     //           self.desugar_expr(expr_arena, v.expr_id, &declared);

                declared.insert(v.name.clone(), v._type.clone());
            }
            Stmt::Println(expr_id) => {
       //         self.desugar_expr(expr_arena, *expr_id, &declared);
            }
        }
    }

    fn desugar_expr(&mut self, id: ExprId) {

    }
}