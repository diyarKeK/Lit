use super::stmt::Stmt;

#[derive(Debug)]
pub struct Block {
    stmts: Vec<Stmt>,
}

impl Block {
    pub fn new(stmts: Vec<Stmt>) -> Block {
        Block { stmts }
    }
    
    pub fn stmts(&self) -> &Vec<Stmt> {
        &self.stmts
    }
    
    pub fn mut_stmts(&mut self) -> &mut Vec<Stmt> {
        &mut self.stmts
    }
}