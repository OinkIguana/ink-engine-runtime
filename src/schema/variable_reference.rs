#[derive(Clone, Debug)]
pub struct VariableReference {
    name: String,
}

impl VariableReference {
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}
