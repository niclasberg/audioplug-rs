use std::{any::Any, marker::PhantomData, rc::Rc};

use super::{CanCreate, CanWrite, NodeId, WatchContext};

pub struct EventChannel<T> {
    emitter_id: NodeId,
    _phantom: PhantomData<*const T>,
}

impl<T: Any> EventChannel<T> {
    pub fn publish(&self, cx: &mut dyn CanWrite, event: T) {
        //cx.publish_event(self.emitter_id, Rc::new(event));
    }
}

pub struct EventSubscription {
    emitter_id: NodeId,
    receiver_id: NodeId,
}

#[derive(Clone, Copy)]
pub struct EventReceiver<T> {
    emitter_id: NodeId,
    _phantom: PhantomData<*const T>,
}

impl<T: Any> EventReceiver<T> {
    pub fn subscribe(
        &self,
        cx: &mut dyn CanCreate,
        f: impl Fn(&mut WatchContext, &T),
    ) -> EventSubscription {
        todo!()
    }
}

pub fn create_event_channel<'cx, T: Any>(
    cx: impl CanCreate<'cx>,
) -> (EventChannel<T>, EventReceiver<T>) {
    let emitter_id = cx.create_context().create_event_emitter();
    let emitter = EventChannel {
        emitter_id,
        _phantom: PhantomData,
    };
    let receiver = EventReceiver {
        emitter_id,
        _phantom: PhantomData,
    };
    (emitter, receiver)
}

pub type HandleEventFn = dyn Fn(&mut WatchContext, &dyn Any);

pub struct EventHandlerState {
    f: Rc<HandleEventFn>,
}
