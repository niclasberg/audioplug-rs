use crate::{
    AnimationFrame,
    ui::{AppState, WidgetId, layout::request_layout, render::invalidate_widget},
};

/// Should be called when the animation timer for a window ticks.
/// Steps all animations that have been enqueued for window.
pub(super) fn drive_animations(app_state: &mut AppState, animation_frame: AnimationFrame) {
    let widget_ids = std::mem::take(&mut app_state.widgets.pending_animations);
    for widget_id in widget_ids {
        let mut ctx = AnimationContext {
            id: widget_id,
            app_state,
        };
        ctx.run_animation(animation_frame)
    }

    app_state
        .reactive_graph
        .drive_animations(&mut app_state.widgets, &mut app_state.task_queue);

    app_state.run_effects();
}

pub struct AnimationContext<'a> {
    id: WidgetId,
    app_state: &'a mut AppState,
}

impl AnimationContext<'_> {
    fn run_animation(&mut self, animation_frame: AnimationFrame) {
        if let Some(mut widget) = self.app_state.widgets.widgets.remove(self.id) {
            widget.animation_frame(animation_frame, self);
            self.app_state.widgets.widgets.insert(self.id, widget);
        }
    }

    pub fn has_focus(&self) -> bool {
        self.app_state.widget_has_focus(self.id)
    }

    pub fn request_render(&mut self) {
        invalidate_widget(self.app_state, self.id);
    }

    pub fn request_animation(&mut self) {
        self.app_state.widgets.request_animation(self.id);
    }

    pub fn request_layout(&mut self) {
        request_layout(&mut self.app_state.widgets, self.id);
    }
}
