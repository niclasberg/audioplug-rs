use std::{any::Any, marker::PhantomData, ops::DerefMut};

use super::{accessor::SourceId, NodeId, Runtime, SignalCreator, SignalGet, SignalGetContext};

pub struct MemoContext<'a> {
    pub(super) memo_id: NodeId,
    pub(super) runtime: &'a mut Runtime
}

impl<'b> SignalGetContext for MemoContext<'b> {
    fn get_node_value_ref<'a>(&'a mut self, signal_id: NodeId) -> &'a dyn Any {
        self.runtime.subscriptions.add_node_subscription(signal_id, self.memo_id);
        self.runtime.get_node_value_ref(signal_id)
    }

    fn get_parameter_ref_untracked<'a>(&'a self, parameter_id: crate::param::ParameterId) -> crate::param::ParamRef<'a> {
        self.runtime.get_parameter_ref_untracked(parameter_id)
    }

    fn get_parameter_ref<'a>(&'a mut self, parameter_id: crate::param::ParameterId) -> crate::param::ParamRef<'a> {
        self.runtime.subscriptions.add_parameter_subscription(parameter_id, self.memo_id);
        self.runtime.get_parameter_ref_untracked(parameter_id)
    }
}

#[derive(Clone, Copy)]
pub struct Memo<T> {
    pub(super) id: NodeId,
    _marker: PhantomData<fn() -> T>
}

impl<T: 'static + PartialEq> Memo<T> {
    pub fn new(cx: &mut impl SignalCreator, f: impl Fn(&mut MemoContext, Option<&T>) -> T + 'static) -> Self {
        let state = MemoState {
            f: Box::new(move |cx, value| {
                if let Some(value) = value {
                    let value = value.deref_mut().downcast_mut::<T>().unwrap();
                    let new_value = f(cx, Some(&value));
                    if *value != new_value {
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
            value: None
        };
        let id = cx.create_memo_node(state);

        Self {
            id,
            _marker: PhantomData
        }
    }
}

impl<T: 'static> SignalGet for Memo<T> {
    type Value = T;

	fn get_source_id(&self) -> SourceId {
		SourceId::Node(self.id)
	}

    fn with_ref<R>(&self, cx: &mut dyn SignalGetContext, f: impl FnOnce(&Self::Value) -> R) -> R {
        let value = cx.get_node_value_ref(self.id).downcast_ref().expect("Memo had wrong type");
        f(value)
    }
}

pub struct MemoState {
	pub(super) f: Box<dyn Fn(&mut MemoContext, &mut Option<Box<dyn Any>>) -> bool>,
	pub(super) value: Option<Box<dyn Any>>,
}

impl MemoState {
    pub fn eval(&mut self, cx: &mut MemoContext) -> bool {
        (self.f)(cx, &mut self.value)
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};
    use crate::{app::{AppState, Effect, Signal, SignalSet}, param::ParameterMap};
    use super::*;

    fn with_appstate(f: impl FnOnce(&mut AppState)) {
        let mut cx = AppState::new(ParameterMap::new(()));
        f(&mut cx);
    }

    #[test]
    fn memo() {
        with_appstate(|cx| {
            let signal = Signal::new(cx, 1);
            let memo = Memo::new(cx, move |cx, _| signal.get(cx) * 2);
            assert_eq!(memo.get(cx), 2);
            signal.set(cx, 2);
            assert_eq!(memo.get(cx), 4);
            signal.set(cx, 3);
            assert_eq!(memo.get(cx), 6);
        });
    }

    #[test]
    fn effect_in_diamond_should_run_once() {
        with_appstate(|cx| {
            let signal = Signal::new(cx, 1);
            let memo1 = Memo::new(cx, move |cx, _| signal.get(cx) * 2);
            let memo2 = Memo::new(cx, move |cx, _| signal.get(cx) * 3);

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