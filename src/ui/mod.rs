mod animation;
mod app_state;
mod clipboard;
mod event_handler;
mod event_handling;
mod host_handle;
mod interaction;
mod layout;
mod overlay;
mod prop;
pub mod reactive;
mod render;
pub mod style;
mod task_queue;
mod text;
mod view;
mod view_ext;
mod view_sequence;
mod widget_prop;
mod widget_ref;
mod widget_tree;
mod widgets;
mod window;

use std::{cell::RefCell, rc::Rc};

pub use animation::AnimationContext;
pub(crate) use app_state::AppState;
pub use event_handler::{Event, EventHandlerExt, EventHandlers, EventResult};
pub use event_handling::EventContext;
pub use host_handle::HostHandle;
pub use overlay::{OverlayAnchor, OverlayOptions};

pub use interaction::{
    ButtonInteraction, Interaction, KnobEvent, KnobInteraction, Sense, SliderEvent,
    SliderInteraction,
};
pub use prop::Prop;
pub use render::{
    DrawStyle, Paint, PaintRef, RenderContext, ShadowKind, ShadowOptions, invalidate_window,
};
pub use task_queue::TaskQueue;
pub use text::{TextContext, TextStyle};
pub use view::*;
pub use view_ext::{StyleExt, TextExt, ViewExt};
pub use view_sequence::*;
pub use widget_prop::{WidgetProp, WidgetProps};
pub use widget_ref::{WidgetMut, WidgetRef};
pub use widget_tree::{WidgetData, WidgetFlags, WidgetId, WidgetTree};
pub use widgets::{WidgetPos, Widgets};
#[cfg(target_os = "macos")]
pub(crate) use window::MyHandler;
pub use window::Window;

use crate::{param::ParameterMap, platform};

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

pub mod prelude {
    pub use super::reactive::prelude::*;
    pub use super::{
        BuildContext, EventResult, IndexedViewSeq, OverlayAnchor, OverlayOptions, ReactiveValueExt,
        View, ViewSequence, view_for_each,
    };
}
