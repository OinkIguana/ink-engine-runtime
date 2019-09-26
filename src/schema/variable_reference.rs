use super::Path;

#[derive(Clone, Debug)]
pub enum VariableReference {
    Variable(String),
    PathForCount(Path),
}
