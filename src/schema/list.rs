#[derive(Clone, Eq, PartialEq, Debug)]
pub struct ListEntry {
    pub(crate) origin: String,
    pub(crate) name: String,
    pub(crate) value: i64,
}

#[derive(Clone, Debug)]
pub struct ListDefinition {
    pub(crate) name: String,
    pub(crate) items: Vec<ListEntry>,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct List {
    pub(crate) items: Vec<ListEntry>,
}

impl List {
    pub(crate) fn of_single_value(value: ListEntry) -> Self {
        Self { items: vec![value] }
    }

    pub(crate) fn is_empty(&self) -> bool { self.items.is_empty() }
}
