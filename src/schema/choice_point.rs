use super::{Container, Path};

#[derive(Clone, Debug)]
pub struct ChoicePoint {
    path_on_choice: Path,
    has_condition: bool,
    has_start_content: bool,
    has_choice_only_content: bool,
    once_only: bool,
    is_invisible_default: bool,
}

impl ChoicePoint {
    fn choice_target(&self) -> Container { unimplemented!() }
}
