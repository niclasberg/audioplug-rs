use super::reactive::{CanRead, CanWrite, ReadScope};
use super::{
    AppState, EventResult, WidgetId, WindowId, animation::drive_animations, clipboard::Clipboard,
    invalidate_window,
};
use crate::event::{MouseClick, MouseDragEvent};
use crate::platform::OSMouseEvent;
use crate::ui::reactive::{ReactiveGraph, ReadContext, WidgetStatus, WriteContext};
use crate::ui::widgets::{Gesture, GestureState};
use crate::ui::{Capabilities, Event, TaskQueue, TextContext, Widgets};
use crate::{
    KeyEvent,
    core::{Key, Rect},
    platform::WindowEvent,
    ui::{WidgetMut, layout::RecomputeLayout, task_queue::Task},
};
use crate::{MouseButton, MouseEvent};

pub fn handle_window_event(app_state: &mut AppState, window_id: WindowId, event: WindowEvent) {
    match event {
        WindowEvent::Resize { .. } => {
            app_state.widgets.layout_window(
                &mut TextContext::new(
                    &mut app_state.font_cx,
                    &mut app_state.text_layout_cx,
                    &mut app_state.glyph_cache,
                ),
                window_id,
                RecomputeLayout::Force,
            );
            invalidate_window(&app_state.widgets, window_id);
        }
        WindowEvent::Mouse(mouse_event) => {
            handle_mouse_event(app_state, window_id, mouse_event);
        }
        WindowEvent::Key(key_event) => {
            handle_key_event(app_state, window_id, key_event);
        }
        WindowEvent::Focused => {}
        WindowEvent::Unfocused => {
            set_focus_widget(app_state, window_id, None);
        }
        WindowEvent::Animation(animation_frame) => {
            drive_animations(app_state, animation_frame);
        }
        WindowEvent::MouseCaptureEnded => {
            set_hovered_widget(app_state, None);
        }
        WindowEvent::ThemeChanged(theme) => {
            //let signal = app_state.theme_signal;
            //signal.set(&mut app_state.write_context(), theme);
        }
        WindowEvent::ScaleFactorChanged(_) => {
            let window = app_state.widgets.window_mut(window_id);
            window.wgpu_surface.is_configured = false;
        }
        WindowEvent::MouseEnterExit(enter) => {
            if !enter
                && app_state
                    .widgets
                    .hovered_widget
                    .is_some_and(|id| app_state.widgets.window_for_widget(id).id == window_id)
            {
                set_hovered_widget(app_state, None);
            }
        }
    };
}

