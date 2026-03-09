use super::reactive::{CLICKED_STATUS, CanRead, CanWrite, FOCUS_STATUS, ReadScope};
use super::{
    AppState, EventStatus, WidgetFlags, WidgetId, WindowId, animation::drive_animations,
    clipboard::Clipboard, invalidate_window,
};
use crate::ui::reactive::{ReactiveGraph, ReadContext, WriteContext};
use crate::ui::{HostHandle, TaskQueue, Widgets};
use crate::{
    KeyEvent, MouseEvent,
    core::{Key, Rect},
    platform::WindowEvent,
    ui::{StatusChange, Widget, WidgetMut, layout::RecomputeLayout, task_queue::Task},
};

pub fn handle_window_event(app_state: &mut AppState, window_id: WindowId, event: WindowEvent) {
    match event {
        WindowEvent::Resize { .. } => {
            app_state.widgets.layout_window(
                &app_state.widget_impls,
                window_id,
                RecomputeLayout::Force,
            );
            invalidate_window(&app_state.widgets, window_id);
        }
        WindowEvent::Mouse(mouse_event) => {
            let widgets_under_mouse = app_state
                .widgets
                .get_widgets_at(window_id, mouse_event.position());

            if let MouseEvent::Down { .. } = mouse_event {
                let new_focus_view = widgets_under_mouse
                    .iter()
                    .copied()
                    .rev()
                    .find(|id| app_state.widgets.tree[*id].flag_is_set(WidgetFlags::FOCUSABLE));

                set_focus_widget(app_state, window_id, new_focus_view);
            };

            let mut new_mouse_capture_widget = app_state.widgets.mouse_capture_widget;
            if let Some(capture_widget) = app_state.widgets.mouse_capture_widget {
                dispatch_mouse_event(
                    app_state,
                    capture_widget,
                    mouse_event,
                    &mut new_mouse_capture_widget,
                );
            } else {
                for id in widgets_under_mouse.iter().copied().rev() {
                    let status = dispatch_mouse_event(
                        app_state,
                        id,
                        mouse_event,
                        &mut new_mouse_capture_widget,
                    );
                    if status == EventStatus::Handled {
                        break;
                    }
                }
            }

            app_state.run_effects();
            set_mouse_capture_widget(app_state, new_mouse_capture_widget);
        }
        WindowEvent::Key(key_event) => {
            let mut event_status = EventStatus::Ignored;
            let mut key_widget = app_state
                .widgets
                .focus_widget_id(window_id)
                .unwrap_or(app_state.widgets.window(window_id).root_widget);

            // We start from the current focus widget, and work our way down until either
            // the event is handled, or we have reached a parentless widget
            while !slotmap::Key::is_null(&key_widget) && event_status != EventStatus::Handled {
                event_status = app_state.widget_impls[key_widget].key_event(
                    key_event.clone(),
                    &mut EventContext {
                        id: key_widget,
                        widgets: &mut app_state.widgets,
                        reactive_graph: &mut app_state.reactive_graph,
                        task_queue: &mut app_state.task_queue,
                        host_handle: app_state.host_handle.as_deref(),
                    },
                );
                key_widget = app_state.widgets.parent(key_widget);
            }
            app_state.run_effects();

            if event_status == EventStatus::Ignored
                && let KeyEvent::KeyDown { key, modifiers, .. } = key_event
            {
                match key {
                    Key::Escape if modifiers.is_empty() => {
                        set_mouse_capture_widget(app_state, None)
                    }
                    _ => {}
                }
            }
        }
        WindowEvent::Unfocused => {
            set_focus_widget(app_state, window_id, None);
        }
        WindowEvent::Animation(animation_frame) => {
            drive_animations(app_state, animation_frame);
        }
        WindowEvent::MouseCaptureEnded => {
            set_mouse_capture_widget(app_state, None);
        }
        WindowEvent::ThemeChanged(theme) => {
            //let signal = app_state.theme_signal;
            //signal.set(&mut app_state.write_context(), theme);
        }
        WindowEvent::ScaleFactorChanged(_) => {
            let window = app_state.widgets.window_mut(window_id);
            window.wgpu_surface.is_configured = false;
        }
        _ => {}
    };
}

