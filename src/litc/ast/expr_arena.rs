use super::expr::Expr;

pub type ExprId = usize;

#[derive(Debug)]
pub struct ExprArena {
    nodes: Vec<Expr>,
}

impl ExprArena {
    pub fn new() -> ExprArena {
        ExprArena {
            nodes: Vec::new(),
        }
    }

    pub fn add(&mut self, expr: Expr) -> ExprId {
        let id = self.nodes.len();

        self.nodes.push(expr);

        id
    }

    pub fn get(&self, id: ExprId) -> &Expr {
        &self.nodes.get(id).unwrap()
    }
}