use std::{marker::PhantomData, any::Any, iter::repeat};
use crate::{ViewMessage, Message, Id, ViewFlags};
use super::{IdPath, ViewNode};

pub trait Context<'a> {
    fn id_path(&self) -> &IdPath;
	fn child_mut(&mut self, id: Id) -> &'a mut Self;
}

pub struct LayoutContext<'a> {
    id_path: IdPath,
    node: &'a mut ViewNode
}

impl<'a> Context<'a> for LayoutContext<'a> {
    fn id_path(&self) -> &IdPath {
        &self.id_path
    }

	

    /*fn with_child<T>(&mut self, child_id: Id, f: impl FnOnce(&mut Self) -> T) -> T {
        // When we do layout straight after building, the child nodes
        // will not have yet been created
        i

		let id_path = self.id_path.child_id(child_id);

		let result = {
			let mut children = std::mem::take(&mut self.node.children);
			f(&mut LayoutContext { node: &mut children[child_id.0], id_path: id_path })
		};
		self.node.combine_child_flags(child_id.0);
		result
    }*/

    fn child_mut(&mut self, id: Id) -> &'a mut Self {
        if self.node.children.len() <= id.0 {
            let items_to_create = id.0 - self.node.children.len() + 1;
            self.node.children.extend(repeat(ViewNode::new()).take(items_to_create));
        }


    }
}

impl<'a> LayoutContext<'a> {
    pub fn new(node: &'a mut ViewNode) -> Self {
        Self { node, id_path: IdPath::root() }
    }

    pub fn request_render(&mut self) {
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
