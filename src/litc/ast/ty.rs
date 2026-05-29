use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Unt,      // unsigned 64-bit integer
    Int,      // signed 64-bit integer
    Float,    // 64-bit number with floating point
    Bool,     // boolean
    Str,      // string
}

impl Type {
    pub fn is_num_type(&self) -> bool {
        match self {
            Type::Unt |
            Type::Int |
            Type::Float
            => true,
            _ => false,
        }
    }
    
    pub fn is_integer_type(&self) -> bool {
        match self {
            Type::Unt |
            Type::Int
            => true,
            _ => false,
        }
    }
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