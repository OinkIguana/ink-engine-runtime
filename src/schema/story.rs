use std::convert::TryInto;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt::{self, Debug, Formatter};
use rand_pcg::Pcg64;
use rand::{Rng, SeedableRng};

use super::*;

#[derive(Clone, Debug)]
pub struct Element {
    current_pointer: Pointer,

    in_expression_evaluation: bool,
    temporary_variables: HashMap<String, Object>,
    push_pop_type: PushPopType,

    evaluation_stack_size_when_called: usize,
    function_start_in_output_stream: usize,
}

impl Element {
    fn new(push_pop_type: PushPopType, current_pointer: Pointer) -> Self {
        Element {
            current_pointer,
            in_expression_evaluation: false,
            temporary_variables: HashMap::new(),
            push_pop_type,
            evaluation_stack_size_when_called: 0,
            function_start_in_output_stream: 0,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Thread {
    elements: Vec<Element>,
    index: usize,
    previous_pointer: Pointer,
}

impl Thread {
    fn new(push_pop_type: PushPopType, pointer: Pointer) -> Self {
        Thread {
            elements: vec![Element::new(push_pop_type, pointer)],
            index: 0,
            previous_pointer: Pointer::NULL,
        }
    }

    fn current_element(&self) -> &Element {
        self.elements.last().unwrap()
    }

    fn can_pop(&self, push_pop_type: Option<PushPopType>) -> bool {
        if self.elements.len() <= 1 { return false }
        match push_pop_type {
            None => true,
            Some(pop_type) => self.current_element().push_pop_type == pop_type,
        }
    }

    fn pop(&mut self, push_pop_type: Option<PushPopType>) {
        if self.can_pop(push_pop_type) {
            self.elements.pop();
        } else {
            panic!("Mismatched push/pop in callstack");
        }
    }
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
#[derive(Clone)]
pub struct Story {
    // Story stuff
    temporary_evaluation_container: Option<Container>,

    main_container: Rc<Container>,
    list_definitions: ListDefinitions,
    // TODO: don't require these to be `fn`, and allow `Box<dyn FnMut>` or something instead.
    //       requires implementing Debug manually
    //       maybe these can be an `Inventory` thing too? probably not
    //       also maybe they aren't needed at all... so we can do this part when they are needed instead
    //variable_observers: HashMap<String, Vec<Rc<RefCell<dyn FnMut(&String, &Value)>>>>,

    has_validated_externals: bool,

    // StoryState stuff
    output_stream: Vec<Object>,
    current_text: RefCell<Option<String>>,
    current_tags: RefCell<Option<Vec<String>>>,
    current_choices: Vec<Rc<Choice>>,

    diverted_pointer: Option<Pointer>,

    story_seed: u64,
    previous_random: u64,
    did_safe_exit: bool,

    current_turn_index: usize,
    visit_counts: HashMap<Path, usize>,
    turn_indices: HashMap<Path, usize>,

    // VariablesState stuff
    // TODO: investigate whether variables hold `Object` or only `Value`
    global_variables: HashMap<String, Object>,
    default_global_variables: HashMap<String, Object>,
    evaluation_stack: Vec<Object>,

    // CallStack stuff
    threads: Vec<Thread>,
    thread_counter: usize,
    start_of_root: Pointer,
}

impl Debug for Story {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Story {{ .. }}")
    }
}

impl Story {
    /// The current version of the ink story file format.
    pub const INK_VERSION_CURRENT: u32 = 19;

    /// The minimum legacy version of ink that can be loaded by the current version of the code.
    pub const INK_VERSION_MINIMUM_COMPATIBLE: u32 = 18;
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
        !self.current_pointer().is_null()
    }

    fn output_stream_dirty(&self) {
        self.current_text.borrow_mut().take();
        self.current_tags.borrow_mut().take();
    }
}

// Story progression
impl Story {
    fn continue_single_step(&mut self) -> bool {
        self.step();

        if !self.can_continue() && self.current_element().push_pop_type != PushPopType::FunctionEvaluationFromGame {
            self.try_follow_default_invisible_choice();
        }

        unimplemented!();
    }

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

        let mut current_obj = pointer.resolve();
        self.set_current_pointer(pointer);

        let is_logic_or_flow_control = self.perform_logic_and_flow_control(current_obj.clone());
        if self.current_pointer().is_null() { return; }

        let mut should_add_to_stream = true;
        if is_logic_or_flow_control {
            should_add_to_stream = false;
        }

        if let Some(choice_point) = current_obj.as_ref().and_then(TryAsRef::<Rc<ChoicePoint>>::try_as_ref).cloned() {
            if let Some(choice) = self.process_choice(choice_point) {
                self.current_choices.push(Rc::new(choice));
            }

            current_obj = None;
            should_add_to_stream = false;
        }

        if current_obj.as_ref().and_then(TryAsRef::<Rc<Container>>::try_as_ref).is_some() {
            should_add_to_stream = false;
        }

        if should_add_to_stream {
            if let Some(var_pointer) = current_obj.as_ref().and_then(TryAsRef::<VariablePointer>::try_as_ref).cloned() {
                let var_pointer = match var_pointer.context {
                    VariableContext::Unknown => {
                        let context = self.context_for_variable_named(&var_pointer.name);
                        VariablePointer {
                            name: var_pointer.name.clone(),
                            context,
                        }
                    }
                    _ => var_pointer.clone(),
                };

                if self.current_element().in_expression_evaluation {
                    self.evaluation_stack.push(Object::Value(Value::VariablePointer(var_pointer)));
                } else {
                    self.output_stream.push(Object::Value(Value::VariablePointer(var_pointer)));
                }
            }
        }

        self.next_content();

        if let Some(Object::ControlCommand(ControlCommand::StartThread)) = current_obj {
            let thread = self.fork_thread();
            self.threads.push(thread);
        }
    }

    /// Performs logic and flow control... returning true if the flow should be cancelled
    fn perform_logic_and_flow_control(&mut self, current_obj: Option<Object>) -> bool {
        let current_obj = match current_obj {
            Some(obj) => obj,
            None => return false,
        };

        match current_obj {
            Object::Divert(divert) => self.perform_divert(divert),
            Object::ControlCommand(command) => self.perform_control_command(command),
            Object::VariableAssignment(assignment) => self.perform_variable_assignment(assignment),
            Object::VariableReference(reference) => self.perform_variable_reference(reference),
            Object::NativeFunctionCall(call) => self.perform_native_function_call(call),
            _ => false,
        }
    }

    fn perform_divert(&mut self, divert: Rc<Divert>) -> bool {
        if divert.is_conditional {
            let val = self.evaluation_stack.pop().expect("No values on evaluation stack to pop when checking Divert condition");
            // if the condition is false, return true to cancel the divert
            if !val.is_truthy() {
                return true;
            }
        }

        match &divert.target {
            DivertTarget::Variable(variable) => {
                let value = self.get_variable_value(variable)
                    .expect(&format!("Attempted to divert to a variable target, but no variable was found named {}", variable));

                match &value {
                    Value::DivertTarget(path) => self.diverted_pointer = self.pointer_at_path(path),
                    _ => panic!("Attempted to divert to a variable target, but variable {} contained a non-divert target value {:?}", variable, value),
                }
            },
            DivertTarget::External { path, args } => {},
            DivertTarget::Path(path) => self.diverted_pointer = self.pointer_to_path(path, None),
        }

        if divert.pushes_to_stack {
            let element = Element {
                current_pointer: self.current_pointer(),
                in_expression_evaluation: false,
                temporary_variables: HashMap::default(),
                push_pop_type: divert.stack_push_type,
                evaluation_stack_size_when_called: 0,
                function_start_in_output_stream: self.output_stream.len(),
            };
            self.threads
                .last_mut()
                .unwrap()
                .elements
                .push(element);
        }

        if self.diverted_pointer.is_none() && !divert.is_external() {
            panic!("Attempted to divert to target {:?}, but could not", divert.target);
        }

        true
    }

    fn perform_control_command(&mut self, command: ControlCommand) -> bool {
        match command {
            ControlCommand::NoOp => {}
            ControlCommand::EvalStart => self.current_element_mut().in_expression_evaluation = true,
            ControlCommand::EvalEnd => self.current_element_mut().in_expression_evaluation = false,
            ControlCommand::EvalOutput => {
                if let Some(output) = self.evaluation_stack.pop() {
                    if output != Object::Void {
                        self.output_stream.push(output);
                    }
                }
            }
            ControlCommand::Duplicate => self.evaluation_stack.push(self.evaluation_stack.first().unwrap().clone()),
            ControlCommand::PopEvaluatedValue => { self.evaluation_stack.pop(); }
            | ControlCommand::PopFunction
            | ControlCommand::PopTunnel => {
                let pop_type = if command == ControlCommand::PopFunction { PushPopType::Function } else { PushPopType::Tunnel };
                let override_path: Option<Path> = if command == ControlCommand::PopTunnel {
                    let object = self.evaluation_stack.pop().expect("Expected value when popping tunnel, but no values existed on the evaluation stack");
                    match object {
                        Object::Void => None,
                        Object::Value(value) => Some(value.try_into().expect("Invalid type of divert target override value encountered when popping tunnel")),
                        _ => panic!("Invalid divert target override object encountered when popping tunnel. Expected Void or Value, found {:?}", object),
                    }
                } else { None };

                if self.try_exit_function_evaluation_from_game() {
                    return true;
                }
                if self.current_element().push_pop_type != pop_type {
                    panic!("Expected to pop {}, but instead attempted to pop {}", self.current_element().push_pop_type, pop_type);
                }
                if self.current_thread().elements.is_empty() {
                    panic!("Expected end of flow, but instead attempted to pop {}", pop_type);
                }

                self.pop_call_stack(None);
            }
            ControlCommand::BeginString => {
                assert!(!self.current_element().in_expression_evaluation, "Error processing control command: Must be in expression evaluation mode to begin a string");
                self.current_element_mut().in_expression_evaluation = false;
                self.output_stream.push(Object::ControlCommand(ControlCommand::BeginString));
            }
            ControlCommand::EndString => {
                let mut string_content = VecDeque::new();
                let mut output_count_consumed = 0;
                for i in (0..=self.output_stream.len()).rev() {
                    let obj = &self.output_stream[i];
                    output_count_consumed += 1;
                    if TryAsRef::<ControlCommand>::try_as_ref(obj).is_some() { break }
                    if let Some(string) = TryAsRef::<String>::try_as_ref(obj) {
                        string_content.push_front(string.as_str());
                    }
                }
                let string: String = string_content.into_iter().collect();
                self.output_stream.truncate(self.output_stream.len() - output_count_consumed);
                self.output_stream.push(Object::Value(Value::String(string)));
                self.current_element_mut().in_expression_evaluation = true;
            }
            ControlCommand::ChoiceCount => {
                let count = self.current_choices.len() as i64;
                self.evaluation_stack.push(Object::Value(Value::Int(count)));
            }
            ControlCommand::Turns => {
                self.evaluation_stack.push(Object::Value(Value::Int(self.current_turn_index as i64)));
            }
            | ControlCommand::TurnsSince
            | ControlCommand::ReadCount => {
                let object = self.evaluation_stack.pop();
                let target = match object.as_ref().and_then(TryAsRef::<Path>::try_as_ref) {
                    Some(target) => target,
                    None => panic!("Expected to find a divert target value to check turns since/read count, but found {:?}", object),
                };

                let count = if command == ControlCommand::TurnsSince {
                    self.visit_counts
                        .get(target)
                        .cloned()
                        .map(|u| u as i64)
                        .unwrap_or(0) // default is zero because never visited this one
                } else {
                    self.turn_indices
                        .get(target)
                        .cloned()
                        .map(|u| u as i64)
                        .unwrap_or(-1) // -1 to indicate never reached before
                };

                self.evaluation_stack.push(Object::Value(Value::Int(count)));
            }
            ControlCommand::Random => {
                let max_int = self.evaluation_stack.pop().and_then(|value| TryInto::<i64>::try_into(value).ok()).expect("Invalid parameter for max value of RANDOM");
                let min_int = self.evaluation_stack.pop().and_then(|value| TryInto::<i64>::try_into(value).ok()).expect("Invalid parameter for min value of RANDOM");
                let result_seed = self.story_seed + self.previous_random;
                let mut rng = Pcg64::seed_from_u64(result_seed as u64);
                let result = rng.gen_range(min_int, max_int + 1);
                self.previous_random = result as u64;
                self.evaluation_stack.push(Object::Value(Value::Int(result)));
            }
            ControlCommand::SeedRandom => {
                let seed = self.evaluation_stack.pop().and_then(|value| TryInto::<i64>::try_into(value).ok()).expect("Integer value was not provided to SEED_RANDOM");
                self.story_seed = seed as u64;
                self.previous_random = 0;
                self.evaluation_stack.push(Object::Void);
            }
            ControlCommand::VisitIndex => {
                let pointer = self.current_pointer();
                let object = Object::Container(pointer.container().unwrap());
                let path = object.path();
                let visit_count = self.visit_counts
                    .get(&path)
                    .cloned()
                    .unwrap_or(0);
                self.evaluation_stack.push(Object::Value(Value::Int(visit_count as i64 - 1)));
            }
            ControlCommand::SequenceShuffleIndex => {
                let index = self.next_sequence_shuffle_index();
                self.evaluation_stack.push(Object::Value(Value::Int(index)));
            }
            ControlCommand::StartThread => { /* handled elsewhere */ }
            ControlCommand::Done => {
                if self.can_pop_thread() {
                    self.threads.pop();
                } else {
                    self.did_safe_exit = true;
                    self.set_current_pointer(Pointer::NULL);
                }
            }
            ControlCommand::End => {
                self.force_end();
            }
            ControlCommand::ListFromInt => {
                let int = self.evaluation_stack.pop().and_then(|value| TryInto::<i64>::try_into(value).ok()).expect("Needs int to make a list from int");
                let list_name = self.evaluation_stack.pop().and_then(|value| TryInto::<String>::try_into(value).ok()).expect("Expected string value for list name when making a list from int");
                let list_definition = self.list_definitions.list_definition_by_name(&list_name).expect(&format!("No list definition found named {}", list_name));
                match list_definition.item_with_value(int) {
                    Some(entry) => self.evaluation_stack.push(Object::Value(Value::List(List::of_single_value(entry.clone())))),
                    None => self.evaluation_stack.push(Object::Value(Value::List(List::default()))),
                }
            }
            ControlCommand::ListRange => {
                let max = self.evaluation_stack.pop().and_then(|obj| TryInto::<Value>::try_into(obj).ok()).expect("Invalid value provided for list range max");
                let min = self.evaluation_stack.pop().and_then(|obj| TryInto::<Value>::try_into(obj).ok()).expect("Invalid value provided for list range min");
                let target_list = self.evaluation_stack.pop().and_then(|obj| TryInto::<List>::try_into(obj).ok()).expect("Invalid value provided for list range list");
                let sliced = target_list.slice(min, max);
                self.evaluation_stack.push(Object::Value(Value::List(sliced)));
            }
            ControlCommand::ListRandom => {
                let list = self.evaluation_stack.pop().and_then(|obj| TryInto::<List>::try_into(obj).ok()).expect("Invalid list provided for list random");
                if list.is_empty() {
                    self.evaluation_stack.push(Object::Value(Value::List(List::default())));
                } else {
                    let result_seed = self.story_seed + self.previous_random;
                    let mut random = Pcg64::seed_from_u64(result_seed);
                    let index = random.gen_range(0, list.len() as u64);
                    let entry = list.items.iter().nth(index as usize).cloned().unwrap();
                    self.evaluation_stack.push(Object::Value(Value::List(List::of_single_value(entry))));
                }
            }
        }

        true
    }

    fn perform_variable_assignment(&mut self, assignment: Rc<VariableAssignment>) -> bool {
        let assigned_value = self.evaluation_stack.pop().expect("Value must be provided for variable assignment");
        self.assign(assignment, assigned_value);
        true
    }

    fn perform_variable_reference(&mut self, reference: Rc<VariableReference>) -> bool {
        match &*reference {
            VariableReference::PathForCount(path) => {
                let count = self.visit_counts.get(path).cloned().unwrap_or(0);
                self.evaluation_stack.push(Object::Value(Value::Int(count as i64)));
            },
            VariableReference::Variable(name) => {
                let value = self.get_variable_value(name).expect(&format!("Variable reference failed to find variable named {}", name));
                self.evaluation_stack.push(Object::Value(value));
            },
        }
        true
    }

    fn perform_native_function_call(&mut self, call: Rc<NativeFunctionCall>) -> bool {
        let new_evaluation_stack = self.evaluation_stack.split_off(call.number_of_parameters());
        let params = std::mem::replace(&mut self.evaluation_stack, new_evaluation_stack);
        let values = params
            .into_iter()
            .map(TryInto::try_into)
            .map(|value| value.unwrap())
            .collect();
        let result = call.call(&self.list_definitions, values);
        self.evaluation_stack.push(Object::Value(result));
        true
    }

    fn process_choice(&mut self, choice_point: Rc<ChoicePoint>) -> Option<Choice> {
        let mut show_choice = true;

        if choice_point.has_condition {
            let condition_value = self.evaluation_stack.pop().unwrap();
            if !condition_value.is_truthy() {
                show_choice = false;
            }
        }

        let choice_only_text = if choice_point.has_choice_only_content {
            self.evaluation_stack.pop().unwrap().try_into().unwrap()
        } else { String::new() };

        let start_text = if choice_point.has_start_content {
            self.evaluation_stack.pop().unwrap().try_into().unwrap()
        } else { String::new() };

        if choice_point.once_only {
            let visit_count = self.visit_counts.get(&choice_point.path_on_choice).cloned().unwrap_or(0);
            if visit_count > 0 {
                show_choice = false;
            }
        }

        if !show_choice { return None } // NOTE: have to always evaluate everything, otherwise the values will be on the stacks

        let choice = Choice::new(
            (start_text + &choice_only_text).trim().to_string(),
            choice_point.path_on_choice.clone(),
            choice_point.is_invisible_default,
            self.fork_thread(),
        );

        Some(choice)
    }

    fn next_content(&mut self) {
        self.current_thread_mut().previous_pointer = self.current_pointer();
        if let Some(pointer) = self.diverted_pointer.take() {
            self.set_current_pointer(pointer);
            self.visit_changed_containers_due_to_divert();
            if !self.current_pointer().is_null() {
                return;
            }
        }
        let successful_pointer_increment = self.increment_content_pointer();
        if !successful_pointer_increment {
            let mut did_pop = false;
            if self.current_thread().can_pop(Some(PushPopType::Function)) {
                self.pop_call_stack(Some(PushPopType::Function));
                if self.current_element().in_expression_evaluation {
                    self.evaluation_stack.push(Object::Void);
                }
                did_pop = true;
            } else if self.can_pop_thread() {
                self.threads.pop();
                did_pop = true;
            } else {
                self.try_exit_function_evaluation_from_game();
            }

            if did_pop && !self.current_pointer().is_null() {
                self.next_content();
            }
        }
    }

    fn try_follow_default_invisible_choice(&mut self) -> bool {
        let all_choices = &self.current_choices;
        let mut invisible_choices = all_choices.iter().filter(|choice| choice.is_invisible_default).collect::<Vec<_>>();
        // can only follow it automatically if it's the only choice
        if invisible_choices.is_empty() || all_choices.len() > invisible_choices.len() {
            return false;
        }

        let choice = invisible_choices.pop().unwrap().clone();
        self.set_current_thread(choice.thread_at_generation.clone());
        self.choose_path(&choice.target_path, false);
        return true;
    }

    fn choose_path(&mut self, path: &Path, incrementing_turn_index: bool) {
        self.set_chosen_path(path, incrementing_turn_index);
        self.visit_changed_containers_due_to_divert();
    }

    fn set_chosen_path(&mut self, path: &Path, incrementing_turn_index: bool) {
        self.current_choices.clear();

        let new_pointer = self.pointer_at_path(path).unwrap();
        self.set_current_pointer(new_pointer);
        if incrementing_turn_index {
            self.current_turn_index += 1;
        }
    }

    fn try_exit_function_evaluation_from_game(&mut self) -> bool {
        if self.current_element().push_pop_type == PushPopType::FunctionEvaluationFromGame {
            self.set_current_pointer(Pointer::NULL);
            self.did_safe_exit = true;
            true
        } else { false }
    }

    fn force_end(&mut self) {
        self.current_choices.clear();
        self.threads = vec![Thread::new(PushPopType::Tunnel, Pointer::NULL)];
        self.did_safe_exit = true;
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

    fn visit_changed_containers_due_to_divert(&mut self) {
        let previous_pointer = &self.current_thread().previous_pointer;
        let current_pointer = self.current_pointer();

        if current_pointer.index.is_none() { return }

        let mut prev_containers = vec![];
        if !previous_pointer.is_null() {
            let mut prev_ancestor = previous_pointer.resolve()
                .as_ref()
                .and_then(TryAsRef::<Rc<Container>>::try_as_ref)
                .cloned()
                .or_else(|| previous_pointer.container());
            while let Some(ancestor) = prev_ancestor {
                prev_ancestor = ancestor.parent
                    .as_ref()
                    .and_then(Pointer::resolve)
                    .as_ref()
                    .and_then(TryAsRef::<Rc<Container>>::try_as_ref)
                    .cloned();
                prev_containers.push(ancestor);
            }
        }

        let mut current_child_of_container = match current_pointer.resolve() {
            Some(child) => child,
            None => return,
        };

        let mut current_container = current_pointer.container();
        while let Some(container) = current_container {
            if prev_containers.iter().any(|prev| Rc::ptr_eq(prev, &container)) || container.counting_at_start_only {
                break;
            }
            let entering_at_start = !container.content.is_empty()
                && &current_child_of_container == container.content.first().unwrap();
            self.visit_container(&container, entering_at_start);
            current_container = container.parent
                .as_ref()
                .and_then(Pointer::resolve)
                .as_ref()
                .and_then(TryAsRef::<Rc<Container>>::try_as_ref)
                .cloned();
            current_child_of_container = Object::Container(container);
        }
    }

    // another sketchy pair of very similarly named functions... but this one seems to do something
    // different
    fn pointer_at_path(&self, path: &Path) -> Option<Pointer> {
        if path.is_empty() { return None }

        match path.parts.last().unwrap() {
            Component::Index(i) => {
                let part_path = path.without_last_component();
                let container: Rc<Container> = self.main_container
                    .content_at_path(&part_path)
                    .as_ref()
                    .and_then(TryAsRef::<Rc<Container>>::try_as_ref)
                    .cloned()?;
                Some(Pointer::new(&container, *i))
            },
            _ => {
                let container = self.main_container
                    .content_at_path(path)
                    .as_ref()
                    .and_then(TryAsRef::<Rc<Container>>::try_as_ref)
                    .cloned()?;
                Some(Pointer::to_start_of_container(&container))
            },
        }
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

    fn increment_content_pointer(&mut self) -> bool {
        let mut successful_increment = false;
        let mut pointer = self.current_pointer();
        // NOTE: some reason we just assume everything is not null here...
        pointer.increment_index();
        loop {
            let container: Rc<Container> = pointer.container().unwrap();
            if pointer.index.unwrap() < container.content.len() { break }

            successful_increment = false;

            let next_ancestor_pointer = match &container.parent {
                Some(pointer) => pointer.clone(),
                None => break,
            };

            let next_ancestor: Rc<Container> = next_ancestor_pointer.container().unwrap();
            let index_in_ancestor = match next_ancestor.content.iter().position(|obj| obj == &Object::Container(container.clone())) {
                Some(index) => index,
                None => break,
            };

            pointer = Pointer::new(&next_ancestor, index_in_ancestor + 1);
            successful_increment = true;
        }

        if !successful_increment { pointer = Pointer::NULL; }
        self.set_current_pointer(pointer);

        successful_increment
    }

    fn trim_whitespace_from_function_end(&mut self) {
        let function_start_point = self.current_element().function_start_in_output_stream;
        for i in (usize::max(function_start_point, 0)..=self.output_stream.len()).rev() {
            let obj = &self.output_stream[i];
            if let Some(text) = TryAsRef::<String>::try_as_ref(obj) {
                // NB: may differ from original implementation behaviour in this line
                //     A string value containing some combination of both spaces/tabs and newline
                //     will not be classified as whitespace in the original implementation, but
                //     will here
                if !text.trim().is_empty() { // if some of the string remains, it's not whitespace, so we can stop trimming
                    break;
                }
                self.output_stream.remove(i);
                self.output_stream_dirty();
            }
        }
    }

    // This method calculates the next sequence shuffle index iteratively by calculating all the
    // previous shuffle indices on the way. The shuffle must be deterministic
    fn next_sequence_shuffle_index(&mut self) -> i64 {
        let num_elements: i64 = *self.evaluation_stack.pop().as_ref().and_then(TryAsRef::try_as_ref).expect("Expected Int value (num_elements) when calculating next sequence shuffle index");
        let seq_container = self.current_pointer().container().expect("Invalid current pointer when calculating next sequence shuffle index");
        let seq_count: i64 = *self.evaluation_stack.pop().as_ref().and_then(TryAsRef::try_as_ref).expect("Expected Int value (seq_count) when calculating next sequence shuffle index");
        let loop_index = (seq_count / num_elements) as u64;
        let iteration_index = seq_count % num_elements;

        let path_str = format!("{}", Object::Container(seq_container).path());
        let hash = path_str.chars().fold(0, |acc, ch| acc + ch as u64);
        let seed = hash + loop_index + self.story_seed;
        let mut random = Pcg64::seed_from_u64(seed);
        let mut unpicked_indices = (0..num_elements).collect::<Vec<_>>();
        (0..iteration_index).fold(0, move |_, _| {
            let chosen: u64 = random.gen_range(0, unpicked_indices.len() as u64);
            unpicked_indices.remove(chosen as usize)
        })
    }
}

// Call stack
impl Story {
    fn current_thread(&self) -> &Thread {
        self.threads.last().unwrap()
    }

    fn set_current_thread(&mut self, thread: Thread) {
        assert!(self.threads.len() == 1);
        self.threads = vec![thread];
    }

    fn current_thread_mut(&mut self) -> &mut Thread {
        self.threads.last_mut().unwrap()
    }

    fn current_element(&self) -> &Element {
        self.current_thread().elements.last().unwrap()
    }

    fn current_element_mut(&mut self) -> &mut Element {
        self.current_thread_mut().elements.last_mut().unwrap()
    }

    fn current_pointer(&self) -> Pointer {
        self.current_element().current_pointer.clone()
    }

    fn set_current_pointer(&mut self, pointer: Pointer) {
        self.current_element_mut().current_pointer = pointer;
    }

    fn pop_call_stack(&mut self, push_pop_type: Option<PushPopType>) {
        if self.current_element().push_pop_type == PushPopType::Function {
            self.trim_whitespace_from_function_end();
        }
        self.current_thread_mut().pop(push_pop_type);
    }

    fn can_pop_thread(&self) -> bool {
        self.threads.len() > 1 && self.current_element().push_pop_type != PushPopType::FunctionEvaluationFromGame
    }

    fn fork_thread(&mut self) -> Thread {
        let mut thread = self.current_thread().clone();
        self.thread_counter += 1;
        thread.index = self.thread_counter;
        return thread;
    }
}

// Variables
impl Story {
    fn get_variable_value(&self, variable: &String) -> Option<Value> {
        self.get_variable_with_context(variable, VariableContext::Unknown)
    }

    fn get_variable_with_context(&self, variable: &String, context: VariableContext) -> Option<Value> {
        let raw_value = self.get_raw_variable_with_context(variable, context)?;

        match &raw_value {
            Value::VariablePointer(VariablePointer { name, context }) => self.get_variable_with_context(name, *context),
            _ => Some(raw_value),
        }
    }

    fn get_raw_variable_with_context(&self, variable: &String, context: VariableContext) -> Option<Value> {
        match context {
            | VariableContext::Unknown
            | VariableContext::Global => {
                let value = self.global_variables.get(variable)
                    .and_then(TryAsRef::<Value>::try_as_ref);
                if value.is_some() { return value.cloned(); }

                let default_value = self.default_global_variables.get(variable)
                    .and_then(TryAsRef::<Value>::try_as_ref);
                if default_value.is_some() { return default_value.cloned(); }

                let list_item_value = self.list_definitions.lookup_list_entry(variable)
                    .map(|entry| List::of_single_value(entry.clone()).into());
                if list_item_value.is_some() { return list_item_value; }
            }
            _ => {}
        }

        match context {
            VariableContext::Unknown => self.get_temporary_variable(variable, VariableContext::Unknown),
            VariableContext::Temporary(context) => self.get_temporary_variable(variable, VariableContext::Temporary(context)),
            VariableContext::Global => None,
        }
    }

    fn get_temporary_variable(&self, variable: &String, context: VariableContext) -> Option<Value> {
        let current_thread = self.threads.last()?;
        let element = match context {
            VariableContext::Unknown | VariableContext::Global => current_thread.elements.last(),
            VariableContext::Temporary(index) => current_thread.elements.get(index),
        };
        element?.temporary_variables.get(variable).and_then(TryAsRef::<Value>::try_as_ref).cloned()
    }

    fn set_temporary_variable(&mut self, name: String, value: Object, is_new_declaration: bool, context: VariableContext) {
        let index = match context {
            VariableContext::Global => panic!("Cannot set temporary variable if it is a global variable"),
            VariableContext::Temporary(index) => index,
            VariableContext::Unknown => self.current_thread().elements.len() - 1,
        };
        let old_value = self.current_thread().elements[index].temporary_variables.get(&name).cloned();
        if !is_new_declaration && old_value.is_none() {
            panic!("Variable {} is not defined in this context", name);
        }
        let new_value = match (old_value, value) {
            (Some(Object::Value(Value::List(List { origins, .. }))), Object::Value(Value::List(list))) => Object::Value(Value::List(list.with_empty_origins(&origins))),
            (_, value) => value,
        };
        self.emit_variable_changed_event(&name, &new_value);
        self.current_thread_mut().elements[index].temporary_variables.insert(name, new_value);
    }

    fn set_global_variable(&mut self, name: String, value: Object) {
        let old_value = self.global_variables.get(&name).cloned();
        let new_value = match (old_value, value) {
            (Some(Object::Value(Value::List(List { origins, .. }))), Object::Value(Value::List(list))) => Object::Value(Value::List(list.with_empty_origins(&origins))),
            (_, value) => value,
        };
        self.emit_variable_changed_event(&name, &new_value);
        self.global_variables.insert(name, new_value);
    }

    fn assign(&mut self, assignment: Rc<VariableAssignment>, mut value: Object) {
        let mut name = assignment.variable_name.clone();
        let mut assign_global = if assignment.is_new_declaration { assignment.is_global } else { self.global_variable_exists(&name) };
        let mut context = VariableContext::Unknown;
        if assignment.is_new_declaration {
            // creating a new variable pointer reference, we do this thing for some reason
            value = match value {
                Object::Value(Value::VariablePointer(VariablePointer { name, context })) => Object::Value(self.resolve_variable_pointer(name, context).into()),
                _ => value,
            }
        } else {
            // assigning to existing variable... then do this thing!
            while let Some(Value::VariablePointer(VariablePointer { name: new_name, context: new_context })) = self.get_raw_variable_with_context(&name, context) {
                name = new_name;
                context = new_context;
                assign_global = context == VariableContext::Global;
            }
        }

        if assign_global {
            self.set_global_variable(name, value);
        } else {
            self.set_temporary_variable(name, value, assignment.is_new_declaration, context);
        }
    }

    fn resolve_variable_pointer(&self, name: String, context: VariableContext) -> VariablePointer {
        let context = match context {
            VariableContext::Unknown => self.resolve_context_of_variable(&name),
            _ => context,
        };
        let value = self.get_raw_variable_with_context(&name, context);
        match value {
            Some(Value::VariablePointer(var_pointer)) => var_pointer,
            _ => VariablePointer { name, context },
        }
    }

    // NOTE: These two functions sound very similar, and work very similar, but are two distinct
    // functions in the original implementation... questionable, right?
    fn context_for_variable_named(&self, name: &String) -> VariableContext {
        if self.current_element().temporary_variables.get(name).is_some() {
            VariableContext::Temporary(self.current_thread().elements.len())
        } else {
            VariableContext::Global
        }
    }

    fn resolve_context_of_variable(&self, name: &String) -> VariableContext {
        if self.global_variable_exists(name) {
            VariableContext::Global
        } else {
            VariableContext::Temporary(self.current_thread().elements.len())
        }
    }

    fn global_variable_exists(&self, name: &String) -> bool {
        self.global_variables.get(name).is_some() || self.default_global_variables.get(name).is_some()
    }
}

// Events
impl Story {
    fn emit_variable_changed_event(&mut self, name: &String, value: &Object) {
        // TODO: this is not yet needed, so it is not implemented
    }
}
