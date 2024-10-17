use std::{any::Any, marker::PhantomData};

use super::{accessor::SourceId, NodeId, Runtime, SignalCreator, SignalGet, SignalGetContext};

pub struct MemoContext<'a> {
    pub(super) memo_id: NodeId,
    pub(super) runtime: &'a mut Runtime
}

impl<'b> SignalGetContext for MemoContext<'b> {
    fn get_node_value_ref_untracked<'a>(&'a self, node_id: NodeId) -> &'a dyn Any {
        self.runtime.get_node_value_ref_untracked(node_id)
    }

    fn get_node_value_ref<'a>(&'a mut self, signal_id: NodeId) -> &'a dyn Any {
        self.runtime.add_subscription(signal_id, self.memo_id);
        self.runtime.get_node_value_ref_untracked(signal_id)
    }

    fn get_parameter_ref_untracked<'a>(&'a self, parameter_id: crate::param::ParameterId) -> crate::param::ParamRef<'a> {
        self.runtime.get_parameter_ref_untracked(parameter_id)
    }

    fn get_parameter_ref<'a>(&'a mut self, parameter_id: crate::param::ParameterId) -> crate::param::ParamRef<'a> {
        self.runtime.add_parameter_subscription(parameter_id, self.memo_id);
        self.runtime.get_parameter_ref_untracked(parameter_id)
    }
}

#[derive(Clone, Copy)]
pub struct Memo<T> {
    pub(super) id: NodeId,
    _marker: PhantomData<fn() -> T>
}

impl<T: 'static> Memo<T> {
    pub fn new(cx: &mut impl SignalCreator, f: impl Fn(&mut MemoContext) -> T + 'static) -> Self {
        let state = MemoState::new(move |cx| Box::new(f(cx)));
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

    fn with_ref<R>(&self, _ctx: &mut dyn SignalGetContext, _f: impl FnOnce(&Self::Value) -> R) -> R {
        //f(ctx.get_memo_value_ref(self))
        todo!()
    }

    fn with_ref_untracked<R>(&self, _ctx: &dyn SignalGetContext, _f: impl FnOnce(&Self::Value) -> R) -> R {
        //f(ctx.get_memo_value_ref_untracked(self))
        todo!()
    }
}

pub struct MemoState {
	pub(super) f: Box<dyn Fn(&mut MemoContext) -> Box<dyn Any>>,
	pub(super) value: Option<Box<dyn Any>>,
}

impl MemoState {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&mut MemoContext) -> Box<dyn Any> + 'static
    {
        Self {
            f: Box::new(f),
            value: None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{app::{Signal, SignalSet}, param::ParameterMap};

    use super::*;

    fn with_runtime(f: impl FnOnce(&mut Runtime)) {
        let mut cx = Runtime::new(ParameterMap::new(()));
        f(&mut cx);
    }

    #[test]
    fn memo() {
        with_runtime(|cx| {
            let signal = Signal::new(cx, 1);
            let memo = Memo::new(cx, move |cx| signal.get(cx) * 2);
            assert_eq!(memo.get(cx), 2);
            signal.set(cx, 2);
            assert_eq!(memo.get(cx), 4);
        });
    }

}