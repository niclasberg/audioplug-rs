use super::reactive::{CanRead, CanWrite, ReadScope};
use super::{
    AppState, EventResult, WidgetId, WindowId, animation::drive_animations, clipboard::Clipboard,
    invalidate_window,
};
use crate::MouseButton;
use crate::event::{MouseClick, MouseDrag};
use crate::platform::OSMouseEvent;
use crate::ui::interaction::Interactions;
use crate::ui::reactive::{ReactiveGraph, ReadContext, WriteContext};
use crate::ui::widgets::{Gesture, GestureState};
use crate::ui::{Event, Sense, TaskQueue, TextContext, Widgets};
use crate::{
    KeyEvent,
    core::{Key, Rect},
    platform::WindowEvent,
    ui::{WidgetMut, layout::RecomputeLayout, task_queue::Task},
};

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
            handle_mouse_event(
                &mut app_state.widgets,
                &mut app_state.widget_handlers.event_handlers,
                &mut app_state.reactive_graph,
                &mut app_state.task_queue,
                window_id,
                mouse_event,
            );
        }
        WindowEvent::Key(key_event) => {
            handle_key_event(app_state, window_id, key_event);
        }
        WindowEvent::Focused => {}
        WindowEvent::Unfocused => {
            app_state
                .widgets
                .set_focus_widget(&mut app_state.reactive_graph, window_id, None);
        }
        WindowEvent::Animation(animation_frame) => {
            drive_animations(app_state, animation_frame);
        }
        WindowEvent::MouseCaptureEnded => {
            app_state
                .widgets
                .set_focus_widget(&mut app_state.reactive_graph, window_id, None);
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
                app_state
                    .widgets
                    .set_hovered_widget(&mut app_state.reactive_graph, None);
            }
        }
    };
    app_state.run_effects();
}

