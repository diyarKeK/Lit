use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Unt,
    Int,
    Float,
    Bool,
    Str,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Type::Unt => write!(f, "unt"),
            Type::Int => write!(f, "int"),
            Type::Float => write!(f, "float"),
            Type::Bool => write!(f, "bool"),
            Type::Str => write!(f, "str"),
        }
    }
}