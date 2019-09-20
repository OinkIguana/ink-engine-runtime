use super::*;
use std::rc::Rc;

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
    Void,
}
