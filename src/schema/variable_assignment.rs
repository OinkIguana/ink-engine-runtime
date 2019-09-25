#[derive(Clone, Debug)]
pub struct VariableAssignment {
    pub(crate) variable_name: String,
    pub(crate) is_new_declaration: bool,
    pub(crate) is_global: bool,
}
