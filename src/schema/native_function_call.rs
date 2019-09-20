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
