use std::any::Any;

use crate::param::{Parameter, ParameterId};

use super::{Memo, NodeId, Signal, SignalGet};

pub(super) enum SourceId {
	None,
	Parameter(ParameterId),
	Node(NodeId)
}

pub enum Accessor<T> {
    Signal(Signal<T>),
    Memo(Memo<T>),
	AnyGetter {
		state: Box<dyn Any>,
		source_id: NodeId,
		with_ref: fn(&Box<dyn Any>) -> &T,
		with_ref_untracked: fn(&Box<dyn Any>) -> &T,
	},
	Parameter {
		id: ParameterId
	},
    Const(T)
}

impl<T> Accessor<T> {
    pub(super) fn get_source_id(&self) -> SourceId {
        match self {
            Accessor::Signal(signal) => SourceId::Node(signal.id),
            Accessor::Memo(memo) => SourceId::Node(memo.id),
			Accessor::AnyGetter { source_id, ..} => SourceId::Node(*source_id),
			Accessor::Parameter { id } => SourceId::Parameter(*id),
            Accessor::Const(_) => SourceId::None,
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
			Accessor::Parameter { id } => todo!(),
        }
    }

    fn with_ref_untracked<R>(&self, cx: &impl super::SignalContext, f: impl FnOnce(&Self::Value) -> R) -> R {
        match self {
            Accessor::Signal(signal) => signal.with_ref_untracked(cx, f),
            Accessor::Memo(memo) => memo.with_ref_untracked(cx, f),
            Accessor::Const(value) => f(value),
			Accessor::AnyGetter { state, with_ref_untracked, .. } => f(with_ref_untracked(state)),
			Accessor::Parameter { id } => todo!(),
        }
    }
}