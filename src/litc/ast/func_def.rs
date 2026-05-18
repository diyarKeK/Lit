use super::stmt::Stmt;

#[derive(Debug)]
pub struct FuncDef {
    pub name: String,
    pub body: Vec<Stmt>,
}