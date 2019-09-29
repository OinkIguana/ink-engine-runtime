use super::Path;

#[derive(Clone, Debug)]
pub struct Choice {
    pub(crate) text: String,
    //pub(crate) source_path: Path, // NB: field excluded because it seems not used/useful yet
    pub(crate) target_path: Path,
    pub(crate) is_invisible_default: bool,
}

impl Choice {
    pub(crate) fn new(
        text: String, 
        target_path: Path,
        is_invisible_default: bool,
    ) -> Self {
        Self { text, target_path, is_invisible_default }
    }
}
