use std::ops::Index;
use std::fmt::{self, Display, Formatter};

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub enum Component {
    Index(usize),
    Name(String),
    Parent,
}

impl Display for Component {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Component::Index(int) => write!(f, "{}", int),
            Component::Name(name) => write!(f, "{}", name),
            Component::Parent => write!(f, "^"),
        }
    }
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

impl Display for Path {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let part_strs: Vec<String> = self.parts
            .iter()
            .map(|comp| format!("{}", comp))
            .collect();
        write!(f, "{}", part_strs.join("."))
    }
}
