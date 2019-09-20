use super::Path;

#[derive(Clone, Debug)]
pub struct Choice {
    text: String,
    source_path: Path,
    target_path: Path,
    is_invisible_default: bool,
}