pub fn handle_mouse_event(
    app_state: &mut AppState,
    window_id: WindowId,
    mouse_event: OSMouseEvent,
) {
    let widgets_under_mouse = app_state
        .widgets
        .get_widgets_at(window_id, mouse_event.position());

    if let OSMouseEvent::Down(_) = mouse_event {
        let new_focus_view = widgets_under_mouse
            .iter()
            .copied()
            .rev()
            .find(|id| app_state.widgets.is_focusable(*id));

        set_focus_widget(app_state, window_id, new_focus_view);
    };

    if let Some(gesture) = app_state.widgets.gesture {
        let capabilities = app_state.widgets.tree[gesture.target].capabilities;
        match mouse_event {
            OSMouseEvent::Down(ev) => {
                dispatch_mouse_event(app_state, gesture.target, MouseEvent::Down(ev));
            }
            OSMouseEvent::Up(ev) => {
                if ev.button == MouseButton::LEFT {
                    match gesture.state {
                        GestureState::Pressed if capabilities.contains(Capabilities::CLICKABLE) => {
                            dispatch_mouse_event(
                                app_state,
                                gesture.target,
                                MouseEvent::Click(MouseClick {
                                    position: ev.position,
                                    modifiers: ev.modifiers,
                                }),
                            );
                        }
                        GestureState::Dragging
                            if capabilities.contains(Capabilities::DRAGGABLE) =>
                        {
                            dispatch_mouse_event(app_state, gesture.target, MouseEvent::DragEnded);
                        }
                        _ => {}
                    }
                    app_state.widgets.gesture = None;
                }
            }
            OSMouseEvent::Move(ev) => {
                let mut new_gesture = gesture;

                if new_gesture.state == GestureState::Pressed
                    && capabilities.contains(Capabilities::DRAGGABLE)
                {
                    if (ev.position - new_gesture.start).length_sq() > 4.0 {
                        new_gesture.state = GestureState::Dragging;
                        dispatch_mouse_event(app_state, gesture.target, MouseEvent::DragStarted)
                    }
                }

                if new_gesture.state == GestureState::Dragging {
                    let delta = ev.position - new_gesture.last;
                    dispatch_mouse_event(
                        app_state,
                        gesture.target,
                        MouseEvent::DragMoved(MouseDragEvent {
                            position: ev.position,
                            modifiers: ev.modifiers,
                            delta,
                        }),
                    );
                    new_gesture.last = ev.position;
                }

                app_state.widgets.gesture = Some(new_gesture);
            }
            OSMouseEvent::Wheel(mouse_wheel_event) => dispatch_mouse_event(
                app_state,
                gesture.target,
                MouseEvent::Wheel(mouse_wheel_event),
            ),
        }
    } else {
        for id in widgets_under_mouse.iter().copied().rev() {
            let capabilities = app_state.widgets.tree[id].capabilities;
            match mouse_event {
                OSMouseEvent::Down(ev) => {
                    if capabilities.contains(Capabilities::CLICKABLE)
                        || capabilities.contains(Capabilities::DRAGGABLE)
                    {
                        dispatch_mouse_event(app_state, id, MouseEvent::Down(ev));

                        if ev.button == MouseButton::LEFT {
                            if capabilities.contains(Capabilities::CLICKABLE) {
                                app_state.widgets.gesture =
                                    Some(Gesture::new(id, ev.position, GestureState::Pressed));
                            } else {
                                // DRAGGABLE. In this case, emit a DragStart immediately
                                app_state.widgets.gesture =
                                    Some(Gesture::new(id, ev.position, GestureState::Dragging));
                                dispatch_mouse_event(app_state, id, MouseEvent::DragStarted);
                            }
                        }

                        break;
                    }
                }
                OSMouseEvent::Up(mouse_up_event) => {
                    if capabilities.contains(Capabilities::CLICKABLE)
                        || capabilities.contains(Capabilities::DRAGGABLE)
                    {
                        dispatch_mouse_event(app_state, id, MouseEvent::Up(mouse_up_event));
                        break;
                    }
                }
                OSMouseEvent::Move(ev) => {
                    dispatch_mouse_event(app_state, id, MouseEvent::Moved(ev));
                    break;
                }
                OSMouseEvent::Wheel(ev) => {
                    dispatch_mouse_event(app_state, id, MouseEvent::Wheel(ev));
                    break;
                }
            }
        }
    }

    app_state.run_effects();
    //set_mouse_gesture(app_state, new_gesture);
}

pub fn handle_key_event(app_state: &mut AppState, window_id: WindowId, key_event: KeyEvent) {
    let mut event_status = EventResult::Continue;
    let mut key_widget = app_state
        .widgets
        .focus_widget_id(window_id)
        .unwrap_or(app_state.widgets.window(window_id).root_widget);

    // We start from the current focus widget, and work our way down until either
    // the event is handled, or we have reached a parentless widget
    while !slotmap::Key::is_null(&key_widget) && event_status != EventResult::Stop {
        event_status = app_state.event_handlers.dispatch(
            &mut EventContext {
                id: key_widget,
                widgets: &mut app_state.widgets,
                reactive_graph: &mut app_state.reactive_graph,
                task_queue: &mut app_state.task_queue,
            },
            key_widget,
            &Event::Key(key_event.clone()),
        );
        key_widget = app_state.widgets.parent(key_widget);
    }
    app_state.run_effects();

    if event_status == EventResult::Continue
        && let KeyEvent::KeyDown { key, modifiers, .. } = key_event
    {
        match key {
            Key::Escape if modifiers.is_empty() => set_hovered_widget(app_state, None),
            _ => {}
        }
    }
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
    app_state
        .write_context()
        .notify_widget_status_changed(widget_id, WidgetStatus::FOCUSED.mask);
    app_state.run_effects();
}

