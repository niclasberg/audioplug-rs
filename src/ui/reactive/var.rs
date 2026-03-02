use std::{any::Any, marker::PhantomData, ops::DerefMut};

use super::{
    CanCreate, CanRead, CanWrite, Effect, NodeId, Owner, ReactiveValue, ReadSignal,
    runtime::NodeType,
};
use crate::ui::{
    ViewProp,
    reactive::{CreateContext, WatchContext},
};

/// A value that may change over time.
pub struct Var<T> {
    pub(crate) id: NodeId,
    _marker: PhantomData<*mut T>,
}

impl<T> Clone for Var<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Var<T> {}

impl<T: Any> Var<T> {
    pub fn new<'cx>(cx: &mut impl CanCreate<'cx>, value: T) -> Self {
        let state = SignalState::new(value);
        let id = cx.create_context().create_var_node(state);
        Self {
            id,
            _marker: PhantomData,
        }
    }

    pub fn new_with<'cx>(
        cx: &mut impl CanCreate<'cx>,
        f: impl FnOnce(&mut CreateContext) -> T,
    ) -> Self {
        let value = f(&mut cx.create_context());
        Self::new(cx, value)
    }

    /// Set the current value, notifies subscribers
    pub fn set<'cx>(&self, cx: &mut impl CanWrite<'cx>, value: T) {
        self.update(cx, move |_, val| *val = value)
    }

    /// Set the current value, notifies subscribers
    pub fn set_with<'cx>(
        &self,
        cx: &mut impl CanWrite<'cx>,
        f: impl FnOnce(&mut CreateContext) -> T,
    ) {
        let new_value = f(&mut cx
            .write_context()
            .as_create_context(super::Owner::Node(self.id)));
        self.update(cx, move |_, value| *value = new_value);
    }

    /// Set the current value, notifies subscribers
    pub fn update<'s>(
        &self,
        cx: &mut impl CanWrite<'s>,
        f: impl FnOnce(&mut CreateContext, &mut T),
    ) {
        let mut cx = cx.write_context();
        if let Some(mut node) = cx.reactive_graph.lease_node(self.id) {
            match node.deref_mut() {
                NodeType::Signal(signal) => {
                    let value = signal
                        .value
                        .downcast_mut()
                        .expect("Invalid signal value type");
                    f(&mut cx.as_create_context(Owner::Node(self.id)), value);
                }
                _ => unreachable!(),
            }
            cx.reactive_graph.unlease_node(node);
        }
        cx.write_context().notify(self.id);
    }

    pub fn as_read_signal(self) -> ReadSignal<T> {
        ReadSignal::from_node(self.id)
    }

    /*pub fn dispose(self, cx: &mut dyn ReactiveContext) {
        cx.reactive_graph_mut().remove_node(self.id);
    }*/
}

impl<T: 'static> From<Var<T>> for ViewProp<T> {
    fn from(value: Var<T>) -> Self {
        ViewProp::ReadSignal(value.as_read_signal())
    }
}

impl<T: 'static> ReactiveValue for Var<T> {
    type Value = T;

    fn track<'a>(&self, cx: &mut impl CanRead<'a>) {
        cx.read_context().track(self.id);
    }

    fn with_ref_untracked<'a, R>(
        &self,
        cx: &mut impl CanRead<'a>,
        f: impl FnOnce(&Self::Value) -> R,
    ) -> R {
        f(cx.read_context()
            .get_node_value_ref(self.id)
            .unwrap()
            .downcast_ref()
            .expect("Signal had wrong type"))
    }

    fn watch<'cx, F>(self, cx: &mut impl CanCreate<'cx>, f: F) -> Effect
    where
        F: FnMut(&mut WatchContext, &Self::Value) + 'static,
    {
        Effect::watch_node(cx.create_context(), self.id, f)
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
