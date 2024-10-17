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

use std::{any::Any, cell::RefCell, marker::PhantomData, rc::Rc};

pub use accessor::Accessor;
use accessor::{MappedAccessor, SourceId};
pub use animation::AnimationContext;
pub(crate) use app_state::AppState;
pub use contexts::{BuildContext, ViewContext};
use effect::EffectState;
pub use event_handling::{EventContext, MouseEventContext, handle_window_event};
pub use host_handle::HostHandle;
pub use layout::{LayoutContext, layout_window};
use memo::MemoState;
pub use memo::{Memo, MemoContext};
pub use param::{ParamContext, ParamEditor, ParamSignal};
pub use render::{RenderContext, render_window, invalidate_window};
pub use runtime::*;
pub use signal::Signal;
use signal::SignalState;
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
    fn get_node_value_ref_untracked<'a>(&'a self, signal_id: NodeId) -> &'a dyn Any;
    fn get_node_value_ref<'a>(&'a mut self, signal_id: NodeId) -> &'a dyn Any;
    fn get_parameter_ref_untracked<'a>(&'a self, parameter_id: ParameterId) -> ParamRef<'a>;
    fn get_parameter_ref<'a>(&'a mut self, parameter_id: ParameterId) -> ParamRef<'a>;
}

pub trait SignalContext: SignalGetContext {
    fn set_signal_value<T: Any>(&mut self, signal: &Signal<T>, value: T);
}

pub trait UntrackedSignalContext {}

pub trait SignalGet {
    type Value;

	fn get_source_id(&self) -> SourceId;

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

	fn map<R, F: Fn(&Self::Value) -> R>(self, f: F) -> Mapped<Self, Self::Value, R, F> 
    where   
        Self: Sized
    {
        Mapped {
            parent: self,
            f,
            _marker: PhantomData,
        }
    }
}


#[derive(Clone, Copy)]
pub struct Mapped<S, T, R, F> {
    parent: S,
    f: F,
    _marker: PhantomData<fn(&T) -> R>
}

impl<S, T, R, F> SignalGet for Mapped<S, T, R, F> 
where
	S: SignalGet<Value = T>,
    T: Any,
    F: Fn(&T) -> R
{
    type Value = R;

	fn get_source_id(&self) -> SourceId {
        self.parent.get_source_id()
    }

    fn with_ref<R2>(&self, cx: &mut dyn SignalGetContext, f: impl FnOnce(&Self::Value) -> R2) -> R2 {
        f(&self.parent.with_ref(cx, |x| (self.f)(x)))
    }

    fn with_ref_untracked<R2>(&self, cx: &dyn SignalGetContext, f: impl FnOnce(&Self::Value) -> R2) -> R2 {
        f(&self.parent.with_ref_untracked(cx, |x| (self.f)(x)))
    }

	fn get(&self, cx: &mut dyn SignalGetContext) -> Self::Value 
	where 
		Self::Value: Clone 
	{
		self.parent.with_ref(cx, |x| (self.f)(x))
	}
}

impl<S, T, B, F> MappedAccessor<B> for Mapped<S, T, B, F> 
where
    S: SignalGet<Value = T> + Clone + 'static,
    T: Any + Clone,
    B: Any + Clone,
    F: Fn(&T) -> B + Clone + 'static
{
    fn get_source_id(&self) -> SourceId {
        SignalGet::get_source_id(self)
    }

    fn get_ref(&self, ctx: &mut dyn SignalGetContext) -> B {
        self.parent.with_ref(ctx, &self.f)
    }

    fn get_ref_untracked(&self, ctx: &dyn SignalGetContext) -> B {
        self.parent.with_ref_untracked(ctx, &self.f)
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

pub trait SignalCreator {
    fn create_signal_node(&mut self, state: SignalState) -> NodeId;
    fn create_memo_node(&mut self, state: MemoState) -> NodeId;
    fn create_effect_node(&mut self, state: EffectState) -> NodeId;
}