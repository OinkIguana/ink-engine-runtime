//! Type definitions of all the components of an Ink story.

mod object;
mod path;
mod pointer;
mod push_pop_type;

pub use object::Object;
pub use path::Path;
pub use pointer::Pointer;
pub use push_pop_type::PushPopType;

mod choice;
mod choice_point;
mod container;
mod control_command;
mod divert;
mod glue;
mod native_function_call;
mod story;
mod tag;
mod variable_assignment;
mod variable_reference;

pub use choice::Choice;
pub use choice_point::ChoicePoint;
pub use container::Container;
pub use control_command::ControlCommand;
pub use divert::Divert;
pub use glue::Glue;
pub use native_function_call::NativeFunctionCall;
pub use story::Story;
pub use tag::Tag;
pub use variable_assignment::VariableAssignment;
pub use variable_reference::VariableReference;