pub fn handle_mouse_event(
    widgets: &mut Widgets,
    event_handlers: &mut Interactions,
    reactive_graph: &mut ReactiveGraph,
    task_queue: &mut TaskQueue,
    window_id: WindowId,
    mouse_event: OSMouseEvent,
) {
    let widgets_under_mouse = widgets.get_widgets_at(window_id, mouse_event.position());

    if let OSMouseEvent::Down(_) = mouse_event {
        let new_focus_view = widgets_under_mouse
            .iter()
            .copied()
            .rev()
            .find(|id| event_handlers.has_sense(*id, Sense::FOCUSABLE));

        widgets.set_focus_widget(reactive_graph, window_id, new_focus_view);
    };

    if let Some(gesture) = widgets.gesture {
        let senses = event_handlers.senses(gesture.target);
        match mouse_event {
            OSMouseEvent::Down(ev) => {
                dispatch_event(
                    event_handlers,
                    widgets,
                    reactive_graph,
                    task_queue,
                    gesture.target,
                    &Event::MouseDown(ev),
                );
            }
            OSMouseEvent::Up(ev) => {
                if ev.button == MouseButton::LEFT {
                    match gesture.state {
                        GestureState::Pressed if senses.contains(Sense::CLICKABLE) => {
                            dispatch_event(
                                event_handlers,
                                widgets,
                                reactive_graph,
                                task_queue,
                                gesture.target,
                                &Event::Click(MouseClick {
                                    position: ev.position,
                                    modifiers: ev.modifiers,
                                }),
                            );
                        }
                        GestureState::Dragging if senses.contains(Sense::DRAGGABLE) => {
                            dispatch_event(
                                event_handlers,
                                widgets,
                                reactive_graph,
                                task_queue,
                                gesture.target,
                                &Event::Drag(MouseDrag::Ended {
                                    position: ev.position,
                                    modifiers: ev.modifiers,
                                }),
                            );
                        }
                        _ => {}
                    }
                    widgets.gesture = None;
                }
            }
            OSMouseEvent::Move(ev) => {
                let mut new_gesture = gesture;

                if new_gesture.state == GestureState::Pressed && senses.contains(Sense::DRAGGABLE) {
                    if (ev.position - new_gesture.start).length_sq() > 4.0 {
                        new_gesture.state = GestureState::Dragging;
                        dispatch_event(
                            event_handlers,
                            widgets,
                            reactive_graph,
                            task_queue,
                            gesture.target,
                            &Event::Drag(MouseDrag::Ended {
                                position: ev.position,
                                modifiers: ev.modifiers,
                            }),
                        );
                    }
                }

                if new_gesture.state == GestureState::Dragging {
                    let delta = ev.position - new_gesture.last;
                    dispatch_event(
                        event_handlers,
                        widgets,
                        reactive_graph,
                        task_queue,
                        gesture.target,
                        &Event::Drag(MouseDrag::Moved {
                            position: ev.position,
                            modifiers: ev.modifiers,
                            delta,
                        }),
                    );
                    new_gesture.last = ev.position;
                }

                widgets.gesture = Some(new_gesture);
            }
            OSMouseEvent::Wheel(mouse_wheel_event) => {
                dispatch_event(
                    event_handlers,
                    widgets,
                    reactive_graph,
                    task_queue,
                    gesture.target,
                    &Event::Wheel(mouse_wheel_event),
                );
            }
        }
    } else {
        for id in widgets_under_mouse.iter().copied().rev() {
            let capabilities = event_handlers.senses(id);
            match mouse_event {
                OSMouseEvent::Down(ev) => {
                    if capabilities.contains(Sense::CLICKABLE)
                        || capabilities.contains(Sense::DRAGGABLE)
                    {
                        dispatch_event(
                            event_handlers,
                            widgets,
                            reactive_graph,
                            task_queue,
                            id,
                            &Event::MouseDown(ev),
                        );

                        if ev.button == MouseButton::LEFT {
                            if capabilities.contains(Sense::CLICKABLE) {
                                widgets.gesture =
                                    Some(Gesture::new(id, ev.position, GestureState::Pressed));
                            } else {
                                // DRAGGABLE. In this case, emit a DragStart immediately
                                widgets.gesture =
                                    Some(Gesture::new(id, ev.position, GestureState::Dragging));
                                dispatch_event(
                                    event_handlers,
                                    widgets,
                                    reactive_graph,
                                    task_queue,
                                    id,
                                    &Event::Drag(MouseDrag::Started {
                                        position: ev.position,
                                        modifiers: ev.modifiers,
                                    }),
                                );
                            }
                        }

                        break;
                    }
                }
                OSMouseEvent::Up(mouse_up_event) => {
                    if capabilities.contains(Sense::CLICKABLE)
                        || capabilities.contains(Sense::DRAGGABLE)
                    {
                        dispatch_event(
                            event_handlers,
                            widgets,
                            reactive_graph,
                            task_queue,
                            id,
                            &Event::MouseUp(mouse_up_event),
                        );
                        break;
                    }
                }
                OSMouseEvent::Move(ev) => {
                    dispatch_event(
                        event_handlers,
                        widgets,
                        reactive_graph,
                        task_queue,
                        id,
                        &Event::MouseMove(ev),
                    );
                    break;
                }
                OSMouseEvent::Wheel(ev) => {
                    dispatch_event(
                        event_handlers,
                        widgets,
                        reactive_graph,
                        task_queue,
                        id,
                        &Event::Wheel(ev),
                    );
                    break;
                }
            }
        }
    }
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
        event_status = app_state.widget_handlers.event_handlers.dispatch(
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
            Key::Escape if modifiers.is_empty() => app_state
                .widgets
                .set_hovered_widget(&mut app_state.reactive_graph, None),
            _ => {}
        }
    }
}

/*pub fn clear_focus_and_mouse_capture(app_state: &mut AppState, widget_id: WidgetId) {
    if app_state.mouse_capture_widget == Some(widget_id) {
        set_mouse_capture_widget(app_state, None);
    }

    let window_id = app_state.get_window_id_for_widget(widget_id);
}*/

fn dispatch_event(
    event_handlers: &mut Interactions,
    widgets: &mut Widgets,
    reactive_graph: &mut ReactiveGraph,
    task_queue: &mut TaskQueue,
    widget_id: WidgetId,
    event: &Event,
) -> EventResult {
    event_handlers.dispatch(
        &mut EventContext {
            id: widget_id,
            widgets,
            reactive_graph,
            task_queue,
        },
        widget_id,
        event,
    )
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
        }
    }
}
