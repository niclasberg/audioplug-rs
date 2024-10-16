use crate::param::ParameterId;
use super::{Mapped, Memo, NodeId, ParamSignal, Signal, SignalGet, SignalGetContext};

pub trait MappedAccessor<T>: CloneMappedAccessor<T> {
    fn get_source_id(&self) -> SourceId;
    fn get_ref(&self, ctx: &mut dyn SignalGetContext) -> T;
    fn get_ref_untracked(&self, ctx: &dyn SignalGetContext) -> T;
}

pub trait CloneMappedAccessor<T> {
    fn box_clone(&self) -> Box<dyn MappedAccessor<T>>;
}

impl<T, V> CloneMappedAccessor<T> for V
where
    V: 'static + MappedAccessor<T> + Clone
{
    fn box_clone(&self) -> Box<dyn MappedAccessor<T>> {
        Box::new(self.clone())
    }
}

impl<T> Clone for Box<dyn MappedAccessor<T>> {
    fn clone(&self) -> Box<dyn MappedAccessor<T>> {
        self.box_clone()
    }
}

pub enum SourceId {
	None,
	Parameter(ParameterId),
	Node(NodeId)
}

#[derive(Clone)]
pub enum Accessor<T> {
    Signal(Signal<T>),
    Memo(Memo<T>),
	Parameter(ParamSignal<T>),
    Const(T),
    Mapped(Box<dyn MappedAccessor<T>>)
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

impl<S, R, F> From<Mapped<S, R, F>> for Accessor<S::Value> 
where
	S: SignalGet,
    Mapped<S, R, F>: MappedAccessor<R> + 'static
{
    fn from(value: Mapped<S, R, F>) -> Self {
        Self::Mapped(Box::new(value))
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

	fn get_source_id(&self) -> SourceId {
		match self {
			Accessor::Signal(signal) => SourceId::Node(signal.id),
            Accessor::Memo(memo) => SourceId::Node(memo.id),
			Accessor::Parameter(param) => SourceId::Parameter(param.id),
            Accessor::Const(_) => SourceId::None,
            Accessor::Mapped(mapped) => mapped.get_source_id()
		}
	}

    fn with_ref<R>(&self, cx: &mut dyn SignalGetContext, f: impl FnOnce(&Self::Value) -> R) -> R {
        match self {
            Accessor::Signal(signal) => signal.with_ref(cx, f),
            Accessor::Memo(memo) => memo.with_ref(cx, f),
            Accessor::Const(value) => f(value),
			Accessor::Parameter(param) => param.with_ref(cx, f),
            Accessor::Mapped(mapped) => f(&mapped.get_ref(cx))
        }
    }

    fn with_ref_untracked<R>(&self, cx: &dyn SignalGetContext, f: impl FnOnce(&Self::Value) -> R) -> R {
        match self {
            Accessor::Signal(signal) => signal.with_ref_untracked(cx, f),
            Accessor::Memo(memo) => memo.with_ref_untracked(cx, f),
            Accessor::Const(value) => f(value),
			Accessor::Parameter(param) => param.with_ref_untracked(cx, f),
            Accessor::Mapped(mapped) => f(&mapped.get_ref_untracked(cx))
        }
    }
}