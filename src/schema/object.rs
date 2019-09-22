use super::*;
use std::rc::Rc;

pub(crate) trait NamedObject {
    fn name(&self) -> &str;
    fn has_valid_name(&self) -> bool { !self.name().is_empty() }
}

impl<T> NamedObject for Rc<T> where T: NamedObject {
    #[allow(unconditional_recursion)] // its (supposed to be) calling a different implementation of this method...
    fn name(&self) -> &str { (*self).name() }
}

#[derive(Clone, Debug)]
pub enum Object {
    Choice(Rc<Choice>),
    ChoicePoint(Rc<ChoicePoint>),
    Container(Rc<Container>),
    ControlCommand(Rc<ControlCommand>),
    Divert(Rc<Divert>),
    Glue(Rc<Glue>),
    NativeFunctionCall(Rc<NativeFunctionCall>),
    Story(Rc<Story>),
    Tag(Rc<Tag>),
    VariableAssignment(Rc<VariableAssignment>),
    VariableReference(Rc<VariableReference>),
    Value(Rc<Value>),
    Void,
}

impl Object {
    /// The path to this object from the root of the story
    pub(crate) fn path(&self) -> Path {
        let mut path = Path::default();
        let mut content = self.clone();
        let mut maybe_container: Option<Object> = self.parent();

        while let Some(container) = maybe_container.as_ref().and_then(TryAsRef::<Rc<Container>>::try_as_ref) {
            // for nameable content, which has a name, use the name
            match TryAsRef::<dyn NamedObject>::try_as_ref(&content) {
                Some(named_content) if named_content.has_valid_name() => path.join(named_content.name().to_owned()),
                _ => {
                    let index = container.index_of(&content);
                    path.join(index.expect("Child Object was not found in parent Container's content list"));
                }
            }
            content = Object::Container(container.clone());
            maybe_container = content.parent();
        }

        path
    }

    // TODO: parents are probably always Container
    pub(crate) fn parent(&self) -> Option<Self> {
        use Object::*;
        match self {
            Container(container) => container.parent.as_ref().and_then(Pointer::resolve),
            _ => None,
        }
    }

    pub(crate) fn resolve_path(&self, path: &Path) -> Option<Self> {
        if path.is_relative {
            match path[0] {
                Component::Parent => TryAsRef::<Rc<Container>>::try_as_ref(&self.parent()?)?.content_at_path_part(path, 1),
                _ => TryAsRef::<Rc<Container>>::try_as_ref(self)?.content_at_path(path),
            }
        } else {
            self.root_container()?.content_at_path(path)
        }
    }

    pub(crate) fn root_container(&self) -> Option<Rc<Container>> {
        match TryAsRef::<Rc<Container>>::try_as_ref(&self.parent()?) {
            None => self.try_as_ref().cloned(),
            Some(..) => self.parent().unwrap().root_container()
        }
    }

    /// Checks the truthiness of the Object. Returns false if the contained object is not a Value
    /// object. Otherwise, follows the values returned from `Value::is_truthy`:
    /// *   Int: value is not 0
    /// *   Float: value is not 0.0
    /// *   String: string is not empty
    /// *   List: list is not empty
    /// *   Anything else: false
    ///
    /// # Panics
    ///
    /// Panics if the value is a divert target or a variable pointer
    pub(crate) fn is_truthy(&self) -> bool {
        match self {
            Object::Value(value) => value.is_truthy(),
            _ => false,
        }
    }
}

