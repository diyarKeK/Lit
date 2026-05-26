use std::fmt;

#[derive(Debug, Clone)]
pub enum BinaryOp {
    
    // arithmetic
    Add,    // `+`
    Sub,    // `-`
    Mul,    // `*`
    Div,    // `/`
    Mod,    // `%`
    
    // comparison
    EqEq,   // `==`
    NotEq,  // `!=`
    Gt,     // `>`
    Lt,     // `<`
    GtEq,   // `>=`
    LtEq,   // `<=`
    
    // logical
    And,    // `&`
    Or,     // `|`
    Xor,    // `^`
}

impl BinaryOp {
    pub fn is_arithmetic(&self) -> bool {
        match self { 
            BinaryOp::Add |
            BinaryOp::Sub |
            BinaryOp::Mul | 
            BinaryOp::Div |
            BinaryOp::Mod
            => true,
            _ => false,
        }
    }
    
    pub fn is_comparison(&self) -> bool {
        match self {
            BinaryOp::EqEq |
            BinaryOp::NotEq
            => true,
            _ => false,
        }
    }
    
    pub fn is_arranging(&self) -> bool {
        match self {
            BinaryOp::Gt |
            BinaryOp::Lt |
            BinaryOp::GtEq |
            BinaryOp::LtEq
            => true,
            _ => false,
        }
    }
    
    pub fn is_logical(&self) -> bool {
        match self {
            BinaryOp::And |
            BinaryOp::Or |
            BinaryOp::Xor
            => true,
            _ => false,
        }
    }
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinaryOp::Add => write!(f, "+"),
            BinaryOp::Sub => write!(f, "-"),
            BinaryOp::Mul => write!(f, "*"),
            BinaryOp::Div => write!(f, "/"),
            BinaryOp::Mod => write!(f, "%"),
            BinaryOp::EqEq => write!(f, "=="),
            BinaryOp::NotEq => write!(f, "!="),
            BinaryOp::Gt => write!(f, ">"),
            BinaryOp::Lt => write!(f, "<"),
            BinaryOp::GtEq => write!(f, ">="),
            BinaryOp::LtEq => write!(f, "<="),
            BinaryOp::And => write!(f, "&"),
            BinaryOp::Or => write!(f, "|"),
            BinaryOp::Xor => write!(f, "^"),
        }
    }
}