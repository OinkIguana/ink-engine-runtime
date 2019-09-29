use super::Path;

#[derive(Clone, Debug)]
pub struct ChoicePoint {
    pub(crate) path_on_choice: Path,
    pub(crate) has_condition: bool,
    pub(crate) has_start_content: bool,
    pub(crate) has_choice_only_content: bool,
    pub(crate) once_only: bool,
    pub(crate) is_invisible_default: bool,
}
