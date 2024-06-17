mod signal;
mod memo;
mod effect;
mod reactive_graph;
mod accessor;

pub use signal::Signal;
pub use memo::Memo;
use slotmap::new_key_type;

new_key_type! { 
    pub struct NodeId;
} 

pub trait SignalContext {
   
}

pub trait UntrackedSignalContext {
   
}

pub trait SignalGet {
    type Value;

    /// Map the current value using `f` and subscribe to changes
    fn with_ref<R>(&self, cx: &mut dyn SignalContext, f: impl Fn(&Self::Value) -> R) -> R;

    /// Get the current value and subscribe to changes
    fn get(&self, cx: &mut dyn SignalContext) -> Self::Value 
        where Self::Value: Clone 
    {
        self.with_ref(cx, Self::Value::clone)
    }

    fn with_ref_untracked<R>(&self, cx: &dyn SignalContext, f: impl Fn(&Self::Value) -> R) -> R;

    fn get_untracked(&self, cx: &dyn SignalContext) -> Self::Value 
        where Self::Value: Clone 
    {
        self.with_ref_untracked(cx, Self::Value::clone)
    }

    fn map<F, R>(self, f: F) -> Map<Self, F>
    where 
        Self: Sized,
        F: Fn(&Self::Value) -> R
    {
        Map {
            source: self,
            f
        }
    }
}

#[derive(Clone, Copy)]
pub struct Map<S, F> {
    source: S,
    f: F
}

impl<B, S: SignalGet, F> SignalGet for Map<S, F> 
where
    F: Fn(&S::Value) -> B
{
    type Value = B;

    fn with_ref<R>(&self, cx: &mut dyn SignalContext, f: impl Fn(&Self::Value) -> R) -> R {
        f(&self.source.with_ref(cx, |x| (self.f)(x)))
    }

    fn with_ref_untracked<R>(&self, cx: &dyn SignalContext, f: impl Fn(&Self::Value) -> R) -> R {
        f(&self.source.with_ref_untracked(cx, |x| (self.f)(x)))
    }
}

pub trait SignalSet {
    type Value;

    /// Set the current value, notifies subscribers
    fn set(&self, cx: &mut dyn SignalContext, value: Self::Value) {
        self.set_with(cx, move || value)
    }

    /// Set the current value, notifies subscribers
    fn set_with(&self, cx: &mut dyn SignalContext, f: impl FnOnce() -> Self::Value);
}

pub trait SignalUpdate {
    type Value;

    /// Set the current value, notifies subscribers
    fn update(&self, cx: &mut dyn SignalContext, f: impl FnOnce(&mut Self::Value));
}

impl<T: AsRef<T>> SignalGet for T {
    type Value = T;

    fn with_ref<R>(&self, _cx: &mut dyn SignalContext, f: impl FnOnce(&Self::Value) -> R) -> R {
        f(&self)
    }

    fn with_ref_untracked<R>(&self, _cx: &dyn SignalContext, f: impl FnOnce(&Self::Value) -> R) -> R {
        f(&self)
    }
}
