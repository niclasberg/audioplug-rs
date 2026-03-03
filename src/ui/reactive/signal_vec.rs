use std::{any::Any, marker::PhantomData};

use super::{
    CanCreate, CanWrite, NodeId, Owner, ReactiveValue, Trigger, runtime::NodeType, var::SignalState,
};
use crate::ui::{
    ViewProp,
    prelude::CanRead,
    reactive::{ReadContext, WatchContext, WriteContext},
};

#[derive(Copy, Clone)]
pub struct SignalVec<T> {
    id: NodeId,
    _phantom: PhantomData<*mut T>,
}

impl<T: Any> SignalVec<T> {
    pub fn new<'cx>(cx: &mut impl CanCreate<'cx>) -> Self {
        let state = SignalState::new(Inner::<T> {
            values: Vec::new(),
            triggers: Vec::new(),
            len_trigger: None,
        });
        let id = cx.create_context().create_var_node(state);
        Self {
            id,
            _phantom: PhantomData,
        }
    }

    pub fn push<'cx>(&self, cx: &mut impl CanWrite<'cx>, value: T) {
        let mut write_context = cx.write_context();
        let trigger = Trigger::new(&mut write_context.as_create_context(Owner::Node(self.id)));
        self.with_inner_mut(&mut write_context, move |inner| {
            inner.values.push(value);
            inner.triggers.push(trigger);
        });
        write_context.notify(self.id);
    }

    pub fn extend<'s>(&self, cx: &mut impl CanWrite<'s>, iter: impl IntoIterator<Item = T>) {
        let mut write_context = cx.write_context();
        self.with_inner_mut(&mut write_context, move |inner| {
            inner.values.extend(iter);
        });
        write_context.notify(self.id);
    }

    pub fn retain<'s>(&self, cx: &mut impl CanWrite<'s>, f: impl Fn(&T) -> bool) {
        let mut write_context = cx.write_context();
        self.with_inner_mut(&mut write_context, move |inner| {
            inner.values.retain(f);
        });
        write_context.notify(self.id);
    }

    fn with_inner<R>(&self, cx: ReadContext, f: impl FnOnce(&Inner<T>) -> R) -> R {
        let value = match &cx.reactive_graph.get_node(self.id).node_type {
            NodeType::Signal(signal) => signal.value.as_ref(),
            _ => unreachable!(),
        };
        f(value.downcast_ref().expect("Signal had wrong type"))
    }

    fn with_inner_mut<R>(&self, cx: &mut WriteContext, f: impl FnOnce(&mut Inner<T>) -> R) -> R {
        let value = match &mut cx.get_node_mut(self.id).node_type {
            NodeType::Signal(signal) => signal.value.as_mut(),
            _ => unreachable!(),
        };
        let inner: &mut Inner<T> = value.downcast_mut().expect("Signal had wrong type");
        let size_before = inner.values.len();
        let result = f(inner);
        if let Some(len_trigger) = inner.len_trigger
            && size_before != inner.values.len()
        {
            len_trigger.notify(cx);
        }
        result
    }
}

impl<T> From<SignalVec<T>> for ViewProp<Vec<T>> {
    fn from(value: SignalVec<T>) -> Self {
        todo!()
    }
}

impl<T: Any> ReactiveValue for SignalVec<T> {
    type Value = Vec<T>;

    fn track<'cx>(&self, cx: &mut impl CanRead<'cx>) {
        cx.read_context().track(self.id);
    }

    fn with_ref<'cx, R>(&self, cx: &mut impl CanRead<'cx>, f: impl FnOnce(&Self::Value) -> R) -> R {
        let mut cx = cx.read_context();
        cx.track(self.id);
        self.with_inner(cx, move |value| f(&value.values))
    }

    fn with_ref_untracked<'cx, R>(
        &self,
        cx: &mut impl CanRead<'cx>,
        f: impl FnOnce(&Self::Value) -> R,
    ) -> R {
        self.with_inner(cx.read_context(), move |value| f(&value.values))
    }

    fn watch<'cx, F>(self, cx: &mut impl CanCreate<'cx>, f: F) -> super::Effect
    where
        F: FnMut(&mut WatchContext, &Self::Value) + 'static,
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

impl<T: Any> From<AtIndex<SignalVec<T>, T>> for ViewProp<T> {
    fn from(value: AtIndex<SignalVec<T>, T>) -> Self {
        todo!()
    }
}

impl<T: Any> ReactiveValue for AtIndex<SignalVec<T>, T> {
    type Value = T;

    fn track<'cx>(&self, cx: &mut impl CanRead<'cx>) {
        cx.read_context().track(self.id);
    }

    fn with_ref_untracked<'cx, R>(
        &self,
        cx: &mut impl CanRead<'cx>,
        f: impl FnOnce(&Self::Value) -> R,
    ) -> R {
        self.parent.with_inner(cx.read_context(), move |inner| {
            f(inner.values.get(self.index).unwrap())
        })
    }

    fn watch<'cx, F>(self, cx: &mut impl CanCreate<'cx>, f: F) -> super::Effect
    where
        F: FnMut(&mut WatchContext, &Self::Value) + 'static,
    {
        todo!()
    }
}

struct Inner<T> {
    values: Vec<T>,
    triggers: Vec<Trigger>,
    len_trigger: Option<Trigger>,
}
