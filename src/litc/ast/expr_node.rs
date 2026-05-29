use super::Expr;
use crate::lexer::Span;

#[derive(Debug, Clone)]
pub struct ExprNode {
    pub expr: Expr,
    pub span: Span,
}

impl ExprNode {
    #[inline]
    pub fn new(expr: Expr, span: Span) -> ExprNode {
        ExprNode { expr, span }
    }
}