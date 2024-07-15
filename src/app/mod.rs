mod memo;
mod accessor;
mod effect;
mod app_state;
mod ref_count_map;
mod signal;
mod binding;

use std::{any::Any, cell::RefCell, rc::Rc};

pub use signal::Signal;
pub use memo::Memo;
pub use accessor::Accessor;
pub use binding::Binding;
pub(crate) use app_state::AppState;
pub(crate) use ref_count_map::{RefCountMap, WeakRefCountMap};
use slotmap::new_key_type;

use crate::platform;

new_key_type! { 
    pub struct NodeId;
} 

pub struct App {
    native: platform::Application,
    pub(crate) state: Rc<RefCell<AppState>>,
}

impl App {
    pub fn new() -> Self {
        Self {
            native: platform::Application::new(),
            state: Rc::new(RefCell::new(AppState::new())),
        }
    }

    pub fn run(&mut self) {
        self.native.run()
    }
}

pub trait SignalContext {
    fn get_signal_value_ref_untracked<'a, T: Any>(&'a self, signal: &Signal<T>) -> &'a T;
    fn get_signal_value_ref<'a, T: Any>(&'a mut self, signal: &Signal<T>) -> &'a T;
}



pub trait UntrackedSignalContext {
   
}

pub trait SignalGet {
    type Value;

    /// Map the current value using `f` and subscribe to changes
    fn with_ref<R>(&self, cx: &mut impl SignalContext, f: impl Fn(&Self::Value) -> R) -> R;

    /// Get the current value and subscribe to changes
    fn get(&self, cx: &mut impl SignalContext) -> Self::Value 
        where Self::Value: Clone 
    {
        self.with_ref(cx, Self::Value::clone)
    }

    fn with_ref_untracked<R>(&self, cx: &impl SignalContext, f: impl Fn(&Self::Value) -> R) -> R;

    fn get_untracked(&self, cx: &impl SignalContext) -> Self::Value 
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

    fn with_ref<R>(&self, cx: &mut impl SignalContext, f: impl Fn(&Self::Value) -> R) -> R {
        f(&self.source.with_ref(cx, |x| (self.f)(x)))
    }

    fn with_ref_untracked<R>(&self, cx: &impl SignalContext, f: impl Fn(&Self::Value) -> R) -> R {
        f(&self.source.with_ref_untracked(cx, |x| (self.f)(x)))
    }
}

pub trait SignalSet {
    type Value;

    /// Set the current value, notifies subscribers
    fn set(&self, cx: &mut impl SignalContext, value: Self::Value) {
        self.set_with(cx, move || value)
    }

    /// Set the current value, notifies subscribers
    fn set_with(&self, cx: &mut impl SignalContext, f: impl FnOnce() -> Self::Value);
}

pub trait SignalUpdate {
    type Value;

    /// Set the current value, notifies subscribers
    fn update(&self, cx: &mut impl SignalContext, f: impl FnOnce(&mut Self::Value));
}

impl<T: AsRef<T>> SignalGet for T {
    type Value = T;

    fn with_ref<R>(&self, _cx: &mut impl SignalContext, f: impl FnOnce(&Self::Value) -> R) -> R {
        f(&self)
    }

    fn with_ref_untracked<R>(&self, _cx: &impl SignalContext, f: impl FnOnce(&Self::Value) -> R) -> R {
        f(&self)
    }
}
