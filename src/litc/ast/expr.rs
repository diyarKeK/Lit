use crate::ast::bin_op::BinaryOp;
use crate::ast::un_op::UnaryOp;
use super::expr_arena::ExprId;
use super::lit::Lit;

#[derive(Debug, Clone)]
pub enum Expr {
    Lit(Lit), // literal
    Var(String), // variable
    
    Binary { op: BinaryOp, left: ExprId, right: ExprId }, // binary action, e.g. `2 + 2 * 2`
    Unary { op: UnaryOp, expr: ExprId }, // unary action, e.g. `-a` where `a` is a variable
}