use std::{any::Any, hash::Hash, marker::PhantomData};

use super::{
    accessor::{MappedAccessor, SourceId},
    Owner, Runtime, Scope, WindowId,
};

pub trait ReactiveContext {
    fn runtime(&self) -> &Runtime;
    fn runtime_mut(&mut self) -> &mut Runtime;
    fn as_create_context(&mut self, owner: Owner) -> LocalCreateContext {
        LocalCreateContext {
            runtime: self.runtime_mut(),
            owner,
        }
    }
}

pub trait CreateContext: ReactiveContext {
    fn owner(&self) -> Option<Owner>;
}

pub struct LocalCreateContext<'a> {
    runtime: &'a mut Runtime,
    owner: Owner,
}

impl<'a> ReactiveContext for LocalCreateContext<'a> {
    fn runtime(&self) -> &Runtime {
        self.runtime
    }

    fn runtime_mut(&mut self) -> &mut Runtime {
        self.runtime
    }
}

impl<'a> CreateContext for LocalCreateContext<'a> {
    fn owner(&self) -> Option<Owner> {
        Some(self.owner)
    }
}

pub trait ViewContext: CreateContext {
    fn window_id(&self) -> WindowId;
}

pub trait ReadContext: ReactiveContext {
    fn scope(&self) -> Scope;
}

pub trait WriteContext: ReactiveContext {}

pub trait Readable {
    type Value;

    fn get_source_id(&self) -> SourceId;

    /// Map the current value using `f` and subscribe to changes
    fn with_ref<R>(&self, cx: &mut dyn ReadContext, f: impl FnOnce(&Self::Value) -> R) -> R;

    /// Get the current value and subscribe to changes
    fn get(&self, cx: &mut dyn ReadContext) -> Self::Value
    where
        Self::Value: Clone,
    {
        self.with_ref(cx, Self::Value::clone)
    }

    fn map<R, F: Fn(&Self::Value) -> R>(self, f: F) -> Mapped<Self, Self::Value, R, F>
    where
        Self: Sized,
    {
        Mapped {
            parent: self,
            f,
            _marker: PhantomData,
        }
    }

    fn with_key<T, K, F>(self, f: F) -> Keyed<Self, F>
    where
        Self: Sized,
        Self::Value: IntoIterator<Item = T>,
        K: Hash + PartialEq + Copy,
        F: Fn(&T) -> K,
    {
        Keyed {
            signal: self,
            key_fn: f,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Mapped<S, T, R, F> {
    parent: S,
    f: F,
    _marker: PhantomData<fn(&T) -> R>,
}

impl<S, T, R, F> Readable for Mapped<S, T, R, F>
where
    S: Readable<Value = T>,
    T: Any,
    F: Fn(&T) -> R,
{
    type Value = R;

    fn get_source_id(&self) -> SourceId {
        self.parent.get_source_id()
    }

    fn with_ref<R2>(&self, cx: &mut dyn ReadContext, f: impl FnOnce(&Self::Value) -> R2) -> R2 {
        f(&self.parent.with_ref(cx, |x| (self.f)(x)))
    }

    fn get(&self, cx: &mut dyn ReadContext) -> Self::Value {
        self.parent.with_ref(cx, |x| (self.f)(x))
    }
}

impl<S, T, B, F> MappedAccessor<B> for Mapped<S, T, B, F>
where
    S: Readable<Value = T> + 'static,
    T: Any,
    B: Any,
    F: Fn(&T) -> B + 'static,
{
    fn get_source_id(&self) -> SourceId {
        Readable::get_source_id(self)
    }

    fn evaluate(&self, ctx: &mut dyn ReadContext) -> B {
        self.parent.with_ref(ctx, &self.f)
    }
}

pub struct Keyed<S, F> {
    signal: S,
    key_fn: F,
}

impl<S: Readable, F> Readable for Keyed<S, F> {
    type Value = S::Value;

    fn get_source_id(&self) -> SourceId {
        self.signal.get_source_id()
    }

    fn with_ref<R>(&self, cx: &mut dyn ReadContext, f: impl FnOnce(&Self::Value) -> R) -> R {
        self.signal.with_ref(cx, f)
    }

    fn get(&self, cx: &mut dyn ReadContext) -> Self::Value
    where
        Self::Value: Clone,
    {
        self.signal.get(cx)
    }
}
