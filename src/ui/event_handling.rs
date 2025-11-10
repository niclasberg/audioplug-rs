use super::{
    AppState, EventStatus, ParamContext, ReactiveContext, ReadContext, ReadScope, WidgetFlags,
    WidgetId, WindowId, WriteContext,
    animation::{drive_animations, request_animation_frame},
    clipboard::Clipboard,
    invalidate_window,
    layout::request_layout,
    layout_window,
    render::invalidate_widget,
};
use crate::{
    KeyEvent, MouseEvent,
    core::{Key, Rect},
    platform::WindowEvent,
    ui::{
        StatusChange, Widget, WidgetMut,
        layout::RecomputeLayout,
        reactive::{CLICKED_STATUS, FOCUS_STATUS, notify_widget_status_changed},
    },
};

pub fn handle_window_event(app_state: &mut AppState, window_id: WindowId, event: WindowEvent) {
    match event {
        WindowEvent::Resize { physical_size, .. } => {
            app_state
                .window_mut(window_id)
                .wgpu_surface
                .resize(physical_size);
            layout_window(app_state, window_id, RecomputeLayout::Force);
            invalidate_window(app_state, window_id);
        }
        WindowEvent::Mouse(mouse_event) => {
            if let MouseEvent::Down { position, .. } = mouse_event {
                let mut new_focus_view = None;
                app_state.for_each_widget_at_rev(window_id, position, |app_state, id| {
                    if app_state.widget_data[id].flag_is_set(WidgetFlags::FOCUSABLE) {
                        new_focus_view = Some(id);
                        false
                    } else {
                        true
                    }
                });
                set_focus_widget(app_state, window_id, new_focus_view);
            };

            let mut new_mouse_capture_widget = app_state.mouse_capture_widget;
            if let Some(capture_view) = app_state.mouse_capture_widget {
                let mut cx = MouseEventContext {
                    id: capture_view,
                    app_state,
                    new_mouse_capture_widget: &mut new_mouse_capture_widget,
                };
                cx.dispatch(mouse_event);
            } else {
                app_state.for_each_widget_at_rev_mut(
                    window_id,
                    mouse_event.position(),
                    |app_state, id| {
                        let mut cx = MouseEventContext {
                            id,
                            app_state,
                            new_mouse_capture_widget: &mut new_mouse_capture_widget,
                        };
                        cx.dispatch(mouse_event) != EventStatus::Handled
                    },
                );
            }

            app_state.run_effects();
            set_mouse_capture_widget(app_state, new_mouse_capture_widget);
        }
        WindowEvent::Key(key_event) => {
            let mut event_status = EventStatus::Ignored;
            let mut key_widget = app_state
                .window(window_id)
                .focus_widget
                .unwrap_or(app_state.window(window_id).root_widget);

            // We start from the current focus widget, and work our way down until either
            // the event is handled, or we have reached a parentless widget
            while !slotmap::Key::is_null(&key_widget) && event_status != EventStatus::Handled {
                let mut ctx = EventContext::new(key_widget, app_state);
                event_status = ctx.dispatch_key_event(key_event.clone());
                key_widget = app_state.widget_data_ref(key_widget).parent_id;
            }
            app_state.run_effects();

            if event_status == EventStatus::Ignored {
                if let KeyEvent::KeyDown { key, modifiers, .. } = key_event {
                    match key {
                        Key::Escape if modifiers.is_empty() => {
                            set_mouse_capture_widget(app_state, None)
                        }
                        _ => {}
                    }
                }
            }
        }
        WindowEvent::Unfocused => {
            set_focus_widget(app_state, window_id, None);
        }
        WindowEvent::Animation(animation_frame) => {
            drive_animations(app_state, window_id, animation_frame);
        }
        WindowEvent::MouseCaptureEnded => {
            set_mouse_capture_widget(app_state, None);
        }
        WindowEvent::ThemeChanged(theme) => {
            let signal = app_state.window(window_id).theme_signal;
            signal.set(app_state, theme);
        }
        WindowEvent::ScaleFactorChanged(_) => {
            let window = app_state.window_mut(window_id);
            let physical_size = window.handle.physical_size();
            window.wgpu_surface.resize(physical_size);
        }
        _ => {}
    };
}

