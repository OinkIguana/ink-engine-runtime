use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

use super::*;

#[derive(Clone, Debug)]
pub struct Element {
    current_pointer: Pointer,

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
    previous_pointer: Pointer,
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
    //       maybe these can be an `Inventory` thing too? probably not
    variable_observers: HashMap<String, Vec<fn(String, Value)>>,

    has_validated_externals: bool,

    // StoryState stuff
    current_errors: Vec<String>,
    current_warnings: Vec<String>,

    output_stream: Vec<Object>,
    current_text: RefCell<Option<String>>,
    current_tags: RefCell<Option<Vec<String>>>,
    current_choices: Vec<Rc<Choice>>,

    diverted_pointer: Option<Pointer>,

    story_seed: usize,
    previous_random: usize,
    did_safe_exit: bool,

    current_turn_index: usize,
    visit_counts: HashMap<Path, usize>,
    turn_indices: HashMap<Path, usize>,

    // VariablesState stuff
    global_variables: HashMap<String, Object>,
    default_global_variables: HashMap<String, Object>,
    // TODO: investigate whether `evaluation_stack` has to be `Vec<Object>` or if it can be `Vec<Value>`
    evaluation_stack: Vec<Object>,

    // CallStack stuff
    threads: Vec<Thread>,
    thread_counter: usize,
    start_of_root: Pointer,
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

// Accessors
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

    pub fn can_continue(&self) -> bool {
        !self.current_pointer().is_null() && !self.has_error()
    }
}

// Story progression
impl Story {
    fn step(&mut self) {
        let mut pointer = self.current_pointer();
        if pointer.is_null() { return; }

        while let Some(container) = pointer.resolve().and_then(|obj| TryAsRef::<Rc<Container>>::try_as_ref(&obj).cloned()) {
            self.visit_container(&container, true);
            if container.is_empty() {
                break;
            }

            pointer = Pointer::to_start_of_container(&container);
        }

        let current_obj = pointer.resolve();
        self.set_current_pointer(pointer);

        let is_logic_or_flow_control = self.perform_logic_and_flow_control(current_obj);
    }

    /// Performs logic and flow control... returning true if the flow should be cancelled
    fn perform_logic_and_flow_control(&mut self, current_obj: Option<Object>) -> bool {
        let current_obj = match current_obj {
            Some(obj) => obj,
            None => return false,
        };

        match current_obj {
            Object::Divert(divert) => {
                if divert.is_conditional {
                    let val = self.evaluation_stack.pop().expect("No values on evaluation stack to pop when checking Divert condition");
                    // if the condition is false, return true to cancel the divert
                    if !val.is_truthy() {
                        return true;
                    }
                }

                match &divert.target {
                    DivertTarget::Variable(variable) => {
                        unimplemented!("TODO");
                    },
                    DivertTarget::External { path, args } => {},
                    DivertTarget::Path(path) => self.diverted_pointer = self.pointer_to_path(path, None),
                }

                if divert.pushes_to_stack {
                    // TODO: push to stack
                }

                if self.diverted_pointer.is_none() {}
            },
            Object::ControlCommand(command) => {},
            Object::VariableAssignment(assignment) => {},
            Object::VariableReference(reference) => {},
            Object::NativeFunctionCall(call) => {},
            _ => return false,
        }
        return true
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

    fn current_element(&self) -> &Element {
        self.threads.last().unwrap().elements.last().unwrap()
    }

    fn current_element_mut(&mut self) -> &mut Element {
        self.threads.last_mut().unwrap().elements.last_mut().unwrap()
    }

    fn current_pointer(&self) -> Pointer {
        self.current_element().current_pointer.clone()
    }

    fn set_current_pointer(&mut self, pointer: Pointer) {
        self.current_element_mut().current_pointer = pointer;
    }

    fn pointer_to_path(&self, path: &Path, relative_to: Option<Object>)-> Option<Pointer> {
        if path.is_empty() { return None }
        if path.is_relative {
            match relative_to {
                Some(object) => return object.resolve_path(path).as_ref().map(Pointer::to),
                None => panic!("Cannot resolve relative path with no provided root"),
            }
        }
        self.main_container.content_at_path(path).as_ref().map(Pointer::to)
    }
}
