use super::{Container, Object};
use std::rc::Weak;

#[derive(Clone, Debug)]
pub struct Pointer {
    pub(crate) container: Option<Weak<Container>>,
    pub(crate) index: Option<usize>,
}

impl Pointer {
    pub(crate) fn resolve(&self) -> Option<Object> {
        let container = self.container.as_ref()?.upgrade()?;
        match self.index {
            None => Some(Object::Container(container.clone())),
            Some(index) => container.content.get(index).cloned(),
        }
    }

    pub(crate) fn is_null(&self) -> bool {
        self.container.is_none() && self.index.is_none()
    }
}
