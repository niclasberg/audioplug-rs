use std::time::Instant;

use crate::{
    AnimationFrame,
    ui::{AppState, WidgetId, WindowId, layout::request_layout, render::invalidate_widget},
};

/// Should be called when the animation timer for a window ticks.
/// Steps all animations that have been enqueued for window.
pub(super) fn drive_animations(
    app_state: &mut AppState,
    window_id: WindowId,
    animation_frame: AnimationFrame,
) {
    let widget_ids = std::mem::take(&mut app_state.window_mut(window_id).pending_widget_animations);
    for widget_id in widget_ids {
        let mut ctx = AnimationContext {
            id: widget_id,
            app_state,
        };
        ctx.run_animation(animation_frame)
    }

    let node_ids = std::mem::take(&mut app_state.window_mut(window_id).pending_node_animations);
    let now = Instant::now();
    for node_id in node_ids {
        let did_change = app_state.runtime.try_drive_animation(node_id, now);
        if did_change {
            app_state.notify(node_id);
            // Re-queue the animation for the next frame
            app_state
                .window_mut(window_id)
                .pending_node_animations
                .insert(node_id);
        }
    }

    app_state.run_effects();
}

pub fn request_animation_frame(app_state: &mut AppState, widget_id: WidgetId) {
    let window_id = app_state.get_window_id_for_widget(widget_id);
    app_state
        .window_mut(window_id)
        .pending_widget_animations
        .insert(widget_id);
}

pub struct AnimationContext<'a> {
    id: WidgetId,
    app_state: &'a mut AppState,
}

impl AnimationContext<'_> {
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
