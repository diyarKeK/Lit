use crate::ast::bin_op::BinaryOp;
use crate::ast::un_op::UnaryOp;
use super::expr_arena::ExprId;
use super::lit::Lit;

#[derive(Debug, Clone)]
pub enum Expr {
    Lit(Lit),
    Var(String),
    
    Binary { op: BinaryOp, left: ExprId, right: ExprId },
    Unary { op: UnaryOp, expr: ExprId },
}