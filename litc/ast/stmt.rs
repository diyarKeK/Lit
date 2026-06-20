use super::expr_arena::ExprId;
use super::var_decl::VarDecl;

#[derive(Debug)]
pub enum Stmt {
    VarDecl(VarDecl),       // `<type> <name> = <expression>`
    Println(ExprId),        // `println()`
    Unreachable,            // `unreachable`
}