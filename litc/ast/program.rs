use super::func_def::FuncDef;
use super::expr_arena::ExprArena;

#[derive(Debug)]
pub struct Program {
    pub funcs: Vec<FuncDef>,
    pub expr_arena: ExprArena,
}