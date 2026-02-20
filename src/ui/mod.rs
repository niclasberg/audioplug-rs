mod animation;
mod app_state;
mod clipboard;
mod event_handling;
mod host_handle;
mod layout;
mod overlay;
pub(super) mod reactive;
mod render;
pub mod style;
mod task_queue;
mod view;
mod view_sequence;
mod widget;
mod widget_data;
mod widget_ref;
mod widgets;
mod window;

use std::{cell::RefCell, marker::PhantomData, rc::Rc};

pub use animation::AnimationContext;
pub(crate) use app_state::AppState;
pub use event_handling::{CallbackContext, EventContext, MouseEventContext};
pub use host_handle::HostHandle;
pub use layout::{LayoutContext, layout_window};
pub use overlay::{OverlayAnchor, OverlayOptions};

pub use reactive::{
    Accessor, Animated, AnimatedFn, Animation, Cached, CachedContext, Computed, CreateContext,
    Easing, Effect, EffectContext, EventChannel, EventReceiver, Mapped, Node, NodeId, Owner,
    ParamContext, ParamSetter, ReactiveContext, ReactiveGraph, ReactiveValue, ReadContext,
    ReadScope, ReadSignal, SpringOptions, Trigger, TweenOptions, Var, WatchContext, WidgetContext,
    WriteContext,
};
pub use render::{
    Canvas, CanvasContext, CanvasWidget, RenderContext, Scene, TextLayout, invalidate_window,
    render_widgets,
};
pub use task_queue::TaskQueue;

pub use view::*;
pub use view_sequence::*;
pub use widget::{EventStatus, StatusChange, Widget, WidgetAdapter};
pub use widget_data::{WidgetData, WidgetFlags, WidgetId};
pub use widget_ref::{WidgetMut, WidgetRef};
pub use widgets::{WidgetPos, Widgets};
#[cfg(target_os = "macos")]
pub(crate) use window::MyHandler;
pub use window::Window;

use crate::{param::ParameterMap, platform};

slotmap::new_key_type! {
    pub struct WindowId;
}

pub struct WidgetHandle<W: Widget + ?Sized> {
    pub id: WidgetId,
    _phantom: PhantomData<fn() -> W>,
}

impl<W: Widget + ?Sized> WidgetHandle<W> {
    pub fn new(id: WidgetId) -> Self {
        Self {
            id,
            _phantom: PhantomData,
        }
    }

    pub fn into_any_widget_id(self) -> AnyWidgetId {
        AnyWidgetId {
            id: self.id,
            _phantom: PhantomData,
        }
    }
}

pub type AnyWidgetId = WidgetHandle<dyn Widget>;

impl<T: Widget + ?Sized> Clone for WidgetHandle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Widget + ?Sized> Copy for WidgetHandle<T> {}

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
            state,
        }
    }

    pub fn run(&mut self) {
        self.native.run()
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

mod prelude {}
