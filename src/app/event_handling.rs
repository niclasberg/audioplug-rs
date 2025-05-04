use super::{
    animation::{drive_animations, request_animation_frame},
    clipboard::Clipboard,
    invalidate_window,
    layout::request_layout,
    layout_window,
    render::invalidate_widget,
    AppState, EventStatus, ParamContext, ReactiveContext, ReadContext, Scope, WidgetFlags,
    WidgetId, WindowId, WriteContext,
};
use crate::{
    app::StatusChange,
    core::{Cursor, Key, Rectangle},
    platform::WindowEvent,
    KeyEvent, MouseEvent,
};

pub fn handle_window_event(app_state: &mut AppState, window_id: WindowId, event: WindowEvent) {
    match event {
        WindowEvent::Resize { .. } => {
            layout_window(app_state, window_id);
            invalidate_window(app_state, window_id);
        }
        WindowEvent::Mouse(mouse_event) => {
            match mouse_event {
                MouseEvent::Down { position, .. } => {
                    println!("{}", position);
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
                }
                MouseEvent::Wheel { position, .. } => {
                    println!("{}", position);
                }
                _ => {}
            };

            let mut ctx = if let Some(capture_view) = app_state.mouse_capture_widget {
                MouseEventContext::new(capture_view, app_state, false)
            } else {
                let id = app_state.window(window_id).root_widget;
                MouseEventContext::new(id, app_state, true)
            };
            ctx.dispatch(mouse_event);
            let new_mouse_capture_widget = ctx.new_mouse_capture_widget;
            app_state.run_effects();
            set_mouse_capture_widget(app_state, new_mouse_capture_widget);
        }
        WindowEvent::Key(key_event) => {
            let mut event_status = EventStatus::Ignored;
            let mut key_widget = app_state
                .window(window_id)
                .focus_widget
                .unwrap_or(app_state.window(window_id).root_widget);

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
            signal.set(app_state.runtime_mut(), theme);
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
            let mut ctx = EventContext::new(old_focus_widget, app_state);
            ctx.dispatch_status_updated(StatusChange::FocusLost);
            app_state.run_effects();
        }

        app_state.window_mut(window_id).focus_widget = new_focus_widget;

        if let Some(focus_gained_widget) = new_focus_widget {
            let mut ctx = EventContext::new(focus_gained_widget, app_state);
            ctx.dispatch_status_updated(StatusChange::FocusGained);
            app_state.run_effects();
        }
    }
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
        if let Some(old_mouse_capture_widget) = app_state.mouse_capture_widget {
            let mut ctx = EventContext::new(old_mouse_capture_widget, app_state);
            ctx.dispatch_status_updated(StatusChange::MouseCaptureLost);
            app_state.run_effects();
        }

        app_state.mouse_capture_widget = new_capture_widget;

        if let Some(new_capture_widget) = new_capture_widget {
            let mut ctx = EventContext::new(new_capture_widget, app_state);
            ctx.dispatch_status_updated(StatusChange::MouseCaptured);
            app_state.run_effects();
        }
    }
}

pub struct MouseEventContext<'a> {
    id: WidgetId,
    app_state: &'a mut AppState,
    can_propagate: bool,
    new_mouse_capture_widget: Option<WidgetId>,
}

impl<'a> MouseEventContext<'a> {
    fn new(id: WidgetId, app_state: &'a mut AppState, can_propagate: bool) -> Self {
        let new_mouse_capture_widget = app_state.mouse_capture_widget;
        Self {
            id,
            app_state,
            can_propagate,
            new_mouse_capture_widget,
        }
    }

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

    pub fn as_callback_context(&mut self) -> CallbackContext {
        CallbackContext {
            id: self.id,
            app_state: self.app_state,
        }
    }

    pub fn app_state_mut(&mut self) -> &mut AppState {
        self.app_state
    }

    pub fn forward_to_children(&mut self, event: MouseEvent) -> EventStatus {
        if !self.can_propagate {
            return EventStatus::Ignored;
        }

        let children = self.app_state.widget_data[self.id].children.clone();
        let old_id = self.id;
        for &child in children.iter().rev() {
            if !self.app_state.widget_data[child]
                .global_bounds()
                .contains(event.position())
            {
                continue;
            }

            self.id = child;
            let mut widget = self
                .app_state
                .widgets
                .remove(child)
                .expect("Widget not found");
            let status = widget.mouse_event(event, self);
            self.app_state.widgets.insert(child, widget);
            if status == EventStatus::Handled {
                self.id = old_id;
                return EventStatus::Handled;
            }
        }
        self.id = old_id;
        EventStatus::Ignored
    }

    pub fn capture_mouse(&mut self) {
        self.new_mouse_capture_widget = Some(self.id);
    }

    pub fn release_capture(&mut self) {
        if self.new_mouse_capture_widget == Some(self.id) {
            self.new_mouse_capture_widget = None;
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

    pub fn bounds(&self) -> Rectangle {
        self.app_state.widget_data_ref(self.id).global_bounds()
    }

    pub fn clipboard(&self) -> Clipboard {
        let window_id = self.app_state.get_window_id_for_widget(self.id);
        self.app_state.clipboard(window_id)
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
        widget.status_updated(event, self);
        self.app_state.widgets.insert(self.id, widget);
    }

    fn dispatch_key_event(&mut self, event: KeyEvent) -> EventStatus {
        let mut widget = self.app_state.widgets.remove(self.id).unwrap();
        let status = widget.key_event(event, self);
        self.app_state.widgets.insert(self.id, widget);
        status
    }

    pub fn bounds(&self) -> Rectangle {
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

    pub fn as_callback_context(&mut self) -> CallbackContext {
        CallbackContext {
            id: self.id,
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

    pub fn clipboard(&self) -> Clipboard {
        let window_id = self.app_state.get_window_id_for_widget(self.id);
        self.app_state.clipboard(window_id)
    }

    pub fn set_cursor(&mut self, _cursor: Cursor) {
        //self.app_state.cursor = cursor;
    }
}

pub struct CallbackContext<'a> {
    id: WidgetId,
    app_state: &'a mut AppState,
}

impl ParamContext for CallbackContext<'_> {
    fn host_handle(&self) -> &dyn super::HostHandle {
        self.app_state.host_handle()
    }
}

impl ReadContext for CallbackContext<'_> {
    fn scope(&self) -> Scope {
        Scope::Root
    }
}

impl WriteContext for CallbackContext<'_> {}

impl ReactiveContext for CallbackContext<'_> {
    fn runtime(&self) -> &super::Runtime {
        self.app_state.runtime()
    }

    fn runtime_mut(&mut self) -> &mut super::Runtime {
        self.app_state.runtime_mut()
    }
}
