use std::{any::Any, marker::PhantomData};

use crate::app::{Accessor, Effect, Owner, ReactiveContext, ReadSignal};

use super::{
    accessor::SourceId, CreateContext, NodeId, NodeType, ReadContext, Readable, WriteContext,
};

pub struct Signal<T> {
    pub(super) id: NodeId,
    _marker: PhantomData<*mut T>,
}

impl<T> Clone for Signal<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Signal<T> {}

impl<T: Any> Signal<T> {
    pub fn new(cx: &mut dyn CreateContext, value: T) -> Self {
        let state = SignalState::new(value);
        let owner = cx.owner();
        let id = cx.runtime_mut().create_signal_node(state, owner);
        Self {
            id,
            _marker: PhantomData,
        }
    }

    pub fn new_with(
        cx: &mut dyn CreateContext,
        f: impl FnOnce(&mut dyn CreateContext) -> T,
    ) -> Self {
        let value = f(cx);
        Self::new(cx, value)
    }

    /// Set the current value, notifies subscribers
    pub fn set(&self, cx: &mut dyn WriteContext, value: T) {
        self.update(cx, move |_, val| *val = value)
    }

    /// Set the current value, notifies subscribers
    pub fn set_with(&self, cx: &mut dyn WriteContext, f: impl FnOnce(&mut dyn CreateContext) -> T) {
        let new_value = f(&mut cx.as_create_context(super::Owner::Node(self.id)));
        self.update(cx, move |_, value| *value = new_value);
    }

    /// Set the current value, notifies subscribers
    pub fn update(
        &self,
        cx: &mut dyn WriteContext,
        f: impl FnOnce(&mut dyn CreateContext, &mut T),
    ) {
        if let Some(mut node) = cx.runtime_mut().lease_node(self.id) {
            match &mut node {
                NodeType::Signal(signal) => {
                    let value = signal
                        .value
                        .downcast_mut()
                        .expect("Invalid signal value type");
                    f(&mut cx.as_create_context(Owner::Node(self.id)), value);
                }
                _ => unreachable!(),
            }
            cx.runtime_mut().unlease_node(self.id, node);
        }
        cx.runtime_mut().notify(self.id);
    }

    pub fn as_read_signal(self) -> ReadSignal<T> {
        ReadSignal::from_node(self.id)
    }

    pub fn dispose(self, cx: &mut dyn ReactiveContext) {
        cx.runtime_mut().remove_node(self.id);
    }
}

impl<T: 'static> From<Signal<T>> for Accessor<T> {
    fn from(value: Signal<T>) -> Self {
        Accessor::ReadSignal(value.as_read_signal())
    }
}

impl<T: 'static> Readable for Signal<T> {
    type Value = T;

    fn track(&self, cx: &mut dyn ReadContext) {
        let scope = cx.scope();
        cx.runtime_mut().track(self.id, scope);
    }

    fn with_ref_untracked<R>(
        &self,
        cx: &mut dyn super::ReactiveContext,
        f: impl FnOnce(&Self::Value) -> R,
    ) -> R {
        f(cx.runtime_mut()
            .get_node_value_ref(self.id)
            .unwrap()
            .downcast_ref()
            .expect("Signal had wrong type"))
    }

    fn watch<F>(self, cx: &mut dyn CreateContext, f: F) -> Effect
    where
        F: FnMut(&mut dyn super::WatchContext, &Self::Value) + 'static,
    {
        Effect::watch_node(cx, self.id, f)
    }
}

pub struct SignalState {
    pub(super) value: Box<dyn Any>,
}

impl SignalState {
    pub fn new<T: Any>(value: T) -> Self {
        Self {
            value: Box::new(value),
        }
    }
}
