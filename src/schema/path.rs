use std::ops::Index;

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub enum Component {
    Index(usize),
    Name(String),
    Parent,
}

#[derive(Clone, Hash, Eq, PartialEq, Debug, Default)]
pub struct Path {
    pub(crate) parts: Vec<Component>,
    pub(crate) is_relative: bool,
}

impl Path {
    pub(crate) fn join<I: Into<Component>>(&mut self, part: I) {
        self.parts.push(part.into());
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.parts.is_empty()
    }
}

impl From<usize> for Component {
    fn from(v: usize) -> Self {
        Self::Index(v)
    }
}

impl From<String> for Component {
    fn from(v: String) -> Self {
        Self::Name(v)
    }
}

impl From<()> for Component {
    fn from((): ()) -> Self {
        Self::Parent
    }
}

impl Index<usize> for Path {
    type Output = Component;
    fn index(&self, index: usize) -> &Self::Output {
        &self.parts[index]
    }
}
