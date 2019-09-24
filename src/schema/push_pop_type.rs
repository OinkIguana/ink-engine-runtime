use std::fmt::{self, Display, Formatter};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum PushPopType {
    Tunnel,
    Function,
    FunctionEvaluationFromGame,
}

impl Display for PushPopType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            PushPopType::Tunnel => write!(f, "tunnel"),
            PushPopType::Function => write!(f, "function"),
            PushPopType::FunctionEvaluationFromGame => write!(f, "function evaluation from game"),
        }
    }
}
