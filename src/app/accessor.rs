use crate::param::ParameterId;
use super::{Memo, NodeId, ParamSignal, Signal, SignalGet};

pub(super) enum SourceId {
	None,
	Parameter(ParameterId),
	Node(NodeId)
}

pub enum Accessor<T> {
    Signal(Signal<T>),
    Memo(Memo<T>),
	Parameter(ParamSignal<T>),
    Const(T)
}

impl<T> Accessor<T> {
    pub(super) fn get_source_id(&self) -> SourceId {
        match self {
            Accessor::Signal(signal) => SourceId::Node(signal.id),
            Accessor::Memo(memo) => SourceId::Node(memo.id),
			Accessor::Parameter(param) => SourceId::Parameter(param.id),
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

impl<T> From<ParamSignal<T>> for Accessor<T> {
	fn from(value: ParamSignal<T>) -> Self {
		Self::Parameter(value)
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
			Accessor::Parameter(param) => param.with_ref(cx, f),
        }
    }

    fn with_ref_untracked<R>(&self, cx: &impl super::SignalContext, f: impl FnOnce(&Self::Value) -> R) -> R {
        match self {
            Accessor::Signal(signal) => signal.with_ref_untracked(cx, f),
            Accessor::Memo(memo) => memo.with_ref_untracked(cx, f),
            Accessor::Const(value) => f(value),
			Accessor::Parameter(param) => param.with_ref_untracked(cx, f),
        }
    }
}