use std::rc::Rc;
use super::{Object, NamedObject, Path, Pointer, Component, TryAsRef};

#[derive(Clone, Debug, Default)]
pub struct Container {
    name: String,

    pub(crate) content: Vec<Object>,
    pub(crate) visits_should_be_counted: bool,
    pub(crate) turn_index_should_be_counted: bool,
    pub(crate) counting_at_start_only: bool,

    // skip when serializing
    pub(crate) parent: Option<Pointer>,
}

impl Container {
    pub(crate) fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    pub(crate) fn index_of(&self, object: &Object) -> Option<usize> {
        self.content.iter().position(|item| item == object)
    }

    pub(crate) fn content_at_path(&self, path: &Path) -> Option<Object> {
        self.content_at_path_part(path, 0)
    }

    pub(crate) fn content_at_path_part(&self, path: &Path, start: usize) -> Option<Object> {
        if path.is_empty() { return None } // cannot resolve an empty path... should deal with that outside
        let mut object: Option<Object> = None;
        let mut container = Some(self);
        for component in &path.parts[start..] {
            match component {
                Component::Index(index) => object = container?.content.get(*index).cloned(),
                Component::Name(name) => {
                    for o in &container?.content {
                        match TryAsRef::<dyn NamedObject>::try_as_ref(o) {
                            Some(named_object) if named_object.name() == name => {
                                object = Some(o.clone());
                                break;
                            }
                            _ => {}
                        }
                    }
                    return None;
                }
                Component::Parent => object = self.parent.as_ref()?.resolve(),
            }

            container = object
                .as_ref()
                .and_then(TryAsRef::<Rc<Container>>::try_as_ref)
                .map(|rc| rc.as_ref());
        }
        object
    }
}

impl NamedObject for Container {
    fn name(&self) -> &str {
        self.name.as_str()
    }
}
