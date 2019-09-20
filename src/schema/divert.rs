use super::{Path, Pointer, PushPopType};

#[derive(Clone, Debug)]
pub enum Divert {
    Path { 
        target_path: Path,
        target_pointer: Pointer,
        pushes_to_stack: bool,
        stack_push_type: PushPopType,
        is_external: bool,
        external_args: i32,
        is_conditional: bool,
    },
    Variable {
        variable_divert_name: String,
        pushes_to_stack: bool,
        stack_push_type: PushPopType,
        is_external: bool,
        external_args: i32,
        is_conditional: bool,
    }
}

impl std::cmp::PartialEq for Divert {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Divert::Path { target_path: a, .. }, Divert::Path { target_path: b, .. }) => a == b,
            (Divert::Variable { variable_divert_name: a, .. }, Divert::Variable { variable_divert_name: b, .. }) => a == b,
            _ => false,
        }
    }
}
