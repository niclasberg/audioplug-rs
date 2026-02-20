use std::{any::Any, marker::PhantomData};

use crate::ui::{Accessor, ReactiveContext};

use super::{
    CreateContext, NodeId, NodeType, Owner, ReactiveValue, Trigger, WriteContext, var::SignalState,
};

#[derive(Copy, Clone)]
pub struct SignalVec<T> {
    id: NodeId,
    _phantom: PhantomData<*mut T>,
}

impl<T: Any> SignalVec<T> {
    pub fn new(cx: &mut dyn CreateContext) -> Self {
        let state = SignalState::new(Inner::<T> {
            values: Vec::new(),
            triggers: Vec::new(),
            len_trigger: None,
        });
        let id = cx.create_var_node(state);
        Self {
            id,
            _phantom: PhantomData,
        }
    }

    pub fn push(&self, cx: &mut impl WriteContext, value: T) {
        let trigger = Trigger::new(&mut cx.with_owner(Owner::Node(self.id)));
        self.with_inner_mut(cx, move |inner| {
            inner.values.push(value);
            inner.triggers.push(trigger);
        });
        super::notify(cx, self.id);
    }

    pub fn extend(&self, cx: &mut impl WriteContext, iter: impl IntoIterator<Item = T>) {
        self.with_inner_mut(cx, move |inner| {
            inner.values.extend(iter);
        });
        super::notify(cx, self.id);
    }

    pub fn retain(&self, cx: &mut impl WriteContext, f: impl Fn(&T) -> bool) {
        self.with_inner_mut(cx, move |inner| {
            inner.values.retain(f);
        });
        super::notify(cx, self.id);
    }

    fn with_inner<R>(&self, cx: &dyn ReactiveContext, f: impl FnOnce(&Inner<T>) -> R) -> R {
        let graph = &cx.reactive_graph();
        let value = match &graph.get_node(self.id).node_type {
            NodeType::Signal(signal) => signal.value.as_ref(),
            _ => unreachable!(),
        };
        f(value.downcast_ref().expect("Signal had wrong type"))
    }

    fn with_inner_mut<R>(
        &self,
        cx: &mut dyn WriteContext,
        f: impl FnOnce(&mut Inner<T>) -> R,
    ) -> R {
        let graph = &mut cx.reactive_graph_mut();
        let value = match &mut graph.get_node_mut(self.id).node_type {
            NodeType::Signal(signal) => signal.value.as_mut(),
            _ => unreachable!(),
        };
        let inner: &mut Inner<T> = value.downcast_mut().expect("Signal had wrong type");
        let size_before = inner.values.len();
        let result = f(inner);
        if let Some(len_trigger) = inner.len_trigger {
            if size_before != inner.values.len() {
                len_trigger.notify(cx);
            }
        }
        result
    }
}

impl<T> From<SignalVec<T>> for Accessor<Vec<T>> {
    fn from(value: SignalVec<T>) -> Self {
        todo!()
    }
}

impl<T: Any> ReactiveValue for SignalVec<T> {
    type Value = Vec<T>;

    fn track(&self, cx: &mut dyn super::ReadContext) {
        cx.track(self.id);
    }

    fn with_ref<R>(&self, cx: &mut dyn super::ReadContext, f: impl FnOnce(&Self::Value) -> R) -> R {
        cx.track(self.id);
        self.with_inner(cx, move |value| f(&value.values))
    }

    fn with_ref_untracked<R>(
        &self,
        cx: &mut dyn super::ReactiveContext,
        f: impl FnOnce(&Self::Value) -> R,
    ) -> R {
        self.with_inner(cx, move |value| f(&value.values))
    }

    fn watch<F>(self, cx: &mut dyn CreateContext, f: F) -> super::Effect
    where
        F: FnMut(&mut dyn super::WatchContext, &Self::Value) + 'static,
    {
        todo!()
    }
}

pub struct AtIndex<Parent, T> {
    index: usize,
    parent: Parent,
    id: NodeId,
    _phantom2: PhantomData<*const T>,
}

impl<T: Any> From<AtIndex<SignalVec<T>, T>> for Accessor<T> {
    fn from(value: AtIndex<SignalVec<T>, T>) -> Self {
        todo!()
    }
}

impl<T: Any> ReactiveValue for AtIndex<SignalVec<T>, T> {
    type Value = T;

    fn track(&self, cx: &mut dyn super::ReadContext) {
        cx.track(self.id);
    }

    fn with_ref_untracked<R>(
        &self,
        cx: &mut dyn super::ReactiveContext,
        f: impl FnOnce(&Self::Value) -> R,
    ) -> R {
        self.parent
            .with_inner(cx, move |inner| f(inner.values.get(self.index).unwrap()))
    }

    fn watch<F>(self, cx: &mut dyn CreateContext, f: F) -> super::Effect
    where
        F: FnMut(&mut dyn super::WatchContext, &Self::Value) + 'static,
    {
        todo!()
    }
}

struct Inner<T> {
    values: Vec<T>,
    triggers: Vec<Trigger>,
    len_trigger: Option<Trigger>,
}
