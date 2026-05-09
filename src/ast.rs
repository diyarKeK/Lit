use std::fmt::Display;

#[derive(Debug)]
pub struct Program {
    pub funcs: Vec<FuncDef>,
    pub expr_arena: ExprArena,
}

#[derive(Debug)]
pub struct FuncDef {
    pub name: String,
    pub body: Vec<Stmt>,
}

#[derive(Debug)]
pub enum Stmt {
    Println(ExprId),
    VarDecl(VarDecl),
}

#[derive(Debug)]
pub struct VarDecl {
    pub _type: Type,
    pub name: String,
    pub value: ExprId,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Unt,
    Int,
    Float,
    Bool,
    Str,
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self { 
            Type::Unt => write!(f, "unt"),
            Type::Int => write!(f, "int"),
            Type::Float => write!(f, "float"),
            Type::Bool => write!(f, "bool"),
            Type::Str => write!(f, "str"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    Unt(u64),
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),

    Var(String),

    Binary { left: ExprId, op: Operand, right: ExprId },
}

#[derive(Debug, Clone)]
pub enum Operand {
    Plus,
    Minus,
    Mul,
    Div,
    Rem,
}

impl Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self { 
            Operand::Plus => write!(f, "+"),
            Operand::Minus => write!(f, "-"),
            Operand::Mul => write!(f, "*"),
            Operand::Div => write!(f, "/"),
            Operand::Rem => write!(f, "%"),
        }
    }
}

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