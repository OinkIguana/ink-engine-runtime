use std::collections::HashMap;
use std::rc::Rc;

use super::*;

#[derive(Clone, Debug)]
pub struct Element {
    current_pointer: Rc<Pointer>,

    is_expression_evaluation: bool,
    temporary_variables: HashMap<String, Rc<Object>>,
    push_pop_type: PushPopType,

    evaluation_stack_size_when_called: usize,
    function_start_in_output_stream: usize,
}

#[derive(Clone, Debug)]
pub struct Thread {
    elements: Vec<Element>,
    index: usize,
    previous_pointer: Rc<Pointer>,
}

/// This `Story` is comparable to the official `Story` class, but with the `StoryState`, `VariablesState`,
/// and `CallStack` all merged together. This had to be done because the 3 components used to use
/// shared references to the parent/each other, but we cannot easily share ownership/self reference
/// in Rust.
///
/// Also note that all stuff related to patching (`StatePatch`) and asynchronous *anything* has been
/// removed, as they are not relevant additions in a Rust implementation. The saving will not be 
/// handled internally (`Story` will implement `Serialize`/`Deserialize`), so a simple `story.clone()` will
/// be enough to take a snapshot and save it in the background on another thread while the game 
/// still plays. Asynchronous features are just out of scope for this project.
#[derive(Clone, Debug)]
pub struct Story {
    // Story stuff
    temporary_evaluation_container: Option<Container>,

    main_container: Rc<Container>,
    list_definitions: HashMap<String, ListDefinition>,
    // TODO: don't require these to be `fn`, and allow `Box<dyn FnMut>` or something instead.
    //       requires implementing Debug manually
    variable_observers: HashMap<String, Vec<fn(String, Value)>>,

    has_validated_externals: bool,

    // StoryState stuff
    current_errors: Vec<String>,
    current_warnings: Vec<String>,
    evaluation_stack: Vec<Object>,
    diverted_pointer: Rc<Pointer>,

    current_turn_index: usize,
    story_seed: usize,
    previous_random: usize,
    did_safe_exit: bool,

    visit_counts: HashMap<String, usize>,
    turn_indices: HashMap<String, usize>,
    output_stream: Vec<Object>,
    output_stream_text_dirty: bool,
    output_stream_tags_dirty: bool,
    current_choices: Vec<Rc<Choice>>,

    // VariablesState stuff
    global_variables: HashMap<String, Object>,
    default_global_variables: HashMap<String, Object>,

    // CallStack stuff
    threads: Vec<Thread>,
    thread_counter: usize,
    start_of_root: Rc<Pointer>,
}

impl Story {
    /// The current version of the ink story file format.
    pub const INK_VERSION_CURRENT: u32 = 19;

    /// The minimum legacy version of ink that can be loaded by the current version of the code.
    pub const INK_VERSION_MINIMUM_COMPATIBLE: u32 = 18;
}

impl Story {
    pub fn current_choices(&self) -> Vec<Rc<Choice>> {
        unimplemented!();
    }

    pub fn current_text(&self) -> String {
        unimplemented!();
    }

    pub fn current_tags(&self) -> Vec<Rc<Tag>> {
        unimplemented!();
    }
}
