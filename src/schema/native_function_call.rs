use std::convert::TryInto;
use super::{List, ListDefinitions, Value};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum NativeFunctionCall {
    Add,
    Subtract,
    Divide,
    Multiply,
    Mod,
    Negate,

    Equal,
    Greater,
    Less,
    GreaterThanOrEquals,
    LessThanOrEquals,
    NotEquals,
    Not,

    And,
    Or,

    Min,
    Max,

    Pow,
    Floor,
    Ceiling,
    Int,
    Float,

    Has,
    Hasnt,
    Intersect,

    ListMin,
    ListMax,
    All,
    Count,
    ValueOfList,
    Invert,
}

impl NativeFunctionCall {
    fn coerce(mut params: Vec<Value>) -> (Value, Value) {
        let a = params.pop().unwrap();
        let b = params.pop().unwrap();
        match (a, b) {
            // fancy types cannot coerce normally
            | (Value::VariablePointer(..), _)
            | (_, Value::VariablePointer(..)) => panic!("Variables should be resolved before calling a function"),
            (Value::DivertTarget(a), b) => (Value::DivertTarget(a), b),
            (a, Value::DivertTarget(b)) => (a, Value::DivertTarget(b)),
            (Value::List(a), b) => (Value::List(a), b),
            (a, Value::List(b)) => (a, Value::List(b)),
            // strings are highest priority
            (Value::String(a), b) => (Value::String(a), Value::String(b.try_into().unwrap())),
            (a, Value::String(b)) => (Value::String(a.try_into().unwrap()), Value::String(b)),
            // then floats
            (Value::Float(a), b) => (Value::Float(a), Value::Float(b.try_into().unwrap())),
            (a, Value::Float(b)) => (Value::Float(a.try_into().unwrap()), Value::Float(b)),
            // ints last
            (Value::Int(a), b) => (Value::Int(a), Value::Int(b.try_into().unwrap())),
            (a, Value::Int(b)) => (Value::Int(a.try_into().unwrap()), Value::Int(b)),
        }
    }

    pub fn number_of_parameters(&self) -> usize {
        use NativeFunctionCall::*;
        match self {
            | Add
            | Subtract
            | Divide
            | Multiply
            | Mod
            | Equal
            | Greater
            | Less
            | GreaterThanOrEquals
            | LessThanOrEquals
            | NotEquals
            | And
            | Or
            | Min
            | Max
            | Pow
            | Has
            | Hasnt
            | Intersect => 2,
            _ => 1,
        }
    }

