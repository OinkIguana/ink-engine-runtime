use super::{Container, Object};
use std::rc::Rc;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Pointer {
    pub(crate) container: Rc<Container>,
    pub(crate) index: Option<usize>,
}

impl Pointer {
    pub(crate) fn resolve(&self) -> Option<Object> {
        match self.index {
            None => Some(Object::Container(self.container.clone())),
            Some(index) => self.container.content.get(index).cloned(),
        }
    }
}
