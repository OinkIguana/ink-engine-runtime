#[derive(Clone, Debug)]
pub struct ListEntry {
    origin: String,
    name: String,
    value: i64,
}

#[derive(Clone, Debug)]
pub struct ListDefinition {
    items: Vec<ListEntry>,
}

#[derive(Clone, Debug)]
pub struct List {
    items: Vec<ListEntry>,
}

impl List {
    pub fn is_empty(&self) -> bool { self.items.is_empty() }
}
