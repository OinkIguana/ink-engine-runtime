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

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Self::Int(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

// TODO: settle on some representation for the other types that is good enough to pass out to
// external functions in the future. For now, these will suffice as a proof of concept.

impl super::TryAsRef<i64> for Value {
    fn try_as_ref(&self) -> Option<&i64> {
        match self {
            Value::Int(ref value) => Some(value),
            _ => None,
        }
    }
}

impl super::TryAsRef<f64> for Value {
    fn try_as_ref(&self) -> Option<&f64> {
        match self {
            Value::Float(ref value) => Some(value),
            _ => None,
        }
    }
}

impl super::TryAsRef<String> for Value {
    fn try_as_ref(&self) -> Option<&String> {
        match self {
            Value::String(ref value) => Some(value),
            _ => None,
        }
    }
}
