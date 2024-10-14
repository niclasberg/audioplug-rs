mod accessor;
mod animation;
mod app_state;
mod binding;
mod contexts;
mod effect;
mod event_handling;
mod host_handle;
mod layout;
mod memo;
mod overlay;
mod param;
mod render;
mod signal;
mod runtime;
mod widget;
mod widget_node;
mod window;

use std::{any::Any, cell::RefCell, rc::Rc};

pub use accessor::Accessor;
pub use animation::AnimationContext;
pub(crate) use app_state::AppState;
pub use contexts::{BuildContext, ViewContext};
pub use event_handling::{EventContext, MouseEventContext, handle_window_event};
pub use host_handle::HostHandle;
pub use layout::{LayoutContext, layout_window};
pub use memo::Memo;
pub use param::{ParamContext, ParamEditor, ParamSignal};
pub use render::{RenderContext, render_window, invalidate_window};
pub use runtime::*;
pub use signal::Signal;
pub use widget::{EventStatus, StatusChange, Widget};
pub use widget_node::{WidgetData, WidgetRef, WidgetMut, WidgetId};
pub use window::Window;
#[cfg(target_os  ="macos")]
pub(crate) use window::MyHandler;

use crate::{param::{ParamRef, ParameterId, ParameterMap}, platform};

slotmap::new_key_type! {
    pub struct NodeId;
}

slotmap::new_key_type! {
    pub struct WindowId;
}

pub struct App {
    native: platform::Application,
    pub(crate) state: Rc<RefCell<AppState>>,
}

impl App {
    pub fn new() -> Self {
        let executor = Rc::new(platform::Executor::new().unwrap());
        let parameters = ParameterMap::new(());
        Self::new_with_app_state(Rc::new(RefCell::new(AppState::new(parameters, executor))))
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

pub trait SignalGetContext {
    fn get_signal_value_ref_untracked<'a>(&'a self, signal_id: NodeId) -> &'a dyn Any;
    fn get_signal_value_ref<'a>(&'a mut self, signal_id: NodeId) -> &'a dyn Any;
    fn get_parameter_ref_untracked<'a>(&'a self, parameter_id: ParameterId) -> ParamRef<'a>;
    fn get_parameter_ref<'a>(&'a mut self, parameter_id: ParameterId) -> ParamRef<'a>;
}

pub trait SignalContext: SignalGetContext {
    fn set_signal_value<T: Any>(&mut self, signal: &Signal<T>, value: T);
}

pub trait UntrackedSignalContext {}

pub trait SignalGet {
    type Value;

    /// Map the current value using `f` and subscribe to changes
    fn with_ref<R>(&self, cx: &mut dyn SignalGetContext, f: impl FnOnce(&Self::Value) -> R) -> R;

    /// Get the current value and subscribe to changes
    fn get(&self, cx: &mut dyn SignalGetContext) -> Self::Value
    where
        Self::Value: Clone,
    {
        self.with_ref(cx, Self::Value::clone)
    }

    fn with_ref_untracked<R>(
        &self,
        cx: &dyn SignalGetContext,
        f: impl FnOnce(&Self::Value) -> R,
    ) -> R;

    fn get_untracked(&self, cx: &dyn SignalGetContext) -> Self::Value
    where
        Self::Value: Clone,
    {
        self.with_ref_untracked(cx, Self::Value::clone)
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

    /// Set the current value, notifies subscribers
    fn update(&self, cx: &mut impl SignalContext, f: impl FnOnce(&mut Self::Value));
}
