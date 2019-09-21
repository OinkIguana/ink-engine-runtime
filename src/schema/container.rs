use uuid::Uuid;
use super::{Object, object::NamedObject, Pointer};

#[derive(Clone, Debug, Default)]
pub struct Container {
    id: Uuid,
    name: String,

    pub(crate) content: Vec<Object>,
    pub(crate) visits_should_be_counted: bool,
    pub(crate) turn_index_should_be_counted: bool,
    pub(crate) counting_at_start_only: bool,

    // skip when serializing
    pub(crate) parent: Option<Pointer>,
}

impl NamedObject for Container {
    fn name(&self) -> &str {
        self.name.as_str()
    }
}

id_hash!(Container);
id_eq!(Container);
