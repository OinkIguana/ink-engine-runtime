#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub enum Component {
    Index(usize),
    Name(String),
    Parent,
}

#[derive(Clone, Hash, Eq, PartialEq, Debug, Default)]
pub struct Path {
    parts: Vec<Component>,
}



impl Path {
    pub(crate) fn join<I: Into<Component>>(&mut self, part: I) {
        self.parts.push(part.into());
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
