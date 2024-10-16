use std::{any::Any, marker::PhantomData};

use super::{accessor::SourceId, AppState, NodeId, SignalGet, SignalGetContext};


#[derive(Clone, Copy)]
pub struct Memo<T> {
    pub(super) id: NodeId,
    _marker: PhantomData<T>
}

impl<T> Memo<T> {
    pub fn new(id: NodeId) -> Self {
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

pub(super) struct MemoState {
	f: Box<dyn Fn(&mut AppState) -> Box<dyn Any>>,
	pub(super) value: Option<Box<dyn Any>>,
}

impl MemoState {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&mut AppState) -> Box<dyn Any> + 'static
    {
        Self {
            f: Box::new(f),
            value: None
        }
    }
}