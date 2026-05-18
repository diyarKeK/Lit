use crate::ast::Type;

#[derive(Debug, Clone)]
pub enum TypeSource {
    Lit(Type),
    Var(Type),
}

impl TypeSource {
    pub fn get_type(&self) -> &Type {
        match self {
            TypeSource::Lit(t) => t,
            TypeSource::Var(t) => t,
        }
    }
}