#[derive(Copy, Clone, Debug)]
pub enum VariableContext {
    Global,
    Temporary(usize),
}

