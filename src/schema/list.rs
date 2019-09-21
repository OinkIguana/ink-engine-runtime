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
