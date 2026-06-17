use super::expr_arena::ExprId;
use super::ty::Type;

#[derive(Debug)]
pub struct VarDecl {
    pub _type: Type,
    pub name: String,
    pub expr_id: ExprId,
}