    pub(crate) fn call(&self, lists: &ListDefinitions, params: Vec<Value>) -> Value {
        use NativeFunctionCall::*;
        match self {
            Add => {
                match Self::coerce(params) {
                    (Value::Int(a), Value::Int(b)) => return (a + b).into(),
                    (Value::Float(a), Value::Float(b)) => return (a + b).into(),
                    (Value::String(a), Value::String(b)) => return (a + &b).into(),
                    (Value::List(a), Value::List(b)) => return (a | &b).into(),
                    | (Value::List(a), Value::Int(b))
                    | (Value::Int(b), Value::List(a)) => return a.increment(b, lists).into(),
                    _ => {},
                }
            }
            Subtract => {
                match Self::coerce(params) {
                    (Value::Int(a), Value::Int(b)) => return (a - b).into(),
                    (Value::Float(a), Value::Float(b)) => return (a - b).into(),
                    (Value::List(a), Value::List(b)) => return (a - &b).into(),
                    | (Value::List(a), Value::Int(b))
                    | (Value::Int(b), Value::List(a)) => return a.increment(-b, lists).into(),
                    _ => {},
                }
            }
            Multiply => {
                match Self::coerce(params) {
                    (Value::Int(a), Value::Int(b)) => return (a * b).into(),
                    (Value::Float(a), Value::Float(b)) => return (a * b).into(),
                    _ => {},
                }
            }
            Divide => {
                match Self::coerce(params) {
                    (Value::Int(a), Value::Int(b)) => return (a / b).into(),
                    (Value::Float(a), Value::Float(b)) => return (a / b).into(),
                    _ => {},
                }
            }
            Mod => {
                match Self::coerce(params) {
                    (Value::Int(a), Value::Int(b)) => return (a % b).into(),
                    (Value::Float(a), Value::Float(b)) => return (a % b).into(),
                    _ => {},
                }
            }
            Negate => {
                match params[0] {
                    Value::Int(a) => return (-a).into(),
                    Value::Float(a) => return (-a).into(),
                    _ => {},
                }
            }
            Equal => {
                match Self::coerce(params) {
                    (Value::Int(a), Value::Int(b)) => return (a == b).into(),
                    (Value::Float(a), Value::Float(b)) => return (a == b).into(),
                    (Value::String(a), Value::String(b)) => return (a == b).into(),
                    (Value::List(a), Value::List(b)) => return (a == b).into(),
                    (Value::DivertTarget(a), Value::DivertTarget(b)) => return (a == b).into(),
                    _ => return false.into(),
                }
            }
            NotEquals => {
                match Self::coerce(params) {
                    (Value::Int(a), Value::Int(b)) => return (a != b).into(),
                    (Value::Float(a), Value::Float(b)) => return (a != b).into(),
                    (Value::String(a), Value::String(b)) => return (a != b).into(),
                    (Value::List(a), Value::List(b)) => return (a != b).into(),
                    (Value::DivertTarget(a), Value::DivertTarget(b)) => return (a != b).into(),
                    _ => return false.into(),
                }
            }
            Greater => {
                match Self::coerce(params) {
                    (Value::Int(a), Value::Int(b)) => return (a > b).into(),
                    (Value::Float(a), Value::Float(b)) => return (a > b).into(),
                    (Value::String(a), Value::String(b)) => return (a > b).into(),
                    (Value::List(a), Value::List(b)) => return (a.gt(&b)).into(),
                    _ => {},
                }
            }
            Less => {
                match Self::coerce(params) {
                    (Value::Int(a), Value::Int(b)) => return (a < b).into(),
                    (Value::Float(a), Value::Float(b)) => return (a < b).into(),
                    (Value::String(a), Value::String(b)) => return (a < b).into(),
                    (Value::List(a), Value::List(b)) => return (a.lt(&b)).into(),
                    _ => {},
                }
            }
            GreaterThanOrEquals => {
                match Self::coerce(params) {
                    (Value::Int(a), Value::Int(b)) => return (a >= b).into(),
                    (Value::Float(a), Value::Float(b)) => return (a >= b).into(),
                    (Value::String(a), Value::String(b)) => return (a >= b).into(),
                    (Value::List(a), Value::List(b)) => return (a.ge(&b)).into(),
                    _ => {},
                }
            }
            LessThanOrEquals => {
                match Self::coerce(params) {
                    (Value::Int(a), Value::Int(b)) => return (a <= b).into(),
                    (Value::Float(a), Value::Float(b)) => return (a <= b).into(),
                    (Value::String(a), Value::String(b)) => return (a <= b).into(),
                    (Value::List(a), Value::List(b)) => return (a.le(&b)).into(),
                    _ => {},
                }
            }
            Not => {
                match &params[0] {
                    &Value::Int(a) => return (a == 0).into(),
                    &Value::Float(a) => return (a == 0f64).into(),
                    &Value::String(ref a) => return a.is_empty().into(), // I think this one is non-standard but makes sense
                    &Value::List(ref a) => return a.is_empty().into(),
                    _ => {},
                }
            }
            And => {
                return (params[0].is_truthy() && params[1].is_truthy()).into()
            }
            Or => {
                return (params[0].is_truthy() || params[1].is_truthy()).into()
            }
            Max => {
                match Self::coerce(params) {
                    (Value::Int(a), Value::Int(b)) => return i64::max(a, b).into(),
                    (Value::Float(a), Value::Float(b)) => return f64::max(a, b).into(),
                    _ => {},
                }
            }
            Min => {
                match Self::coerce(params) {
                    (Value::Int(a), Value::Int(b)) => return i64::min(a, b).into(),
                    (Value::Float(a), Value::Float(b)) => return f64::min(a, b).into(),
                    _ => {},
                }
            }
            Pow => {
                match Self::coerce(params) {
                    (Value::Int(a), Value::Int(b)) => return (a as f64).powf(b as f64).into(),
                    (Value::Float(a), Value::Float(b)) => return a.powf(b).into(),
                    _ => {},
                }
            }
            Floor => {
                match &params[0] {
                    &Value::Int(a) => return a.into(),
                    &Value::Float(a) => return a.floor().into(),
                    _ => {},
                }
            }
            Ceiling => {
                match &params[0] {
                    &Value::Int(a) => return a.into(),
                    &Value::Float(a) => return a.ceil().into(),
                    _ => {},
                }
            }
            Int => {
                match &params[0] {
                    &Value::Int(a) => return a.into(),
                    &Value::Float(a) => return (a as i64).into(),
                    _ => {},
                }
            }
            Float => {
                match &params[0] {
                    &Value::Int(a) => return (a as f64).into(),
                    &Value::Float(a) => return a.into(),
                    _ => {},
                }
            }
            Has => {
                match Self::coerce(params) {
                    (Value::String(a), Value::String(b)) => return a.contains(&b).into(),
                    (Value::List(a), Value::List(b)) => return a.contains(&b).into(),
                    _ => {},
                }
            }
            Hasnt => {
                match Self::coerce(params) {
                    (Value::String(a), Value::String(b)) => return (!a.contains(&b)).into(),
                    (Value::List(a), Value::List(b)) => return (!a.contains(&b)).into(),
                    _ => {},
                }
            }
            Invert => {
                match &params[0] {
                    Value::List(a) => return a.invert(lists).into(),
                    _ => {},
                }
            }
            Intersect => {
                match Self::coerce(params) {
                    (Value::List(a), Value::List(b)) => return (a & &b).into(),
                    _ => {},
                }
            }
            ListMin => {
                match &params[0] {
                    Value::List(a) => return a.min()
                        .cloned()
                        .map(|entry| List::of_single_value_with_origins(entry, &a.origins))
                        .unwrap_or_else(|| List::default().with_empty_origins(&a.origins))
                        .into(),
                    _ => {},
                }
            }
            ListMax => {
                match &params[0] {
                    Value::List(a) => return a.max()
                        .cloned()
                        .map(|entry| List::of_single_value_with_origins(entry, &a.origins))
                        .unwrap_or_else(|| List::default().with_empty_origins(&a.origins))
                        .into(),
                    _ => {},
                }
            }
            All => {
                match &params[0] {
                    Value::List(a) => return lists.all_from_origins(&a.origins).into(),
                    _ => {},
                }
            }
            Count => {
                match &params[0] {
                    Value::List(a) => return (a.len() as i64).into(),
                    _ => {},
                }
            }
            ValueOfList => {
                match &params[0] {
                    Value::List(a) => return a.max().unwrap().value.into(),
                    _ => {},
                }
            }
        }
        panic!("Incompatible parameters passed to native function call: {:?}", self);
    }
}
