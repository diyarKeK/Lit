#[derive(Debug, Clone)]
pub enum Lit {
    Unt(u64),      // literal unsigned 64-bit integer
    Int(i64),      // literal signed 64-bit integer
    Float(f64),    // literal 64-bit number with floating point
    Bool(bool),    // literal boolean: `true` or `false`
    Str(String),   // literal string
}