pub fn set_focus_widget(
    app_state: &mut AppState,
    window_id: WindowId,
    new_focus_widget: Option<WidgetId>,
) {
    if new_focus_widget != app_state.window(window_id).focus_widget {
        println!(
            "Focus change {:?}, {:?}",
            app_state.window(window_id).focus_widget,
            new_focus_widget
        );

        if let Some(old_focus_widget) = app_state.window(window_id).focus_widget {
            dispatch_focus_change(app_state, old_focus_widget, false);
        }

        app_state.window_mut(window_id).focus_widget = new_focus_widget;

        if let Some(focus_gained_widget) = new_focus_widget {
            dispatch_focus_change(app_state, focus_gained_widget, true);
        }
    }
}

fn dispatch_focus_change(app_state: &mut AppState, widget_id: WidgetId, has_focus: bool) {
    let mut ctx = EventContext::new(widget_id, app_state);
    ctx.dispatch_status_updated(if has_focus {
        StatusChange::FocusGained
    } else {
        StatusChange::FocusLost
    });
    notify_widget_status_changed(app_state, widget_id, FOCUS_STATUS.mask);
    app_state.run_effects();
}

/*pub fn clear_focus_and_mouse_capture(app_state: &mut AppState, widget_id: WidgetId) {
    if app_state.mouse_capture_widget == Some(widget_id) {
        set_mouse_capture_widget(app_state, None);
    }

    let window_id = app_state.get_window_id_for_widget(widget_id);
}*/

pub fn set_mouse_capture_widget(app_state: &mut AppState, new_capture_widget: Option<WidgetId>) {
    if new_capture_widget != app_state.mouse_capture_widget {
        println!(
            "Mouse capture change {:?}, {:?}",
            app_state.mouse_capture_widget, new_capture_widget
        );
        let old_capture_widget =
            std::mem::replace(&mut app_state.mouse_capture_widget, new_capture_widget);

        if let Some(old_mouse_capture_widget) = old_capture_widget {
            dispatch_mouse_capture_change(app_state, old_mouse_capture_widget, false);
        }

        if let Some(new_capture_widget) = new_capture_widget {
            dispatch_mouse_capture_change(app_state, new_capture_widget, true);
        }
    }
}

fn dispatch_mouse_capture_change(
    app_state: &mut AppState,
    widget_id: WidgetId,
    has_mouse_capture: bool,
) {
    let mut ctx = EventContext::new(widget_id, app_state);
    ctx.dispatch_status_updated(if has_mouse_capture {
        StatusChange::MouseCaptured
    } else {
        StatusChange::MouseCaptureLost
    });
    notify_widget_status_changed(app_state, widget_id, CLICKED_STATUS.mask);
    app_state.run_effects();
}

pub struct MouseEventContext<'a> {
    id: WidgetId,
    app_state: &'a mut AppState,
    new_mouse_capture_widget: &'a mut Option<WidgetId>,
}

impl<'a> MouseEventContext<'a> {
    fn dispatch(&mut self, event: MouseEvent) -> EventStatus {
        let mut widget = self
            .app_state
            .widgets
            .remove(self.id)
            .expect("Widget not found");
        let old_id = self.id;
        let status = widget.mouse_event(event, self);
        self.id = old_id;
        self.app_state.widgets.insert(old_id, widget);
        status
    }

    pub fn has_focus(&self) -> bool {
        self.app_state.widget_has_focus(self.id)
    }

    pub fn has_mouse_capture(&self) -> bool {
        self.app_state.widget_has_captured_mouse(self.id)
    }

    pub fn app_state(&self) -> &AppState {
        self.app_state
    }

