use std::collections::HashMap;
use super::{ListDefinition, ListEntry};

#[derive(Clone, Debug)]
pub struct ListDefinitions {
    list_definitions: HashMap<String, ListDefinition>,
    list_entry_lookup_cache: HashMap<String, ListEntry>,
}

impl ListDefinitions {
    pub(crate) fn new(list_definitions: Vec<ListDefinition>) -> Self {
        let list_definitions = list_definitions
            .into_iter()
            .map(|def| (def.name.to_string(), def))
            .collect::<HashMap<_, _>>();

        let list_entry_lookup_cache = list_definitions.values()
            .flat_map(|def| def.items.iter()
                .map(move |item| (format!("{}.{}", def.name, item.name), item.clone()))
            )
            .collect::<HashMap<_, _>>();
        
        Self {
            list_definitions,
            list_entry_lookup_cache,
        }
    }

    pub(crate) fn lookup_list_entry(&self, name: &String) -> Option<&ListEntry> {
        self.list_entry_lookup_cache.get(name)
    }
}
