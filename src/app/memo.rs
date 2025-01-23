use std::{any::Any, marker::PhantomData, ops::DerefMut};

use super::{accessor::SourceId, CreateContext, NodeId, NodeType, ReactiveContext, ReadContext, Runtime, Scope, SignalGet};

pub struct MemoContext<'a> {
    pub(super) memo_id: NodeId,
    pub(super) runtime: &'a mut Runtime
}

impl<'b> ReactiveContext for MemoContext<'b> {
    fn runtime(&self) -> &Runtime {
        &self.runtime
    }
    
    fn runtime_mut(&mut self) -> &mut Runtime {
        &mut self.runtime
    }
}

impl<'b> ReadContext for MemoContext<'b> {
    fn scope(&self) -> Scope {
        Scope::Node(self.memo_id)
    }
}

#[derive(Clone, Copy)]
pub struct Memo<T> {
    pub(super) id: NodeId,
    _marker: PhantomData<fn() -> T>
}

impl<T: Any> Memo<T> {
    pub fn new(cx: &mut impl CreateContext, f: impl Fn(&mut MemoContext, Option<&T>) -> T + 'static) -> Self 
	where 
		T: PartialEq
	{
        Self::new_with_compare(cx, f, PartialEq::eq)
    }

	pub fn new_with_compare(
		cx: &mut impl CreateContext, 
		f: impl Fn(&mut MemoContext, Option<&T>) -> T + 'static,
		compare: fn(&T, &T) -> bool
	) -> Self {
		let state = MemoState {
            f: Box::new(move |cx, value| {
                if let Some(value) = value {
                    let value = value.deref_mut().downcast_mut::<T>().unwrap();
                    let new_value = f(cx, Some(&value));
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
            value: None
        };
		let owner = cx.owner();
        let id = cx.runtime_mut().create_memo_node(state, owner);

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

    fn with_ref<R>(&self, cx: &mut dyn ReadContext, f: impl FnOnce(&Self::Value) -> R) -> R {
        let scope = cx.scope();
		cx.runtime_mut().update_if_necessary(self.id);
        let value = match &cx.runtime_mut().get_node_mut(self.id).node_type {
            NodeType::Memo(state) => state.value.as_ref().expect("Memo should have been evaluated before accessed").as_ref(),
            _ => unreachable!()
        };
        let r = f(value.downcast_ref().expect("Memo had wrong type"));
		cx.runtime_mut().track(self.id, scope);
		r
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
    use crate::{app::{AppState, Effect, Signal}, param::ParameterMap};
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