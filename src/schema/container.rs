use std::rc::Rc;
use uuid::Uuid;
use super::Object;

#[derive(Clone, Debug, Default)]
pub struct Container {
    id: Uuid,
    name: String,

    pub(crate) content: Vec<Object>,
    visits_should_be_counted: bool,
    turn_index_should_be_counted: bool,
    counting_at_start_only: bool,
}

id_hash!(Container);
id_eq!(Container);
