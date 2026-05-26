use std::fmt;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum UnaryOp {
    Minus,  // `-`
    Not,    // `!`
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnaryOp::Minus => write!(f, "-"),
            UnaryOp::Not => write!(f, "!"),
        }
    }
}