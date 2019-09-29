use super::{Path, Thread};

#[derive(Debug)]
pub struct Choice {
    pub(crate) text: String,
    //pub(crate) source_path: Path, // NB: field excluded because it seems not used/useful yet
    pub(crate) target_path: Path,
    pub(crate) is_invisible_default: bool,
    pub(crate) thread_at_generation: Thread,
}

impl Choice {
    pub(crate) fn new(
        text: String, 
        target_path: Path,
        is_invisible_default: bool,
        thread_at_generation: Thread,
    ) -> Self {
        Self { text, target_path, is_invisible_default, thread_at_generation }
    }
}
