use std::{any::Any, marker::PhantomData, rc::Rc};

use crate::app::{CreateContext, NodeId, WatchContext, WriteContext};

pub struct EventChannel<T> {
    emitter_id: NodeId,
    _phantom: PhantomData<*const T>,
}

impl<T: Any> EventChannel<T> {
    pub fn publish(&self, cx: &mut dyn WriteContext, event: T) {
        cx.runtime_mut()
            .publish_event(self.emitter_id, Rc::new(event));
    }

    pub fn subscribe(
        &self,
        cx: &mut dyn CreateContext,
        f: impl Fn(&mut dyn WatchContext, &T),
    ) -> EventSubscription {
        todo!()
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

impl<T: Any> EventReceiver<T> {}

pub fn create_event_channel<T: Any>(
    cx: &mut dyn CreateContext,
) -> (EventChannel<T>, EventReceiver<T>) {
    let owner = cx.owner();
    let emitter_id = cx.runtime_mut().create_event_emitter(owner);
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

pub(super) type HandleEventFn = dyn Fn(&mut dyn WatchContext, &dyn Any);

pub struct EventHandlerState {
    f: Rc<HandleEventFn>,
}
