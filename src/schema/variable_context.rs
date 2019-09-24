#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum VariableContext {
    Global,
    Temporary(usize),
}

