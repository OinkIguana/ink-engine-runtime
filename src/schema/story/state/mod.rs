mod patch;
use patch::StatePatch;

#[derive(Clone, Debug)]
pub struct StoryState {
    patch: Option<StatePatch>,
}

impl StoryState {
    /// The current version of the state save file JSON-based format.<Paste>
    pub const INK_SAVE_STATE_VERSION: u32 = 8;
    pub const MIN_COMPATIBLE_LOAD_VERSION: u32 = 8;
}

impl StoryState {
    pub fn visit_count_at_path_string(path_string: String) -> u32 {
        0
    }
}
