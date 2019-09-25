#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum VariableContext {
    Unknown,
    Global,
    Temporary(usize),
}

