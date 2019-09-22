use super::{Path, PushPopType};

#[derive(Clone, Debug)]
pub struct Divert {
    pub(crate) target: DivertTarget,
    pub(crate) pushes_to_stack: bool,
    pub(crate) stack_push_type: PushPopType,
    pub(crate) is_conditional: bool,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum DivertTarget {
    Path(Path),
    Variable(String),
    External { path: String, args: i32 },
}

impl std::cmp::PartialEq for Divert {
    fn eq(&self, other: &Self) -> bool {
        self.target.eq(&other.target)
    }
}
