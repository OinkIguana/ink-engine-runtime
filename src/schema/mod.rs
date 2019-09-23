//! Type definitions of all the components of an Ink story.

pub trait TryAsRef<T> where T: ?Sized {
    fn try_as_ref(&self) -> Option<&T>;
}

mod list;
mod object;
mod path;
mod pointer;
mod push_pop_type;
mod value;
#[macro_use]
mod external_function;
mod list_definitions;

pub use list::{ListEntry, List, ListDefinition};
pub use list_definitions::ListDefinitions;
pub use object::Object;
pub(crate) use object::NamedObject;
pub use path::{Path, Component};
pub use pointer::Pointer;
pub use push_pop_type::PushPopType;
pub use value::Value;
pub use external_function::ExternalFunction;

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
pub use divert::{Divert, DivertTarget};
pub use glue::Glue;
pub use native_function_call::NativeFunctionCall;
pub use story::Story;
pub use tag::Tag;
pub use variable_assignment::VariableAssignment;
pub use variable_reference::VariableReference;
