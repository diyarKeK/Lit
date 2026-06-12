use crate::ast::Type;

#[derive(Debug, Clone, PartialEq)]
pub enum LlvmType {
    I64Unsigned,    // i64
    I64Signed,      // i64
    Double,         // double
    I1,             // i1
    I8,             // i8
    I8Ptr,          // i8*
}

impl LlvmType {
    pub fn from(ty: &Type) -> LlvmType {
        match ty {
            Type::Unt => LlvmType::I64Unsigned,
            Type::Int => LlvmType::I64Signed,
            Type::Float => LlvmType::Double,
            Type::Bool => LlvmType::I1,
            Type::Char => LlvmType::I8,
            Type::Str => LlvmType::I8Ptr,
        }
    }
    
    pub fn get_alloca_type(&self) -> &'static str {
        match self {
            LlvmType::I64Unsigned => "i64",
            LlvmType::I64Signed => "i64",
            LlvmType::Double => "double",
            LlvmType::I1 => "i1",
            LlvmType::I8 => "i8",
            LlvmType::I8Ptr => "i8*",
        }
    }
}