use std::iter::FromIterator;
use std::cmp::{Ord, PartialOrd, Ordering};
use std::ops::{BitAnd, BitOr, Sub};
use std::collections::BTreeSet;
use super::{Value, ListDefinitions};

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct ListEntry {
    pub(crate) origin: String,
    pub(crate) name: String,
    pub(crate) value: i64,
}

impl Ord for ListEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value).then(self.name.cmp(&other.name))
    }
}

impl PartialOrd for ListEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Debug)]
pub struct ListDefinition {
    pub(crate) name: String,
    pub(crate) items: BTreeSet<ListEntry>,
}

impl ListDefinition {
    pub(crate) fn item_with_value(&self, value: i64) -> Option<&ListEntry> {
        self.items
            .iter()
            .find(|entry| entry.value == value)
    }
}

#[derive(Clone, Debug, Default)]
pub struct List {
    pub(crate) origins: BTreeSet<String>,
    pub(crate) items: BTreeSet<ListEntry>,
}

// Constructors
impl List {
    pub(crate) fn of_single_value(value: ListEntry) -> Self {
        Self {
            origins: vec![value.origin.clone()].into_iter().collect(),
            items: vec![value].into_iter().collect(),
        }
    }

    pub(crate) fn of_single_value_with_origins<'a, I: IntoIterator<Item = &'a String>>(value: ListEntry, origins: I) -> Self {
        Self {
            origins: origins.into_iter().cloned().collect(),
            items: vec![value].into_iter().collect(),
        }
    }

    pub(crate) fn with_empty_origins<'a, I: IntoIterator<Item = &'a String>>(mut self, origins: I) -> Self {
        if self.items.is_empty() {
            self.origins = origins.into_iter().cloned().collect();
        }
        self
    }
}

// Accessors
impl List {
    pub(crate) fn len(&self) -> usize { self.items.len() }
    pub(crate) fn is_empty(&self) -> bool { self.items.is_empty() }
}

// Operations
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
            origins: self.origins.clone(),
            items: self.items
                .iter()
                .filter(|entry| entry.value >= min && entry.value <= max)
                .cloned()
                .collect()
        }
    }

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

    pub fn gt(&self, other: &Self) -> bool {
        if self.is_empty() { return false }
        if other.is_empty() { return true }
        self.min().unwrap() > other.max().unwrap()
    }

    pub fn lt(&self, other: &Self) -> bool {
        if other.is_empty() { return false }
        if self.is_empty() { return true }
        self.max().unwrap() < other.min().unwrap()
    }

    pub fn ge(&self, other: &Self) -> bool {
        if self.is_empty() { return false }
        if other.is_empty() { return true }
        self.max().unwrap() >= other.max().unwrap() && self.min().unwrap() >= other.min().unwrap()
    }

    pub fn le(&self, other: &Self) -> bool {
        if other.is_empty() { return false }
        if self.is_empty() { return true }
        self.max().unwrap() <= other.max().unwrap() && self.min().unwrap() <= other.min().unwrap()
    }

    pub fn increment(&self, distance: i64, lists: &ListDefinitions) -> Self {
        Self {
            origins: self.origins.clone(),
            items: self.items
                .iter()
                .filter_map(|&ListEntry { ref origin, value, .. }| lists
                    .list_definition_by_name(origin)?
                    .item_with_value(value + distance)
                )
                .cloned()
                .collect(),
        }
    }

    pub fn contains(&self, other: &Self) -> bool {
        for item in &other.items {
            if !self.items.contains(item) {
                return false
            }
        }
        true
    }

    pub fn invert(&self, lists: &ListDefinitions) -> List {
        Self {
            origins: self.origins.clone(),
            items: &self.origins
                .iter()
                .filter_map(|name| lists.list_definition_by_name(name))
                .flat_map(|list| list.items.iter())
                .cloned()
                .collect::<BTreeSet<_>>() - &self.items
        }
    }
}

impl Sub<&List> for List {
    type Output = List;
    fn sub(self, other: &List) -> List {
        Self {
            items: &self.items - &other.items,
            ..self
        }
    }
}

impl BitOr<&List> for List {
    type Output = List;
    fn bitor(self, other: &List) -> List {
        Self {
            origins: &self.origins | &other.origins,
            items: &self.items | &other.items,
        }
    }
}

impl BitAnd<&List> for List {
    type Output = List;
    fn bitand(self, other: &List) -> List {
        Self {
            origins: &self.origins & &other.origins,
            items: &self.items & &other.items,
        }
    }
}

impl PartialEq for List {
    fn eq(&self, other: &Self) -> bool {
        self.items == other.items
    }
}

impl FromIterator<ListEntry> for List {
    fn from_iter<I: IntoIterator<Item=ListEntry>>(iter: I) -> Self {
        let items: BTreeSet<ListEntry> = iter.into_iter().collect();
        List {
            origins: items.iter().map(|item| item.origin.to_string()).collect(),
            items,
        }
    } 
}
