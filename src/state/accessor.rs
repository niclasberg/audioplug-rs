use super::{Memo, Signal, SignalGet};

pub enum Accessor<T> {
    Signal(Signal<T>),
    Memo(Memo<T>),
    Const(T)
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

impl<T: 'static> SignalGet for Accessor<T> {
    type Value = T;

    fn with_ref<R>(&self, cx: &mut dyn super::SignalContext, f: impl Fn(&Self::Value) -> R) -> R {
        match self {
            Accessor::Signal(signal) => signal.with_ref(cx, f),
            Accessor::Memo(memo) => memo.with_ref(cx, f),
            Accessor::Const(value) => f(value),
        }
    }

    fn with_ref_untracked<R>(&self, cx: &dyn super::SignalContext, f: impl Fn(&Self::Value) -> R) -> R {
        match self {
            Accessor::Signal(signal) => signal.with_ref_untracked(cx, f),
            Accessor::Memo(memo) => memo.with_ref_untracked(cx, f),
            Accessor::Const(value) => f(value),
        }
    }
}