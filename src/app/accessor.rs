use std::rc::Rc;

use crate::param::ParameterId;
use super::{Mapped, Memo, NodeId, ParamSignal, ReactiveContext, ReadContext, Runtime, Signal, SignalGet};

pub trait MappedAccessor<T> {
    fn get_source_id(&self) -> SourceId;
    fn evaluate(&self, ctx: &mut dyn ReadContext) -> T;
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
    Mapped(Rc<dyn MappedAccessor<T>>)
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

	pub fn get(&self, cx: &mut dyn ReadContext) -> T where T: Clone {
		match self {
            Accessor::Signal(signal) => signal.get(cx),
            Accessor::Memo(memo) => memo.get(cx),
            Accessor::Const(value) => value.clone(),
			Accessor::Parameter(param) => param.get(cx),
            Accessor::Mapped(mapped) => mapped.evaluate(cx)
        }
	}

    pub fn with_ref<R>(&self, cx: &mut dyn ReadContext, f: impl FnOnce(&T) -> R) -> R {
        match self {
            Accessor::Signal(signal) => signal.with_ref(cx, f),
            Accessor::Memo(memo) => memo.with_ref(cx, f),
            Accessor::Const(value) => f(value),
			Accessor::Parameter(param) => param.with_ref(cx, f),
            Accessor::Mapped(mapped) => f(&mapped.evaluate(cx))
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
        Self::Mapped(Rc::new(value))
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