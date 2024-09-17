mod accessor;
mod app_state;
mod binding;
mod contexts;
mod effect;
mod event_handling;
mod host_handle;
mod layout;
mod memo;
mod param;
mod render;
mod signal;
mod runtime;
mod widget;
mod widget_node;
mod window;

use std::{any::Any, cell::RefCell, rc::Rc};

pub use accessor::Accessor;
pub(crate) use app_state::AppState;
pub use contexts::BuildContext;
pub use event_handling::{EventContext, MouseEventContext, handle_window_event};
pub use host_handle::HostHandle;
pub use layout::{LayoutContext, layout_window};
pub use memo::Memo;
pub use param::{ParamContext, ParamEditor, ParamSignal};
pub use render::{RenderContext, render_window, invalidate_window};
pub use runtime::*;
pub use signal::Signal;
use slotmap::new_key_type;
pub use widget::{EventStatus, StatusChange, Widget};
pub use widget_node::{WidgetData, WidgetRef, WidgetMut, WidgetId};
pub use window::{AppContext, Window};
pub(crate) use window::MyHandler;

use crate::platform;

new_key_type! {
    pub struct NodeId;
}

new_key_type! {
    pub struct WindowId;
}

pub struct App {
    native: platform::Application,
    pub(crate) state: Rc<RefCell<AppState>>,
}
impl App {
    pub fn new() -> Self {
        Self::new_with_app_state(Rc::new(RefCell::new(AppState::new(()))))
    }

    pub fn new_with_app_state(state: Rc<RefCell<AppState>>) -> Self {
        Self {
            native: platform::Application::new(),
            state
        }
    }

    pub fn run(&mut self) {
        self.native.run()
    }
}

pub trait SignalContext {
    fn get_signal_value_ref_untracked<'a, T: Any>(&'a self, signal: &Signal<T>) -> &'a T;
    fn get_signal_value_ref<'a, T: Any>(&'a mut self, signal: &Signal<T>) -> &'a T;
    fn get_parameter_value_untracked<T: Any>(&self, parameter: &ParamSignal<T>) -> T;
    fn get_parameter_value<T: Any>(&mut self, parameter: &ParamSignal<T>) -> T;
    fn set_signal_value<T: Any>(&mut self, signal: &Signal<T>, value: T);
}

pub trait UntrackedSignalContext {}

pub trait SignalGet {
    type Value;

    /// Map the current value using `f` and subscribe to changes
    fn with_ref<R>(&self, cx: &mut impl SignalContext, f: impl FnOnce(&Self::Value) -> R) -> R;

    /// Get the current value and subscribe to changes
    fn get(&self, cx: &mut impl SignalContext) -> Self::Value
    where
        Self::Value: Clone,
    {
        self.with_ref(cx, Self::Value::clone)
    }

    fn with_ref_untracked<R>(
        &self,
        cx: &impl SignalContext,
        f: impl FnOnce(&Self::Value) -> R,
    ) -> R;

    fn get_untracked(&self, cx: &impl SignalContext) -> Self::Value
    where
        Self::Value: Clone,
    {
        self.with_ref_untracked(cx, Self::Value::clone)
    }
}

#[derive(Clone, Copy)]
pub struct Map<S, F> {
    source: S,
    f: F,
}

impl<B, S: SignalGet, F> SignalGet for Map<S, F>
where
    F: Fn(&S::Value) -> B,
{
    type Value = B;

    fn with_ref<R>(&self, cx: &mut impl SignalContext, f: impl FnOnce(&Self::Value) -> R) -> R {
        f(&self.source.with_ref(cx, |x| (self.f)(x)))
    }

    fn with_ref_untracked<R>(
        &self,
        cx: &impl SignalContext,
        f: impl FnOnce(&Self::Value) -> R,
    ) -> R {
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