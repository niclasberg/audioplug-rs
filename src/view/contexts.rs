use std::{marker::PhantomData, any::Any, iter::repeat};
use crate::{ViewMessage, Message, Id, ViewFlags};
use super::{IdPath, ViewNode};

pub trait Context {
    fn id_path(&self) -> &IdPath;
    //fn with_child<'s, 'r, T>(&'s mut self, child_id: Id, f: impl FnOnce(&'r mut Self) -> T) -> T where 's: 'r;
}

pub struct LayoutContext<'a> {
    id_path: IdPath,
    node: &'a mut ViewNode
}

impl<'a> Context for LayoutContext<'a> {
    fn id_path(&self) -> &IdPath {
        &self.id_path
    }
}

impl<'a> LayoutContext<'a> {
    pub fn new(node: &'a mut ViewNode) -> Self {
        Self { node, id_path: IdPath::root() }
    }

    pub fn request_render(&mut self) {
        self.node.set_flag(ViewFlags::NEEDS_RENDER);
    }

	pub fn get_child<'s>(&'s mut self, id: Id) -> Option<LayoutContext<'s>> {
		self.node.children.get_mut(id.0).map(|child| {
			LayoutContext { id_path: self.id_path.child_id(id), node: child }
		})
	}

	/*fn with_child<'s, T>(&'s mut self, id: Id, f: impl FnOnce(&mut Self) -> T) -> T where 'a: 's {
        if self.node.children.len() <= id.0 {
            let items_to_create = id.0 - self.node.children.len() + 1;
            self.node.children.extend(repeat(ViewNode::new()).take(items_to_create));
        }

        let result = {
            let id_path = self.id_path.child_id(id);
            let mut child = Self { id_path, node: &mut self.node.children[id.0] };
            f(&mut child)
        };

		self.node.combine_child_flags(id.0);
		result
    }*/
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

    /*fn with_child<'s, 'r, T>(&'s mut self, id: Id, f: impl FnOnce(&'r mut Self) -> T) -> T where 's: 'r {
        let id_path = self.id_path.child_id(id);
        let mut children = std::mem::take(&mut self.node.children);

        let result = {
            let mut child = Self { 
                id_path, 
                node: &mut children[id.0], 
                messages: &mut self.messages, 
                _phantom: PhantomData  
            };
            f(&mut child)
        };
        std::mem::swap(&mut self.node.children, &mut children);
        result
    }*/
}

impl<'a, 'b, T> EventContext<'a, 'b, T> where 'a: 'b {
    pub fn new(node: &'a mut ViewNode, messages: &'b mut Vec<ViewMessage<Box<dyn Any>>>) -> Self{
        Self { id_path: IdPath::root(), node, messages, _phantom: PhantomData }
    }

    pub fn with_type_mut<U>(&mut self) -> &mut EventContext<'a, 'b, U> {
        unsafe {
            (self as *mut EventContext<'a, 'b, T> as *mut EventContext<'a, 'b, U>).as_mut()
        }.unwrap()
    }

	pub fn get_child<'s>(&'s mut self, id: Id) -> Option<EventContext<'s, 'b, T>> {
		self.node.children.get_mut(id.0).map(|child| {
			Self { id_path: self.id_path.child_id(id), node: child, messages: &mut self.messages, _phantom: PhantomData }
		})
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
