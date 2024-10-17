use std::{any::Any, marker::PhantomData};

use crate::AnimationFrame;

use super::{layout::request_layout, render::invalidate_widget, AppState, NodeId, WidgetId, WindowId};

pub(super) fn drive_animations(app_state: &mut AppState, window_id: WindowId, animation_frame: AnimationFrame) {
    let requested_animations = std::mem::take(&mut app_state.window_mut(window_id).requested_animations);
    for widget_id in requested_animations {
		let mut ctx = AnimationContext {
			id: widget_id,
			app_state
		};
		ctx.run_animation(animation_frame)
    }
}

pub fn request_animation_frame(app_state: &mut AppState, widget_id: WidgetId) {
    let window_id = app_state.get_window_id_for_widget(widget_id);
    app_state.window_mut(window_id).requested_animations.insert(widget_id);
}

pub struct AnimationContext<'a> {
	id: WidgetId, 
	app_state: &'a mut AppState
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
        invalidate_widget(&self.app_state, self.id);
    }

	pub fn request_animation(&mut self) {
		request_animation_frame(&mut self.app_state, self.id)
	}

    pub fn request_layout(&mut self) {
		request_layout(&mut self.app_state, self.id);
    }
}

pub(super) struct AnimationState {
	pub(super) value: Box<dyn Any>
}

pub struct Animation<T> {
	id: NodeId,
	_phantom: PhantomData<*const T>
}

