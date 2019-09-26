use std::convert::TryInto;
use super::{Object, List, Path};

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
    fn unpack_binary_params<T>(params: &[Object]) -> Option<(T, T)>
    where Object: TryInto<T> {
        match params {
            [a, b] => Some((a.clone().try_into().ok()?, b.clone().try_into().ok()?)),
            _ => None
        }
    }

    fn unpack_unary_params<T>(params: &[Object]) -> Option<T>
    where Object: TryInto<T> {
        match params {
            [a] => a.clone().try_into().ok(),
            _ => None
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

    pub(crate) fn call(&self, params: Vec<Object>) -> Object {
        use NativeFunctionCall::*;
        match self {
            Add => {
                if let Some((a, b)) = Self::unpack_binary_params::<i64>(&params) { return Object::Value((a + b).into()) }
                if let Some((a, b)) = Self::unpack_binary_params::<f64>(&params) { return Object::Value((a + b).into()) }
                if let Some((a, b)) = Self::unpack_binary_params::<String>(&params) { return Object::Value((a + &b).into()) }
                if let Some((a, b)) = Self::unpack_binary_params::<List>(&params) { return Object::Value((a | &b).into()) }
            }
            Subtract => {
                if let Some((a, b)) = Self::unpack_binary_params::<i64>(&params) { return Object::Value((a - b).into()) }
                if let Some((a, b)) = Self::unpack_binary_params::<f64>(&params) { return Object::Value((a - b).into()) }
                if let Some((a, b)) = Self::unpack_binary_params::<List>(&params) { return Object::Value((a - &b).into()) }
            }
            Multiply => {
                if let Some((a, b)) = Self::unpack_binary_params::<i64>(&params) { return Object::Value((a * b).into()) }
                if let Some((a, b)) = Self::unpack_binary_params::<f64>(&params) { return Object::Value((a * b).into()) }
            }
            Divide => {
                if let Some((a, b)) = Self::unpack_binary_params::<i64>(&params) { return Object::Value((a / b).into()) }
                if let Some((a, b)) = Self::unpack_binary_params::<f64>(&params) { return Object::Value((a / b).into()) }
            }
            Mod => {
                if let Some((a, b)) = Self::unpack_binary_params::<i64>(&params) { return Object::Value((a % b).into()) }
                if let Some((a, b)) = Self::unpack_binary_params::<f64>(&params) { return Object::Value((a % b).into()) }
            }
            Negate => {
                if let Some(a) = Self::unpack_unary_params::<i64>(&params) { return Object::Value((-a).into()) }
                if let Some(a) = Self::unpack_unary_params::<f64>(&params) { return Object::Value((-a).into()) }
            }
            Equal => {
                if let Some((a, b)) = Self::unpack_binary_params::<i64>(&params) { return Object::Value((a == b).into()) }
                if let Some((a, b)) = Self::unpack_binary_params::<f64>(&params) { return Object::Value((a == b).into()) }
                if let Some((a, b)) = Self::unpack_binary_params::<String>(&params) { return Object::Value((a == b).into()) }
                if let Some((a, b)) = Self::unpack_binary_params::<List>(&params) { return Object::Value((a == b).into()) }
                if let Some((a, b)) = Self::unpack_binary_params::<Path>(&params) { return Object::Value((a == b).into()) }
            }
            NotEquals => {
                if let Some((a, b)) = Self::unpack_binary_params::<i64>(&params) { return Object::Value((a != b).into()) }
                if let Some((a, b)) = Self::unpack_binary_params::<f64>(&params) { return Object::Value((a != b).into()) }
                if let Some((a, b)) = Self::unpack_binary_params::<String>(&params) { return Object::Value((a != b).into()) }
                if let Some((a, b)) = Self::unpack_binary_params::<List>(&params) { return Object::Value((a != b).into()) }
                if let Some((a, b)) = Self::unpack_binary_params::<Path>(&params) { return Object::Value((a != b).into()) }
            }
            Greater => {
                if let Some((a, b)) = Self::unpack_binary_params::<i64>(&params) { return Object::Value((a > b).into()) }
                if let Some((a, b)) = Self::unpack_binary_params::<f64>(&params) { return Object::Value((a > b).into()) }
                if let Some((a, b)) = Self::unpack_binary_params::<String>(&params) { return Object::Value((a > b).into()) }
                if let Some((a, b)) = Self::unpack_binary_params::<List>(&params) { return Object::Value((a.gt(&b)).into()) }
            }
            Less => {
                if let Some((a, b)) = Self::unpack_binary_params::<i64>(&params) { return Object::Value((a < b).into()) }
                if let Some((a, b)) = Self::unpack_binary_params::<f64>(&params) { return Object::Value((a < b).into()) }
                if let Some((a, b)) = Self::unpack_binary_params::<String>(&params) { return Object::Value((a < b).into()) }
                if let Some((a, b)) = Self::unpack_binary_params::<List>(&params) { return Object::Value((a.lt(&b)).into()) }
            }
            GreaterThanOrEquals => {
                if let Some((a, b)) = Self::unpack_binary_params::<i64>(&params) { return Object::Value((a >= b).into()) }
                if let Some((a, b)) = Self::unpack_binary_params::<f64>(&params) { return Object::Value((a >= b).into()) }
                if let Some((a, b)) = Self::unpack_binary_params::<String>(&params) { return Object::Value((a >= b).into()) }
                if let Some((a, b)) = Self::unpack_binary_params::<List>(&params) { return Object::Value((a.ge(&b)).into()) }
            }
            LessThanOrEquals => {
                if let Some((a, b)) = Self::unpack_binary_params::<i64>(&params) { return Object::Value((a <= b).into()) }
                if let Some((a, b)) = Self::unpack_binary_params::<f64>(&params) { return Object::Value((a <= b).into()) }
                if let Some((a, b)) = Self::unpack_binary_params::<String>(&params) { return Object::Value((a <= b).into()) }
                if let Some((a, b)) = Self::unpack_binary_params::<List>(&params) { return Object::Value((a.le(&b)).into()) }
            }
            Not => {
                if let Some(a) = Self::unpack_unary_params::<i64>(&params) { return Object::Value((a == 0).into()) }
                if let Some(a) = Self::unpack_unary_params::<f64>(&params) { return Object::Value((a == 0f64).into()) }
                if let Some(a) = Self::unpack_unary_params::<List>(&params) { return Object::Value(a.is_empty().into()) }
            }
            And => {
                if let Some((a, b)) = Self::unpack_binary_params::<i64>(&params) { return Object::Value((a != 0 && b != 0).into()) }
                if let Some((a, b)) = Self::unpack_binary_params::<f64>(&params) { return Object::Value((a != 0.0 && b != 0.0).into()) }
                if let Some((a, b)) = Self::unpack_binary_params::<List>(&params) { return Object::Value((!a.is_empty() && !b.is_empty()).into()) }
            }
            Or => {
                if let Some((a, b)) = Self::unpack_binary_params::<i64>(&params) { return Object::Value((a != 0 || b != 0).into()) }
                if let Some((a, b)) = Self::unpack_binary_params::<f64>(&params) { return Object::Value((a != 0.0 || b != 0.0).into()) }
                if let Some((a, b)) = Self::unpack_binary_params::<List>(&params) { return Object::Value((!a.is_empty() || !b.is_empty()).into()) }
            }
            Max => {
                if let Some((a, b)) = Self::unpack_binary_params::<i64>(&params) { return Object::Value(i64::max(a, b).into()) }
                if let Some((a, b)) = Self::unpack_binary_params::<f64>(&params) { return Object::Value(f64::max(a, b).into()) }
            }
            Min => {
                if let Some((a, b)) = Self::unpack_binary_params::<i64>(&params) { return Object::Value(i64::min(a, b).into()) }
                if let Some((a, b)) = Self::unpack_binary_params::<f64>(&params) { return Object::Value(f64::min(a, b).into()) }
            }
            Pow => {
                if let Some((a, b)) = Self::unpack_binary_params::<i64>(&params) { return Object::Value((a as f64).powf(b as f64).into()) }
                if let Some((a, b)) = Self::unpack_binary_params::<f64>(&params) { return Object::Value(a.powf(b).into()) }
            }
            Floor => {
                if let Some(a) = Self::unpack_unary_params::<i64>(&params) { return Object::Value(a.into()) }
                if let Some(a) = Self::unpack_unary_params::<f64>(&params) { return Object::Value(a.floor().into()) }
            }
            Ceiling => {
                if let Some(a) = Self::unpack_unary_params::<i64>(&params) { return Object::Value(a.into()) }
                if let Some(a) = Self::unpack_unary_params::<f64>(&params) { return Object::Value(a.ceil().into()) }
            }
            Int => {
                if let Some(a) = Self::unpack_unary_params::<i64>(&params) { return Object::Value(a.into()) }
                if let Some(a) = Self::unpack_unary_params::<f64>(&params) { return Object::Value((a as i64).into()) }
            }
            Float => {
                if let Some(a) = Self::unpack_unary_params::<i64>(&params) { return Object::Value((a as f64).into()) }
                if let Some(a) = Self::unpack_unary_params::<f64>(&params) { return Object::Value(a.into()) }
            }
            _ => unimplemented!("WIP"),
        }
        panic!("Incompatible parameters passed to native function call: {:?}", self);
    }
}
