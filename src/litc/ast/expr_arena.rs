use super::expr_node::ExprNode;

pub type ExprId = usize;

#[derive(Debug)]
pub struct ExprArena {
    nodes: Vec<ExprNode>,
}

impl ExprArena {
    pub fn new() -> ExprArena {
        ExprArena {
            nodes: Vec::new(),
        }
    }

    pub fn add(&mut self, node: ExprNode) -> ExprId {
        let id = self.nodes.len();

        self.nodes.push(node);

        id
    }

    pub fn get(&self, id: ExprId) -> &ExprNode {
        &self.nodes.get(id).unwrap()
    }
}