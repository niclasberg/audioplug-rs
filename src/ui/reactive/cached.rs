use std::{any::Any, marker::PhantomData, ops::DerefMut};

use crate::ui::{ViewProp, Widgets};

use super::{
    CreateContext, Effect, LocalContext, NodeId, ReactiveContext, ReactiveGraph, ReactiveValue,
    ReadContext, ReadScope, ReadSignal, runtime::NodeType,
};

pub struct CachedContext<'a> {
    pub(super) memo_id: NodeId,
    pub(super) cx: LocalContext<'a>,
}

impl ReactiveContext for CachedContext<'_> {
    fn reactive_graph_and_widgets(&self) -> (&ReactiveGraph, &Widgets) {
        self.cx.reactive_graph_and_widgets()
    }

    fn reactive_graph_mut_and_widgets(&mut self) -> (&mut ReactiveGraph, &Widgets) {
        self.cx.reactive_graph_mut_and_widgets()
    }
}

impl ReadContext for CachedContext<'_> {
    fn scope(&self) -> ReadScope {
        ReadScope::Node(self.memo_id)
    }
}

pub struct Cached<T> {
    pub(super) id: NodeId,
    _marker: PhantomData<fn() -> T>,
}

impl<T> Clone for Cached<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Cached<T> {}

impl<T: Any> Cached<T> {
    pub fn new(
        cx: &mut dyn CreateContext,
        f: impl Fn(&mut CachedContext, Option<&T>) -> T + 'static,
    ) -> Self
    where
        T: PartialEq,
    {
        Self::new_with_compare(cx, f, PartialEq::eq)
    }

    pub fn new_with_compare(
        cx: &mut dyn CreateContext,
        f: impl Fn(&mut CachedContext, Option<&T>) -> T + 'static,
        compare: fn(&T, &T) -> bool,
    ) -> Self {
        let state = CachedState {
            f: Box::new(move |cx, value| {
                if let Some(value) = value {
                    let value = value.deref_mut().downcast_mut::<T>().unwrap();
                    let new_value = f(cx, Some(value));
                    if !compare(value, &new_value) {
                        *value = new_value;
                        true
                    } else {
                        false
                    }
                } else {
                    let new_value = f(cx, None);
                    *value = Some(Box::new(new_value));
                    true
                }
            }),
            value: None,
        };
        let id = cx.create_memo_node(state);

        Self {
            id,
            _marker: PhantomData,
        }
    }

    pub fn as_read_signal(self) -> ReadSignal<T> {
        ReadSignal::from_node(self.id)
    }
}

impl<T: 'static> From<Cached<T>> for ViewProp<T> {
    fn from(value: Cached<T>) -> Self {
        Self::ReadSignal(value.as_read_signal())
    }
}

impl<T: 'static> ReactiveValue for Cached<T> {
    type Value = T;

    fn track(&self, cx: &mut dyn ReadContext) {
        cx.track(self.id);
    }

    fn with_ref_untracked<R>(
        &self,
        cx: &mut dyn ReactiveContext,
        f: impl FnOnce(&Self::Value) -> R,
    ) -> R {
        f(update_and_get_memo_value(cx, self.id)
            .downcast_ref()
            .expect("Memo had wrong type"))
    }

    fn watch<F>(self, cx: &mut dyn CreateContext, f: F) -> Effect
    where
        F: FnMut(&mut dyn super::WatchContext, &Self::Value) + 'static,
    {
        Effect::watch_node(cx, self.id, f)
    }
}

fn update_and_get_memo_value(cx: &mut dyn ReactiveContext, id: NodeId) -> &dyn Any {
    super::update_value_if_needed(cx, id);
    match &cx.reactive_graph().get_node(id).node_type {
        NodeType::Memo(state) => state
            .value
            .as_ref()
            .expect("Memo should have been evaluated before accessed")
            .as_ref(),
        _ => unreachable!(),
    }
}

type MemoFn = dyn Fn(&mut CachedContext, &mut Option<Box<dyn Any>>) -> bool;

pub struct CachedState {
    pub(super) f: Box<MemoFn>,
    pub(super) value: Option<Box<dyn Any>>,
}

impl CachedState {
    pub fn eval(&mut self, memo_id: NodeId, cx: LocalContext) -> bool {
        let mut cx = CachedContext { memo_id, cx };
        (self.f)(&mut cx, &mut self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        param::ParameterMap,
        ui::AppState,
        ui::reactive::{Effect, Var},
    };
    use std::{cell::Cell, rc::Rc};

    fn with_appstate(f: impl FnOnce(&mut AppState)) {
        let mut cx = AppState::new(ParameterMap::new(()));
        f(&mut cx);
    }

    #[test]
    fn memo() {
        with_appstate(|cx| {
            let signal = Var::new(cx, 1);
            let memo = Cached::new(cx, move |cx, _| signal.get(cx) * 2);
            assert_eq!(memo.get(cx), 2);
            signal.set(cx, 2);
            assert_eq!(memo.get(cx), 4);
            signal.set(cx, 3);
            assert_eq!(memo.get_untracked(cx), 6);
            assert_eq!(memo.get(cx), 6);
        });
    }

    #[test]
    fn effect_in_diamond_should_run_once() {
        with_appstate(|cx| {
            let signal = Var::new(cx, 1);
            let memo1 = Cached::new(cx, move |cx, _| signal.get(cx) * 2);
            let memo2 = Cached::new(cx, move |cx, _| signal.get(cx) * 3);

            let call_count = Rc::new(Cell::new(0));
            let _call_count = call_count.clone();
            Effect::new(cx, move |cx| {
                _call_count.replace(_call_count.get() + 1);
                let _ = memo1.get(cx) + memo2.get(cx);
            });
            cx.run_effects();

            signal.set(cx, 2);
            cx.run_effects();
            assert_eq!(call_count.get(), 2);
        });
    }
}