/*pub fn clear_focus_and_mouse_capture(app_state: &mut AppState, widget_id: WidgetId) {
    if app_state.mouse_capture_widget == Some(widget_id) {
        set_mouse_capture_widget(app_state, None);
    }

    let window_id = app_state.get_window_id_for_widget(widget_id);
}*/

pub fn set_hovered_widget(app_state: &mut AppState, new_widget: Option<WidgetId>) {
    if new_widget != app_state.widgets.hovered_widget {
        let old_capture_widget =
            std::mem::replace(&mut app_state.widgets.hovered_widget, new_widget);

        if let Some(old_mouse_capture_widget) = old_capture_widget {
            dispatch_hovered_changed(app_state, old_mouse_capture_widget);
        }

        if let Some(new_capture_widget) = new_widget {
            dispatch_hovered_changed(app_state, new_capture_widget);
        }
    }
}

fn dispatch_hovered_changed(app_state: &mut AppState, widget_id: WidgetId) {
    app_state
        .write_context()
        .notify_widget_status_changed(widget_id, WidgetStatus::HOVERED.mask);
    app_state.run_effects();
}

fn dispatch_mouse_event(app_state: &mut AppState, widget_id: WidgetId, event: MouseEvent) {
    /*app_state.event_handlers.dispatch(
        &mut EventContext {
            id: widget_id,
            widgets: &mut app_state.widgets,
            reactive_graph: &mut app_state.reactive_graph,
            task_queue: &mut app_state.task_queue,
            host_handle: app_state.host_handle.as_deref(),
        },
        widget_id,
    );*/
}

pub struct EventContext<'a> {
    id: WidgetId,
    widgets: &'a mut Widgets,
    reactive_graph: &'a mut ReactiveGraph,
    task_queue: &'a mut TaskQueue,
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
        self.widgets.is_pressed(self.id)
    }

    pub fn release_gesture(&mut self) -> bool {
        if self.widgets.is_pressed(self.id) {
            // *self.new_mouse_capture_widget = None;
            true
        } else {
            false
        }
    }

    pub fn request_layout(&mut self) {
        self.widgets.request_layout(self.id);
    }

    pub fn request_animation(&mut self) {
        self.widgets.request_animation(self.id)
    }

    pub fn request_render(&mut self) {
        self.widgets.request_render(self.id);
    }

    pub fn clipboard(&self) -> Clipboard<'_> {
        let window_id = self.widgets.window_for_widget(self.id).id;
        self.widgets.clipboard(window_id)
    }

    pub fn defer_update(&mut self, f: impl FnOnce(WidgetMut) + 'static) {
        self.task_queue.push(Task::UpdateWidget {
            widget_id: self.id,
            f: Box::new(f),
        });
    }
}

impl<'s> CanRead<'s> for EventContext<'s> {
    fn read_context<'s2>(&'s2 mut self) -> ReadContext<'s2>
    where
        's: 's2,
    {
        ReadContext {
            widgets: self.widgets,
            reactive_graph: self.reactive_graph,
            scope: ReadScope::Untracked,
            current_widget: Some(self.id),
        }
    }
}

impl<'s> CanWrite<'s> for EventContext<'s> {
    fn write_context<'s2>(&'s2 mut self) -> WriteContext<'s2>
    where
        's: 's2,
    {
        WriteContext {
            widgets: self.widgets,
            reactive_graph: self.reactive_graph,
            task_queue: self.task_queue,
        }
    }
}
