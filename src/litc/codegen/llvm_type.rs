#[derive(Debug, Clone, PartialEq)]
pub enum LlvmType {
    I64Unsigned,
    I64Signed,
    Double,
    I1,
    I8Ptr
}

impl LlvmType {
    pub fn get_alloca_type(&self) -> &'static str {
        match self {
            LlvmType::I64Unsigned => "i64",
            LlvmType::I64Signed => "i64",
            LlvmType::Double => "double",
            LlvmType::I1 => "i1",
            LlvmType::I8Ptr => "i8*",
        }
    }
}