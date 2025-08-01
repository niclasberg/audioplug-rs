mod animation;
mod app_state;
mod clipboard;
mod event_handling;
mod host_handle;
mod layout;
mod overlay;
mod param;
pub(super) mod reactive;
mod render;
pub mod style;
mod view;
mod view_sequence;
mod widget;
mod widget_data;
mod widget_ref;
mod window;

use indexmap::{IndexMap, IndexSet};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    marker::PhantomData,
    rc::Rc,
};

pub use animation::AnimationContext;
pub(crate) use app_state::AppState;
pub use event_handling::{CallbackContext, EventContext, MouseEventContext, handle_window_event};
pub use host_handle::HostHandle;
pub use layout::{LayoutContext, layout_window};
pub use overlay::{OverlayAnchor, OverlayOptions};
pub use param::{ParamContext, ParamSetter};

pub use reactive::{
    Accessor, Animated, AnimatedFn, Animation, Cached, CachedContext, Computed, CreateContext,
    Easing, Effect, EffectContext, EventChannel, EventReceiver, Mapped, Node, NodeId, Owner,
    ReactiveContext, ReactiveGraph, ReactiveValue, ReadContext, ReadSignal, Scope, SpringOptions,
    Trigger, TweenOptions, Var, ViewContext, WatchContext, WidgetContext, WriteContext,
};
pub use render::{
    Brush, BrushRef, Canvas, CanvasContext, CanvasWidget, LinearGradient, PathGeometry,
    PathGeometryBuilder, RadialGradient, RenderContext, Shape, ShapeRef, TextLayout,
    invalidate_window, render_window,
};

pub use view::*;
pub use view_sequence::*;
pub use widget::{EventStatus, StatusChange, Widget, WrappedWidget};
pub use widget_data::{WidgetData, WidgetFlags, WidgetId};
pub use widget_ref::{WidgetMut, WidgetRef};
#[cfg(target_os = "macos")]
pub(crate) use window::MyHandler;
pub use window::Window;

use crate::{param::ParameterMap, platform};

slotmap::new_key_type! {
    pub struct WindowId;
}

pub struct TypedWidgetId<W: Widget + ?Sized> {
    pub id: WidgetId,
    _phantom: PhantomData<fn() -> W>,
}

impl<W: Widget + ?Sized> TypedWidgetId<W> {
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

pub type AnyWidgetId = TypedWidgetId<dyn Widget>;

impl<T: Widget + ?Sized> Clone for TypedWidgetId<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Widget + ?Sized> Copy for TypedWidgetId<T> {}

pub struct AppContext {
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl AppContext {
    pub fn new() -> Self {
        todo!()
    }
}

pub struct App {
    native: platform::Application,
    pub(crate) state: Rc<RefCell<AppState>>,
    pub(crate) context: AppContext,
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
            context: AppContext::new(),
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