pub fn set_focus_widget(
    app_state: &mut AppState,
    window_id: WindowId,
    new_focus_widget: Option<WidgetId>,
) {
    if new_focus_widget != app_state.widgets.focus_widget_id(window_id) {
        println!(
            "Focus change {:?}, {:?}",
            app_state.widgets.focus_widget_id(window_id),
            new_focus_widget
        );

        if let Some(old_focus_widget) = app_state.focus_widget(window_id) {
            dispatch_focus_change(app_state, old_focus_widget.id(), false);
        }

        app_state.widgets.window_mut(window_id).focus_widget = new_focus_widget;

        if let Some(focus_gained_widget) = new_focus_widget {
            dispatch_focus_change(app_state, focus_gained_widget, true);
        }
    }
}

fn dispatch_focus_change(app_state: &mut AppState, widget_id: WidgetId, has_focus: bool) {
    dispatch_status_updated(
        app_state,
        widget_id,
        if has_focus {
            StatusChange::FocusGained
        } else {
            StatusChange::FocusLost
        },
    );
    app_state
        .write_context()
        .notify_widget_status_changed(widget_id, FOCUS_STATUS.mask);
    app_state.run_effects();
}

/*pub fn clear_focus_and_mouse_capture(app_state: &mut AppState, widget_id: WidgetId) {
    if app_state.mouse_capture_widget == Some(widget_id) {
        set_mouse_capture_widget(app_state, None);
    }

    let window_id = app_state.get_window_id_for_widget(widget_id);
}*/

pub fn set_mouse_capture_widget(app_state: &mut AppState, new_capture_widget: Option<WidgetId>) {
    if new_capture_widget != app_state.widgets.mouse_capture_widget {
        println!(
            "Mouse capture change {:?}, {:?}",
            app_state.widgets.mouse_capture_widget, new_capture_widget
        );
        let old_capture_widget = std::mem::replace(
            &mut app_state.widgets.mouse_capture_widget,
            new_capture_widget,
        );

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
    dispatch_status_updated(
        app_state,
        widget_id,
        if has_mouse_capture {
            StatusChange::MouseCaptured
        } else {
            StatusChange::MouseCaptureLost
        },
    );
    app_state
        .write_context()
        .notify_widget_status_changed(widget_id, CLICKED_STATUS.mask);
    app_state.run_effects();
}

fn dispatch_status_updated(app_state: &mut AppState, widget_id: WidgetId, event: StatusChange) {
    app_state.widget_impls[widget_id].status_change(
        event,
        &mut EventContext {
            id: widget_id,
            widgets: &mut app_state.widgets,
            reactive_graph: &mut app_state.reactive_graph,
            task_queue: &mut app_state.task_queue,
            host_handle: app_state.host_handle.as_deref(),
        },
    );
}

fn dispatch_mouse_event(
    app_state: &mut AppState,
    widget_id: WidgetId,
    event: MouseEvent,
    new_mouse_capture_widget: &mut Option<WidgetId>,
) -> EventStatus {
    app_state.widget_impls[widget_id].mouse_event(
        event,
        &mut MouseEventContext {
            id: widget_id,
            widgets: &mut app_state.widgets,
            reactive_graph: &mut app_state.reactive_graph,
            task_queue: &mut app_state.task_queue,
            host_handle: app_state.host_handle.as_deref(),
            new_mouse_capture_widget,
        },
    )
}

pub struct MouseEventContext<'a> {
    id: WidgetId,
    widgets: &'a mut Widgets,
    reactive_graph: &'a mut ReactiveGraph,
    task_queue: &'a mut TaskQueue,
    host_handle: Option<&'a dyn HostHandle>,
    new_mouse_capture_widget: &'a mut Option<WidgetId>,
}

impl<'a> MouseEventContext<'a> {
    pub fn has_focus(&self) -> bool {
        self.widgets.has_focus(self.id)
    }

    pub fn has_mouse_capture(&self) -> bool {
        self.widgets.has_mouse_capture(self.id)
    }

