#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Unt,
    Int,
    Float,
    Bool,
    Str,
}

#[derive(Debug, Clone)]
pub enum Value {
    Unt(u64),
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
}

#[derive(Debug, Clone)]
pub enum Operand {
    Plus,
    Minus,
    Mul,
    Div,
    Rem,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Lit(Value),
    Var(String),
    BinOp { left: Box<Expr>, op: Operand, right: Box<Expr> },
}

#[derive(Debug)]
pub struct VarDecl {
    pub _type: Type,
    pub name: String,
    pub value: Value,
}

#[derive(Debug)]
pub enum PrintlnArg {
    StringLit(String),
    Var(String),
}

#[derive(Debug)]
pub enum Stmt {
    Println(PrintlnArg),
    VarDecl(VarDecl),
}

#[derive(Debug)]
pub struct  FuncDef {
    pub name: String,
    pub body: Vec<Stmt>,
}

#[derive(Debug)]
pub struct Program {
    pub funcs: Vec<FuncDef>,
}