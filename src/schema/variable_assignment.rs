#[derive(Clone, Debug)]
pub struct VariableAssignment {
    variable_name: String,
    is_new_declaration: bool,
    is_global: bool,
}
