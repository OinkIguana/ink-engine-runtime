use super::Value;

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

impl ListDefinition {
    pub(crate) fn item_with_value(&self, value: i64) -> Option<&ListEntry> {
        self.items
            .iter()
            .find(|entry| entry.value == value)
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub struct List {
    pub(crate) items: Vec<ListEntry>,
}

impl List {
    pub(crate) fn slice(&self, min: Value, max: Value) -> Self {
        let min = match min {
            Value::Int(v) => v,
            Value::List(list) => list.min().map(|v| v.value).unwrap_or(0),
            _ => 0,
        };
        let max = match max {
            Value::Int(v) => v,
            Value::List(list) => list.min().map(|v| v.value).unwrap_or(i64::max_value()),
            _ => i64::max_value(),
        };

        List {
            items: self.items
                .iter()
                .filter(|entry| entry.value >= min && entry.value <= max)
                .cloned()
                .collect()
        }
    }

    pub(crate) fn of_single_value(value: ListEntry) -> Self {
        Self { items: vec![value] }
    }

    pub(crate) fn len(&self) -> usize { self.items.len() }
    pub(crate) fn is_empty(&self) -> bool { self.items.is_empty() }

    pub(crate) fn min(&self) -> Option<&ListEntry> {
        self.items
            .iter()
            .fold(None, |min, cur| match (min, cur) {
                (None, entry) => Some(entry),
                (Some(ListEntry { value: a, .. }), ListEntry { value: b, .. }) if b < a => Some(cur),
                _ => min,
            })
    }

    pub(crate) fn max(&self) -> Option<&ListEntry> {
        self.items
            .iter()
            .fold(None, |max, cur| match (max, cur) {
                (None, entry) => Some(entry),
                (Some(ListEntry { value: a, .. }), ListEntry { value: b, .. }) if b > a => Some(cur),
                _ => max,
            })
    }
}
