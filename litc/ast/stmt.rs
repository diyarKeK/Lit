use super::expr_arena::ExprId;
use super::var_decl::VarDecl;

#[derive(Debug)]
pub enum Stmt {
    Println(ExprId),     // `println()`
    VarDecl(VarDecl),    // `<type> <name> = <expression>`
}