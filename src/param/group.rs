use std::ops::Deref;

use super::{
    traversal::{ParamVisitor, ParameterTraversal},
    GroupId,
};

pub trait AnyParameterGroup: 'static {
    fn id(&self) -> GroupId;
    fn name(&self) -> &'static str;
}

pub struct ParameterGroup<P: ParameterTraversal> {
    id: GroupId,
    name: &'static str,
    children: P,
}

impl<P: ParameterTraversal> ParameterGroup<P> {
    pub fn new(id: GroupId, name: &'static str, children: P) -> Self {
        Self { id, name, children }
    }

    pub fn children(&self) -> &P {
        &self.children
    }
}

impl<P: ParameterTraversal> AnyParameterGroup for ParameterGroup<P> {
    fn id(&self) -> GroupId {
        self.id
    }

    fn name(&self) -> &'static str {
        &self.name
    }
}

impl<P: ParameterTraversal> ParameterTraversal for ParameterGroup<P> {
    fn visit<V: ParamVisitor>(&self, visitor: &mut V) {
        visitor.group(self)
    }
}

impl<P: ParameterTraversal> Deref for ParameterGroup<P> {
    type Target = P;

    fn deref(&self) -> &Self::Target {
        &self.children
    }
}
