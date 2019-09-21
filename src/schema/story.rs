use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

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
    output_stream: Vec<Object>,
    current_text: RefCell<Option<String>>,
    current_tags: RefCell<Option<Vec<String>>>,
    current_choices: Vec<Rc<Choice>>,

    current_errors: Vec<String>,
    current_warnings: Vec<String>,
    evaluation_stack: Vec<Object>,
    diverted_pointer: Rc<Pointer>,

    story_seed: usize,
    previous_random: usize,
    did_safe_exit: bool,

    current_turn_index: usize,
    visit_counts: HashMap<Path, usize>,
    turn_indices: HashMap<Path, usize>,

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

// Error accessors
impl Story {
    pub fn has_error(&self) -> bool {
        !self.current_errors.is_empty()
    }

    pub fn has_warning(&self) -> bool {
        !self.current_warnings.is_empty()
    }

    pub fn current_errors(&self) -> &[String] {
        &self.current_errors
    }

    pub fn current_warnings(&self) -> &[String] {
        &self.current_warnings
    }
}

// Story progression
impl Story {
    pub fn current_choices(&self) -> Vec<Rc<Choice>> {
        // current choices does not include the invisible default choice
        self.current_choices
            .iter()
            .filter(|choice| !choice.is_invisible_default)
            .cloned()
            .collect()
    }

    pub fn current_text(&mut self) -> String {
        if let Some(ref text) = *self.current_text.borrow() {
            return text.clone();
        }

        let text = self.output_stream
            .iter()
            .filter_map(TryAsRef::<String>::try_as_ref)
            .map(|string| string.as_str())
            .collect::<String>();
        *self.current_text.borrow_mut() = Some(text.clone());
        text
    }

    pub fn current_tags(&self) -> Vec<String> {
        if let Some(ref tags) = *self.current_tags.borrow() {
            return tags.clone();
        }

        let tags = self.output_stream
            .iter()
            .filter_map(TryAsRef::<Rc<Tag>>::try_as_ref)
            .map(|tag| tag.text().to_owned())
            .collect::<Vec<_>>();
        *self.current_tags.borrow_mut() = Some(tags.clone());
        tags
    }

    pub fn step(&mut self) {
        let pointer = self.current_pointer();
        if pointer.is_null() { return; }

        while let Some(container) = pointer.resolve().and_then(|obj| TryAsRef::<Rc<Container>>::try_as_ref(&obj).cloned()) {
            self.visit_container(&container, true);
        }
    }

    fn current_pointer(&self) -> Rc<Pointer> {
        self.threads.last().unwrap().elements.last().unwrap().current_pointer.clone()
    }

    pub fn can_continue(&self) -> bool {
        !self.current_pointer().is_null() && !self.has_error()
    }
}

// Story helpers
impl Story {
    fn visit_container(&mut self, container: &Rc<Container>, at_start: bool) {
        if !container.counting_at_start_only || at_start {
            if container.visits_should_be_counted {
                *self.visit_counts.entry(Object::Container(container.clone()).path()).or_default() += 1;
            }
            if container.turn_index_should_be_counted {
                self.turn_indices.insert(Object::Container(container.clone()).path(), self.current_turn_index);
            }
        }
    }
}
