use std::{marker::PhantomData, any::Any, iter::repeat};
use crate::{ViewMessage, Message, Id, ViewFlags};
use super::{IdPath, ViewNode};

pub trait Context {
    fn id_path(&self) -> &IdPath;
    fn with_child<T>(&mut self, child_id: Id, f: impl FnOnce(&mut Self) -> T) -> T;
}

pub struct LayoutContext<'a> {
    id_path: IdPath,
    node: &'a mut ViewNode
}

impl<'a> Context for LayoutContext<'a> {
    fn id_path(&self) -> &IdPath {
        &self.id_path
    }

    fn with_child<T>(&mut self, child_id: Id, f: impl FnOnce(&mut Self) -> T) -> T {
        // When we do layout straight after building, the child nodes
        // will not have yet been created
        if self.node.children.len() <= child_id.0 {
            let items_to_create = child_id.0 - self.node.children.len() + 1;
            self.node.children.extend(repeat(ViewNode::new()).take(items_to_create))
        }

        let child = &mut self.node.children[child_id.0];
        self.id_path.with_child_id(child_id, |id_path| {
            let result = f(&mut LayoutContext { node: child, id_path: id_path.clone() });
            self.node.combine_flags(child);
            result
        })
    }
}

impl<'a> LayoutContext<'a> {
    fn new(node: &'a mut ViewNode) -> Self {
        Self { node, id_path: IdPath::root() }
    }

    fn request_render(&mut self) {
        self.node.set_flag(ViewFlags::NEEDS_RENDER);
    }
}

pub struct RenderContext {

}

pub struct EventContext<'a, 'b, Message: 'static> {
    id_path: IdPath,
    node: &'a mut ViewNode,
    messages: &'b mut Vec<ViewMessage<Box<dyn Any>>>,
    _phantom: PhantomData<Message>,
}

impl<'a, 'b, Message: 'static> Context for EventContext<'a, 'b, Message> {
    fn id_path(&self) -> &IdPath {
        &self.id_path
    }

    fn with_child<T>(&mut self, child_id: Id, f: impl FnOnce(&mut Self) -> T) -> T {
        todo!()
    }
}

impl<'a, 'b, T> EventContext<'a, 'b, T> {
    pub fn new(node: &'a mut ViewNode, messages: &'b mut Vec<ViewMessage<Box<dyn Any>>>) -> Self{
        Self { id_path: IdPath::root(), node, messages, _phantom: PhantomData }
    }

    pub fn with_type_mut<U>(&mut self) -> &mut EventContext<'a, 'b, U> {
        unsafe {
            (self as *mut EventContext<'a, 'b, T> as *mut EventContext<'a, 'b, U>).as_mut()
        }.unwrap()
    }

    pub fn request_layout(&mut self) {
        todo!()
    }

    pub fn request_render(&mut self) {
        todo!()
    }

    pub fn request_rebuild(&mut self) {
        todo!()
    }

    pub fn publish_message(&mut self, msg: T) {
        let body: Box<dyn Any> = Box::new(msg);
        let body = Message::Widget(body);
        self.messages.push(ViewMessage { 
            view_id: self.id_path().clone(),
            body
        });
    }
}
