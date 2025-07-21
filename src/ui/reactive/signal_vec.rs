use std::{any::Any, marker::PhantomData};

use crate::ui::Accessor;

use super::{
    CreateContext, NodeId, NodeType, Owner, ReactiveValue, ReactiveGraph, Trigger, WriteContext,
    var::SignalState,
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
        let owner = cx.owner();
        let id = cx.runtime_mut().create_signal_node(state, owner);
        Self {
            id,
            _phantom: PhantomData,
        }
    }

    pub fn push(&self, cx: &mut impl WriteContext, value: T) {
        let trigger = Trigger::new(&mut cx.as_create_context(Owner::Node(self.id)));
        self.with_inner_mut(cx.runtime_mut(), move |inner| {
            inner.values.push(value);
            inner.triggers.push(trigger);
        });
        cx.runtime_mut().notify(self.id);
    }

    pub fn extend(&self, cx: &mut impl WriteContext, iter: impl IntoIterator<Item = T>) {
        self.with_inner_mut(cx.runtime_mut(), move |inner| {
            inner.values.extend(iter);
        });
        cx.runtime_mut().notify(self.id);
    }

    pub fn retain(&self, cx: &mut impl WriteContext, f: impl Fn(&T) -> bool) {
        self.with_inner_mut(cx.runtime_mut(), move |inner| {
            inner.values.retain(f);
        });
        cx.runtime_mut().notify(self.id);
    }

    fn with_inner<R>(&self, cx: &ReactiveGraph, f: impl FnOnce(&Inner<T>) -> R) -> R {
        let value = match &cx.get_node(self.id).node_type {
            NodeType::Signal(signal) => signal.value.as_ref(),
            _ => unreachable!(),
        };
        f(value.downcast_ref().expect("Signal had wrong type"))
    }

    fn with_inner_mut<R>(&self, cx: &mut ReactiveGraph, f: impl FnOnce(&mut Inner<T>) -> R) -> R {
        let value = match &mut cx.get_node_mut(self.id).node_type {
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
        let scope = cx.scope();
        cx.runtime_mut().track(self.id, scope);
    }

    fn with_ref<R>(&self, cx: &mut dyn super::ReadContext, f: impl FnOnce(&Self::Value) -> R) -> R {
        let scope = cx.scope();
        cx.runtime_mut().track(self.id, scope);
        self.with_inner(cx.runtime(), move |value| f(&value.values))
    }

    fn with_ref_untracked<R>(
        &self,
        cx: &mut dyn super::ReactiveContext,
        f: impl FnOnce(&Self::Value) -> R,
    ) -> R {
        self.with_inner(cx.runtime(), move |value| f(&value.values))
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
        let scope = cx.scope();
        cx.runtime_mut().track(self.id, scope);
    }

    fn with_ref_untracked<R>(
        &self,
        cx: &mut dyn super::ReactiveContext,
        f: impl FnOnce(&Self::Value) -> R,
    ) -> R {
        self.parent.with_inner(cx.runtime(), move |inner| {
            f(inner.values.get(self.index).unwrap())
        })
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
