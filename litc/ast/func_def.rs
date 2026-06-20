use super::block::Block;

#[derive(Debug)]
pub struct FuncDef {
    pub name: String,
    pub body: Block,
}