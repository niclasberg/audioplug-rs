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
mod signal_vec;
mod runtime;
mod trigger;
mod widget;
mod widget_data;
mod widget_ref;
mod window;

use std::{any::Any, cell::RefCell, marker::PhantomData, rc::Rc};

pub use accessor::Accessor;
use accessor::{MappedAccessor, SourceId};
pub use animation::AnimationContext;
pub(crate) use app_state::AppState;
pub use contexts::{BuildContext, ViewContext};
pub use effect::{Effect, EffectState};
pub use event_handling::{EventContext, MouseEventContext, handle_window_event};
pub use host_handle::HostHandle;
pub use layout::{LayoutContext, layout_window};
pub use memo::{Memo, MemoContext};
pub use param::{ParamContext, ParamEditor, ParamSignal};
pub use render::{RenderContext, render_window, invalidate_window};
pub use runtime::*;
pub use signal::Signal;
pub use trigger::Trigger;
pub use widget::{EventStatus, StatusChange, Widget};
pub use widget_ref::{WidgetRef, WidgetMut};
pub use widget_data::{WidgetData, WidgetId, WidgetFlags};
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
        let parameters = ParameterMap::new(());
        Self::new_with_app_state(Rc::new(RefCell::new(AppState::new(parameters))))
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

pub trait ReactiveContext {
    fn runtime(&self) -> &Runtime;
    fn runtime_mut(&mut self) -> &mut Runtime;
}

pub trait CreateContext: ReactiveContext {
    fn owner(&self) -> Option<Owner>;
}

pub trait ReadContext: ReactiveContext {
    fn scope(&self) -> Scope;
}

pub trait WriteContext: ReactiveContext {
    
}

pub trait SignalGet {
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

    fn with_ref<R2>(&self, cx: &mut dyn ReadContext, f: impl FnOnce(&Self::Value) -> R2) -> R2 {
        f(&self.parent.with_ref(cx, |x| (self.f)(x)))
    }

	fn get(&self, cx: &mut dyn ReadContext) -> Self::Value {
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

    fn get_ref(&self, ctx: &mut dyn ReadContext) -> B {
        self.parent.with_ref(ctx, &self.f)
    }
}
