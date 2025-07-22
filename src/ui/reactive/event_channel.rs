use std::{any::Any, marker::PhantomData, rc::Rc};

use crate::ui::{CreateContext, NodeId, WatchContext, WriteContext};

pub struct EventChannel<T> {
    emitter_id: NodeId,
    _phantom: PhantomData<*const T>,
}

impl<T: Any> EventChannel<T> {
    pub fn publish(&self, cx: &mut dyn WriteContext, event: T) {
        cx.publish_event(self.emitter_id, Rc::new(event));
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
        cx: &mut dyn CreateContext,
        f: impl Fn(&mut dyn WatchContext, &T),
    ) -> EventSubscription {
        todo!()
    }
}

pub fn create_event_channel<T: Any>(
    cx: &mut dyn CreateContext,
) -> (EventChannel<T>, EventReceiver<T>) {
    let emitter_id = super::create_event_emitter(cx);
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

pub type HandleEventFn = dyn Fn(&mut dyn WatchContext, &dyn Any);

pub struct EventHandlerState {
    f: Rc<HandleEventFn>,
}
