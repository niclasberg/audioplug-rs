use std::{any::Any, marker::PhantomData};

use crate::AnimationFrame;

use super::{
    accessor::SourceId, layout::request_layout, render::invalidate_widget, AppState, NodeId,
    NodeType, ReadContext, Readable, ViewContext, WidgetId, WindowId,
};

pub(super) fn drive_animations(
    app_state: &mut AppState,
    window_id: WindowId,
    animation_frame: AnimationFrame,
) {
    let requested_animations =
        std::mem::take(&mut app_state.window_mut(window_id).requested_animations);
    for widget_id in requested_animations {
        let mut ctx = AnimationContext {
            id: widget_id,
            app_state,
        };
        ctx.run_animation(animation_frame)
    }
}

pub fn request_animation_frame(app_state: &mut AppState, widget_id: WidgetId) {
    let window_id = app_state.get_window_id_for_widget(widget_id);
    app_state
        .window_mut(window_id)
        .requested_animations
        .insert(widget_id);
}

pub struct AnimationContext<'a> {
    id: WidgetId,
    app_state: &'a mut AppState,
}

impl<'a> AnimationContext<'a> {
    fn run_animation(&mut self, animation_frame: AnimationFrame) {
        if let Some(mut widget) = self.app_state.widgets.remove(self.id) {
            widget.animation_frame(animation_frame, self);
            self.app_state.widgets.insert(self.id, widget);
        }
    }

    pub fn has_focus(&self) -> bool {
        self.app_state.widget_has_focus(self.id)
    }

    pub fn request_render(&mut self) {
        invalidate_widget(self.app_state, self.id);
    }

    pub fn request_animation(&mut self) {
        request_animation_frame(self.app_state, self.id)
    }

    pub fn request_layout(&mut self) {
        request_layout(self.app_state, self.id);
    }
}

pub(crate) struct AnimationState {
    pub(super) value: Box<dyn Any>,
    /// Function that drives the animation, the first argument is the current value
    /// and the second is the delta time since the last frame
    /// Should return false when the animation is finished, and true otherwise
    pub(super) drive_fn: Box<dyn Fn(&mut dyn Any, f64) -> bool>,
    pub(super) window_id: WindowId,
}

pub struct Animated<T> {
    id: NodeId,
    _phantom: PhantomData<*const T>,
}

impl<T> Animated<T> {
    pub fn new<S: Readable<Value = T>>(cx: &mut dyn ViewContext, signal: &S) -> Self {
        todo!()
    }
}

impl<T: 'static> Readable for Animated<T> {
    type Value = T;

    fn get_source_id(&self) -> SourceId {
        SourceId::Node(self.id)
    }

    fn with_ref<R>(&self, cx: &mut dyn ReadContext, f: impl FnOnce(&T) -> R) -> R {
        let scope = cx.scope();
        cx.runtime_mut().track(self.id, scope);
        let value = match &cx.runtime().get_node(self.id).node_type {
            NodeType::Animation(animation) => animation.value.as_ref(),
            _ => unreachable!(),
        };
        f(value
            .downcast_ref()
            .expect("Animation value had wrong type"))
    }
}