    pub fn as_callback_context(&mut self) -> CallbackContext<'_> {
        CallbackContext {
            _id: self.id,
            app_state: self.app_state,
        }
    }

    pub fn app_state_mut(&mut self) -> &mut AppState {
        self.app_state
    }

    pub fn capture_mouse(&mut self) {
        *self.new_mouse_capture_widget = Some(self.id);
    }

    pub fn release_capture(&mut self) -> bool {
        if self.app_state.mouse_capture_widget == Some(self.id) {
            *self.new_mouse_capture_widget = None;
            true
        } else {
            false
        }
    }

    pub fn request_layout(&mut self) {
        request_layout(self.app_state, self.id);
    }

    pub fn request_render(&mut self) {
        invalidate_widget(self.app_state, self.id);
    }

    pub fn request_animation(&mut self) {
        request_animation_frame(self.app_state, self.id)
    }

    pub fn bounds(&self) -> Rect {
        self.app_state.widget_data_ref(self.id).global_bounds()
    }

    pub fn clipboard(&self) -> Clipboard<'_> {
        let window_id = self.app_state.get_window_id_for_widget(self.id);
        self.app_state.clipboard(window_id)
    }

    pub fn defer_update<W: Widget>(
        &mut self,
        _widget: &W,
        f: impl FnOnce(WidgetMut<'_, W>) + 'static,
    ) {
        self.app_state
            .push_task(super::app_state::Task::UpdateWidget {
                widget_id: self.id,
                f: Box::new(move |widget| f(widget.unchecked_cast())),
            });
    }
}

pub struct EventContext<'a> {
    id: WidgetId,
    app_state: &'a mut AppState,
}

impl<'a> EventContext<'a> {
    fn new(id: WidgetId, app_state: &'a mut AppState) -> Self {
        Self { id, app_state }
    }

    fn dispatch_status_updated(&mut self, event: StatusChange) {
        let mut widget = self.app_state.widgets.remove(self.id).unwrap();
        widget.status_change(event, self);
        self.app_state.widgets.insert(self.id, widget);
    }

    fn dispatch_key_event(&mut self, event: KeyEvent) -> EventStatus {
        let mut widget = self.app_state.widgets.remove(self.id).unwrap();
        let status = widget.key_event(event, self);
        self.app_state.widgets.insert(self.id, widget);
        status
    }

    pub fn bounds(&self) -> Rect {
        self.app_state.widget_data_ref(self.id).global_bounds()
    }

    pub fn has_focus(&self) -> bool {
        self.app_state.widget_has_focus(self.id)
    }

    pub fn has_mouse_capture(&self) -> bool {
        self.app_state.widget_has_captured_mouse(self.id)
    }

    pub fn app_state(&self) -> &AppState {
        self.app_state
    }

    pub fn app_state_mut(&mut self) -> &mut AppState {
        self.app_state
    }

    pub fn as_callback_context(&mut self) -> CallbackContext<'_> {
        CallbackContext {
            _id: self.id,
            app_state: self.app_state,
        }
    }

    pub fn request_layout(&mut self) {
        request_layout(self.app_state, self.id);
    }

    pub fn request_animation(&mut self) {
        request_animation_frame(self.app_state, self.id)
    }

    pub fn request_render(&mut self) {
        invalidate_widget(self.app_state, self.id);
    }

    pub fn clipboard(&self) -> Clipboard<'_> {
        let window_id = self.app_state.get_window_id_for_widget(self.id);
        self.app_state.clipboard(window_id)
    }

    pub fn defer_update<W: Widget>(
        &mut self,
        _widget: &W,
        f: impl FnOnce(WidgetMut<'_, W>) + 'static,
    ) {
        self.app_state
            .push_task(super::app_state::Task::UpdateWidget {
                widget_id: self.id,
                f: Box::new(move |widget| f(widget.unchecked_cast())),
            });
    }
}

pub struct CallbackContext<'a> {
    _id: WidgetId,
    app_state: &'a mut AppState,
}

impl ParamContext for CallbackContext<'_> {
    fn host_handle(&self) -> &dyn super::HostHandle {
        self.app_state.host_handle()
    }
}

impl ReadContext for CallbackContext<'_> {
    fn scope(&self) -> ReadScope {
        ReadScope::Untracked
    }
}

impl WriteContext for CallbackContext<'_> {}

impl ReactiveContext for CallbackContext<'_> {
    fn app_state(&self) -> &AppState {
        self.app_state
    }

    fn app_state_mut(&mut self) -> &mut AppState {
        self.app_state
    }
}