    pub fn as_callback_context(&mut self) -> CallbackContext<'_> {
        CallbackContext {
            _id: self.id,
            widgets: self.widgets,
            reactive_graph: self.reactive_graph,
            task_queue: self.task_queue,
            host_handle: self.host_handle,
        }
    }

    pub fn capture_mouse(&mut self) {
        *self.new_mouse_capture_widget = Some(self.id);
    }

    pub fn release_capture(&mut self) -> bool {
        if self.widgets.has_mouse_capture(self.id) {
            *self.new_mouse_capture_widget = None;
            true
        } else {
            false
        }
    }

    pub fn request_layout(&mut self) {
        self.widgets.request_layout(self.id);
    }

    pub fn request_render(&mut self) {
        self.widgets.invalidate_widget(self.id);
    }

    pub fn request_animation(&mut self) {
        self.widgets.request_animation(self.id)
    }

    pub fn bounds(&self) -> Rect {
        self.widgets.global_bounds(self.id)
    }

    pub fn clipboard(&self) -> Clipboard<'_> {
        let window_id = self.widgets.window_for_widget(self.id).id;
        self.widgets.clipboard(window_id)
    }

    pub fn defer_update<W: Widget>(
        &mut self,
        _widget: &W,
        f: impl FnOnce(WidgetMut<'_, W>) + 'static,
    ) {
        self.task_queue.push(Task::UpdateWidget {
            widget_id: self.id,
            f: Box::new(move |widget| f(widget.unchecked_cast())),
        });
    }
}

pub struct EventContext<'a> {
    id: WidgetId,
    widgets: &'a mut Widgets,
    reactive_graph: &'a mut ReactiveGraph,
    task_queue: &'a mut TaskQueue,
    host_handle: Option<&'a dyn HostHandle>,
}

impl<'a> EventContext<'a> {
    /*fn dispatch_status_updated(&mut self, event: StatusChange) {
        let mut widget = self.app_state.widgets.lease_widget(self.id).unwrap();
        widget.status_change(event, self);
        self.app_state.widgets.unlease_widget(widget);
    }

    fn dispatch_key_event(&mut self, event: KeyEvent) -> EventStatus {
        let mut widget = self.app_state.widgets.lease_widget(self.id).unwrap();
        let status = widget.key_event(event, self);
        self.app_state.widgets.unlease_widget(widget);
        status
    }*/

    pub fn bounds(&self) -> Rect {
        self.widgets.global_bounds(self.id)
    }

    pub fn has_focus(&self) -> bool {
        self.widgets.has_focus(self.id)
    }

    pub fn has_mouse_capture(&self) -> bool {
        self.widgets.has_mouse_capture(self.id)
    }

    pub fn as_callback_context(&mut self) -> CallbackContext<'_> {
        CallbackContext {
            _id: self.id,
            widgets: self.widgets,
            reactive_graph: self.reactive_graph,
            task_queue: self.task_queue,
            host_handle: self.host_handle,
        }
    }

    pub fn request_layout(&mut self) {
        self.widgets.request_layout(self.id);
    }

    pub fn request_animation(&mut self) {
        self.widgets.request_animation(self.id)
    }

    pub fn request_render(&mut self) {
        self.widgets.invalidate_widget(self.id);
    }

    pub fn clipboard(&self) -> Clipboard<'_> {
        let window_id = self.widgets.window_for_widget(self.id).id;
        self.widgets.clipboard(window_id)
    }

    pub fn defer_update<W: Widget>(
        &mut self,
        _widget: &W,
        f: impl FnOnce(WidgetMut<'_, W>) + 'static,
    ) {
        self.task_queue.push(Task::UpdateWidget {
            widget_id: self.id,
            f: Box::new(move |widget| f(widget.unchecked_cast())),
        });
    }
}

pub struct CallbackContext<'a> {
    _id: WidgetId,
    widgets: &'a mut Widgets,
    reactive_graph: &'a mut ReactiveGraph,
    task_queue: &'a mut TaskQueue,
    host_handle: Option<&'a dyn HostHandle>,
}

impl<'s> CanRead<'s> for CallbackContext<'s> {
    fn read_context<'s2>(&'s2 mut self) -> ReadContext<'s2>
    where
        's: 's2,
    {
        ReadContext {
            widgets: self.widgets,
            reactive_graph: self.reactive_graph,
            scope: ReadScope::Untracked,
        }
    }
}

impl<'s> CanWrite<'s> for CallbackContext<'s> {
    fn write_context<'s2>(&'s2 mut self) -> WriteContext<'s2>
    where
        's: 's2,
    {
        WriteContext {
            widgets: self.widgets,
            reactive_graph: self.reactive_graph,
            task_queue: self.task_queue,
            host_handle: self.host_handle,
        }
    }
}