impl std::cmp::Eq for Object {}
impl std::cmp::PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        use Object::*;
        match (self, other) {
            (Choice(a), Choice(b)) => Rc::ptr_eq(a, b),
            (ChoicePoint(a), ChoicePoint(b)) => Rc::ptr_eq(a, b),
            (Container(a), Container(b)) => Rc::ptr_eq(a, b),
            (ControlCommand(a), ControlCommand(b)) => Rc::ptr_eq(a, b),
            (Divert(a), Divert(b)) => Rc::ptr_eq(a, b),
            (Glue(a), Glue(b)) => Rc::ptr_eq(a, b),
            (NativeFunctionCall(a), NativeFunctionCall(b)) => Rc::ptr_eq(a, b),
            (Story(a), Story(b)) => Rc::ptr_eq(a, b),
            (Tag(a), Tag(b)) => Rc::ptr_eq(a, b),
            (VariableAssignment(a), VariableAssignment(b)) => Rc::ptr_eq(a, b),
            (VariableReference(a), VariableReference(b)) => Rc::ptr_eq(a, b),
            (Value(a), Value(b)) => Rc::ptr_eq(a, b),
            (Void, Void) => true,
            _ => false,
        }
    }
}

impl TryAsRef<Rc<Choice>> for Object {
    fn try_as_ref(&self) -> Option<&Rc<Choice>> {
        match self {
            Self::Choice(ref choice) => Some(choice),
            _ => None,
        }
    }
}

impl TryAsRef<Rc<ChoicePoint>> for Object {
    fn try_as_ref(&self) -> Option<&Rc<ChoicePoint>> {
        match self {
            Self::ChoicePoint(ref value) => Some(value),
            _ => None,
        }
    }
}

impl TryAsRef<Rc<Container>> for Object {
    fn try_as_ref(&self) -> Option<&Rc<Container>> {
        match self {
            Self::Container(ref value) => Some(value),
            _ => None,
        }
    }
}

impl TryAsRef<Rc<Divert>> for Object {
    fn try_as_ref(&self) -> Option<&Rc<Divert>> {
        match self {
            Self::Divert(ref value) => Some(value),
            _ => None,
        }
    }
}

impl TryAsRef<Rc<Glue>> for Object {
    fn try_as_ref(&self) -> Option<&Rc<Glue>> {
        match self {
            Self::Glue(ref value) => Some(value),
            _ => None,
        }
    }
}

impl TryAsRef<Rc<NativeFunctionCall>> for Object {
    fn try_as_ref(&self) -> Option<&Rc<NativeFunctionCall>> {
        match self {
            Self::NativeFunctionCall(ref value) => Some(value),
            _ => None,
        }
    }
}

impl TryAsRef<Rc<Story>> for Object {
    fn try_as_ref(&self) -> Option<&Rc<Story>> {
        match self {
            Self::Story(ref value) => Some(value),
            _ => None,
        }
    }
}

impl TryAsRef<Rc<Tag>> for Object {
    fn try_as_ref(&self) -> Option<&Rc<Tag>> {
        match self {
            Self::Tag(ref value) => Some(value),
            _ => None,
        }
    }
}

impl TryAsRef<Rc<VariableAssignment>> for Object {
    fn try_as_ref(&self) -> Option<&Rc<VariableAssignment>> {
        match self {
            Self::VariableAssignment(ref value) => Some(value),
            _ => None,
        }
    }
}

impl TryAsRef<Rc<VariableReference>> for Object {
    fn try_as_ref(&self) -> Option<&Rc<VariableReference>> {
        match self {
            Self::VariableReference(ref value) => Some(value),
            _ => None,
        }
    }
}

impl TryAsRef<Rc<Value>> for Object {
    fn try_as_ref(&self) -> Option<&Rc<Value>> {
        match self {
            Self::Value(ref value) => Some(value),
            _ => None
        }
    }
}

impl<T> TryAsRef<T> for Object where Value: TryAsRef<T> {
    fn try_as_ref(&self) -> Option<&T> {
        match self {
            Self::Value(ref value) => value.try_as_ref(),
            _ => None,
        }
    }
}

impl TryAsRef<(dyn NamedObject + 'static)> for Object {
    fn try_as_ref(&self) -> Option<&(dyn NamedObject + 'static)> {
        match self {
            Self::Container(ref container) => Some(container),
            _ => None,
        }
    }
}
