use crate::param::ParameterId;
use super::{Mapped, Memo, NodeId, ParamSignal, Signal, SignalGet, SignalGetContext};

pub trait MappedAccessor<T>: CloneMappedAccessor<T> {
    fn get_source_id(&self) -> SourceId;
    fn get_ref(&self, ctx: &mut dyn SignalGetContext) -> T;
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

#[derive(Clone, Copy)]
pub enum SourceId {
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

impl<T: 'static> Accessor<T> {
    pub fn get_source_id(&self) -> Option<SourceId> {
		match self {
			Accessor::Signal(signal) => Some(SourceId::Node(signal.id)),
            Accessor::Memo(memo) => Some(SourceId::Node(memo.id)),
			Accessor::Parameter(param) => Some(SourceId::Parameter(param.id)),
            Accessor::Const(_) => None,
            Accessor::Mapped(mapped) => Some(mapped.get_source_id())
		}
	}

    pub fn with_ref<R>(&self, cx: &mut dyn SignalGetContext, f: impl FnOnce(&T) -> R) -> R {
        match self {
            Accessor::Signal(signal) => signal.with_ref(cx, f),
            Accessor::Memo(memo) => memo.with_ref(cx, f),
            Accessor::Const(value) => f(value),
			Accessor::Parameter(param) => param.with_ref(cx, f),
            Accessor::Mapped(mapped) => f(&mapped.get_ref(cx))
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

impl<S, T, R, F> From<Mapped<S, T, R, F>> for Accessor<R> 
where
	S: SignalGet,
    Mapped<S, T, R, F>: MappedAccessor<R> + 'static
{
    fn from(value: Mapped<S, T, R, F>) -> Self {
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