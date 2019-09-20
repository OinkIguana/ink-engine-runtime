use serde::{Serialize, Deserialize};

mod state;
use state::StoryState;

use crate::schema::Choice;

#[derive(Clone, Debug)]
pub struct Story {
    state: StoryState,
}

impl Story {
    /// The current version of the ink story file format.
    pub const INK_VERSION_CURRENT: u32 = 19;

    /// The minimum legacy version of ink that can be loaded by the current version of the code.
    pub const INK_VERSION_MINIMUM_COMPATIBLE: u32 = 18;
}

impl Story {
    pub fn current_choices() -> Vec<Choice> {
        unimplemented!();
    }
}
