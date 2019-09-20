use std::collections::{HashMap, HashSet};

use crate::schema::{Container, Object};

#[derive(Clone, Debug, Default)]
pub struct StatePatch {
    pub globals: HashMap<String, Object>,
    pub changed_variables: HashSet<String>,
    pub visit_counts: HashMap<Container, u32>,
    pub turn_indices: HashMap<Container, u32>,
}

impl StatePatch {
    pub fn try_get_global<K: AsRef<str>>(&self, name: K) -> Option<Object> {
        self.globals.get(name.as_ref()).cloned()
    }

    pub fn set_global(&mut self, name: String, value: Object) {
        self.globals.insert(name, value);
    }
}
