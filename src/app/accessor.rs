use std::any::Any;

use super::{Memo, NodeId, Signal, SignalGet};

pub enum Accessor<T> {
    Signal(Signal<T>),
    Memo(Memo<T>),
	AnyGetter {
		state: Box<dyn Any>,
		source_id: NodeId,
		with_ref: fn(&Box<dyn Any>) -> &T,
		with_ref_untracked: fn(&Box<dyn Any>) -> &T,
	},
    Const(T)
}

impl<T> Accessor<T> {
    pub(super) fn get_source_id(&self) -> Option<NodeId> {
        match self {
            Accessor::Signal(signal) => Some(signal.id),
            Accessor::Memo(memo) => Some(memo.id),
			Accessor::AnyGetter { source_id, ..} => Some(*source_id),
            Accessor::Const(_) => None,
        }
    }
}

impl<T> From<Signal<T>> for Accessor<T> {
    fn from(value: Signal<T>) -> Self {
        Self::Signal(value)
    }
}

impl<T> From<Memo<T>> for Accessor<T> {
    fn from(value: Memo<T>) -> Self {
        Self::Memo(value)
    }
}

impl<T> From<T> for Accessor<T> {
    fn from(value: T) -> Self {
        Self::Const(value)
    }
}

impl From<&str> for Accessor<String> {
    fn from(value: &str) -> Self {
        Self::Const(value.to_string())
    }
}

impl<T: 'static> SignalGet for Accessor<T> {
    type Value = T;

    fn with_ref<R>(&self, cx: &mut impl super::SignalContext, f: impl FnOnce(&Self::Value) -> R) -> R {
        match self {
            Accessor::Signal(signal) => signal.with_ref(cx, f),
            Accessor::Memo(memo) => memo.with_ref(cx, f),
            Accessor::Const(value) => f(value),
			Accessor::AnyGetter { state, with_ref, .. } => f(with_ref(state)),
			
        }
    }

    fn with_ref_untracked<R>(&self, cx: &impl super::SignalContext, f: impl FnOnce(&Self::Value) -> R) -> R {
        match self {
            Accessor::Signal(signal) => signal.with_ref_untracked(cx, f),
            Accessor::Memo(memo) => memo.with_ref_untracked(cx, f),
            Accessor::Const(value) => f(value),
			Accessor::AnyGetter { state, with_ref_untracked, .. } => f(with_ref_untracked(state)),
        }
    }
}