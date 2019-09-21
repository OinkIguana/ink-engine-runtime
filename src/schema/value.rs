use super::{List, Path};

#[derive(Clone, Debug)]
pub enum Value {
    Int(i64),
    Float(f64),
    List(List),
    String(String),
    
    DivertTarget(Path),
    VariablePointer(String),
}
