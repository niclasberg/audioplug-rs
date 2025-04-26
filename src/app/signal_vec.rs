use std::{any::Any, marker::PhantomData};

use super::{
    accessor::SourceId, signal::SignalState, CreateContext, NodeId, NodeType, Owner, Readable,
    Runtime, Trigger, WriteContext,
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

    fn with_inner<R>(&self, cx: &Runtime, f: impl FnOnce(&Inner<T>) -> R) -> R {
        let value = match &cx.get_node(self.id).node_type {
            NodeType::Signal(signal) => signal.value.as_ref(),
            _ => unreachable!(),
        };
        f(value.downcast_ref().expect("Signal had wrong type"))
    }

    fn with_inner_mut<R>(&self, cx: &mut Runtime, f: impl FnOnce(&mut Inner<T>) -> R) -> R {
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

impl<T: Any> Readable for SignalVec<T> {
    type Value = Vec<T>;

    fn get_source_id(&self) -> SourceId {
        SourceId::Node(self.id)
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
}

pub struct AtIndex<Parent, T> {
    index: usize,
    parent: Parent,
    id: NodeId,
    _phantom2: PhantomData<*const T>,
}

impl<T: Any> Readable for AtIndex<SignalVec<T>, T> {
    type Value = T;

    fn get_source_id(&self) -> SourceId {
        SourceId::Node(self.id)
    }

    fn with_ref<R>(&self, cx: &mut dyn super::ReadContext, f: impl FnOnce(&Self::Value) -> R) -> R {
        let scope = cx.scope();
        cx.runtime_mut().track(self.id, scope);
        self.parent.with_inner(cx.runtime(), move |inner| {
            f(inner.values.get(self.index).unwrap())
        })
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
}

struct Inner<T> {
    values: Vec<T>,
    triggers: Vec<Trigger>,
    len_trigger: Option<Trigger>,
}
