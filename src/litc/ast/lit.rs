use std::fmt;

#[derive(Debug, Clone)]
pub enum Lit {
    Unt(u64),      // literal unsigned 64-bit integer
    Int(i64),      // literal signed 64-bit integer
    Float(f64),    // literal 64-bit number with floating point
    Bool(bool),    // literal boolean: `true` or `false`
    Str(String),   // literal string
}

impl fmt::Display for Lit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self { 
            Lit::Unt(u) => write!(f, "Unt({})", u),
            Lit::Int(i) => write!(f, "Int({})", i),
            Lit::Float(fl) => write!(f, "Float({})", fl),
            Lit::Bool(b) => write!(f, "Bool({})", b),
            Lit::Str(s) => write!(f, "Str(\"{}\")", s),
        }
    }
}