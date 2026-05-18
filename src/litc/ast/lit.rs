#[derive(Debug, Clone)]
pub enum Lit {
    Unt(u64),
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
}