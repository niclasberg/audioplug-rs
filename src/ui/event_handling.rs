use super::{
    AppState, EventResult, WidgetId, WindowId, animation::drive_animations, invalidate_window,
};
use crate::MouseButton;
use crate::core::Modifiers;
use crate::event::{MouseClick, MouseDrag};
use crate::platform::OSMouseEvent;
use crate::ui::ui_state::{Gesture, GestureState, Invalidation, WidgetHandlers};
use crate::ui::{Event, Sense, UiState};
use crate::{KeyEvent, core::Key, platform::WindowEvent, ui::layout::RecomputeLayout};

pub fn handle_window_event(app_state: &mut AppState, window_id: WindowId, event: WindowEvent) {
    match event {
        WindowEvent::Resize { .. } => {
            app_state
                .ui
                .layout_window(window_id, RecomputeLayout::Force);
            invalidate_window(&app_state.ui, window_id);
        }
        WindowEvent::Mouse(mouse_event) => {
            handle_mouse_event(
                &mut app_state.ui,
                &mut app_state.widget_handlers,
                window_id,
                mouse_event,
            );
        }
        WindowEvent::Key(key_event) => {
            handle_key_event(
                &mut app_state.ui,
                &mut app_state.widget_handlers,
                window_id,
                key_event,
            );
        }
        WindowEvent::Focused => {}
        WindowEvent::Unfocused => {
            app_state.ui.set_focus_widget(window_id, None);
        }
        WindowEvent::Animation(animation_frame) => {
            drive_animations(app_state, animation_frame);
        }
        WindowEvent::MouseCaptureEnded => {
            // Drop any active gesture
        }
        WindowEvent::ThemeChanged(theme) => {
            //let signal = app_state.theme_signal;
            //signal.set(&mut app_state.write_context(), theme);
        }
        WindowEvent::ScaleFactorChanged(_) => {
            let window = app_state.ui.window_mut(window_id);
            window.wgpu_surface.is_configured = false;
        }
        WindowEvent::MouseEnterExit(enter) => {
            if !enter
                && app_state
                    .ui
                    .hovered_widget
                    .is_some_and(|id| app_state.ui.window_for_widget(id).id == window_id)
            {
                app_state.ui.set_hovered_widget(None);
            }
        }
    };
    app_state.run_effects_and_layout();
}

pub fn handle_mouse_event(
    ui: &mut UiState,
    handlers: &mut WidgetHandlers,
    window_id: WindowId,
    mouse_event: OSMouseEvent,
) {
    let widgets_under_mouse = ui.get_widgets_at(window_id, mouse_event.position());

    if let OSMouseEvent::Down(_) = mouse_event {
        let new_focus_view = widgets_under_mouse
            .iter()
            .copied()
            .rev()
            .find(|id| ui.is_focusable(&handlers.event_handlers, *id));

        ui.set_focus_widget(window_id, new_focus_view);
    };

    if let OSMouseEvent::Move(_) = mouse_event {
        let new_hovered_widget = widgets_under_mouse
            .iter()
            .copied()
            .rev()
            .find(|id| ui.is_hoverable(*id));

        ui.set_hovered_widget(new_hovered_widget);
    }

    if let Some(gesture) = ui.gesture {
        let senses = handlers.event_handlers.senses(gesture.target);
        match mouse_event {
            OSMouseEvent::Down(ev) => {
                ui.dispatch(handlers, gesture.target, Event::MouseDown(ev));
            }
            OSMouseEvent::Up(ev) => {
                if ev.button == MouseButton::LEFT {
                    match gesture.state {
                        GestureState::Pressed if senses.contains(Sense::CLICKABLE) => {
                            ui.dispatch(
                                handlers,
                                gesture.target,
                                Event::Click(MouseClick {
                                    position: ev.position,
                                    modifiers: ev.modifiers,
                                }),
                            );
                        }
                        GestureState::Dragging if senses.contains(Sense::DRAGGABLE) => {
                            ui.dispatch(
                                handlers,
                                gesture.target,
                                Event::Drag(MouseDrag::Ended {
                                    position: ev.position,
                                    modifiers: ev.modifiers,
                                }),
                            );
                        }
                        _ => {}
                    }
                    ui.gesture = None;
                }
            }
            OSMouseEvent::Move(ev) => {
                let mut new_gesture = gesture;

                if new_gesture.state == GestureState::Pressed && senses.contains(Sense::DRAGGABLE) {
                    if (ev.position - new_gesture.start).length_sq() > 4.0 {
                        new_gesture.state = GestureState::Dragging;
                        ui.dispatch(
                            handlers,
                            gesture.target,
                            Event::Drag(MouseDrag::Ended {
                                position: ev.position,
                                modifiers: ev.modifiers,
                            }),
                        );
                    }
                }

                if new_gesture.state == GestureState::Dragging {
                    let delta = ev.position - new_gesture.last;
                    ui.dispatch(
                        handlers,
                        gesture.target,
                        Event::Drag(MouseDrag::Moved {
                            position: ev.position,
                            modifiers: ev.modifiers,
                            delta,
                        }),
                    );
                    new_gesture.last = ev.position;
                }

                ui.gesture = Some(new_gesture);
            }
            OSMouseEvent::Wheel(mouse_wheel_event) => {
                ui.dispatch(handlers, gesture.target, Event::Wheel(mouse_wheel_event));
            }
        }
    } else {
        for id in widgets_under_mouse.iter().copied().rev() {
            let capabilities = handlers.event_handlers.senses(id);
            match mouse_event {
                OSMouseEvent::Down(ev) => {
                    if capabilities.contains(Sense::CLICKABLE)
                        || capabilities.contains(Sense::DRAGGABLE)
                    {
                        ui.dispatch(handlers, id, Event::MouseDown(ev));

                        if ev.button == MouseButton::LEFT {
                            if capabilities.contains(Sense::CLICKABLE) {
                                ui.gesture =
                                    Some(Gesture::new(id, ev.position, GestureState::Pressed));
                            } else {
                                // DRAGGABLE. In this case, emit a DragStart immediately
                                ui.gesture =
                                    Some(Gesture::new(id, ev.position, GestureState::Dragging));
                                ui.dispatch(
                                    handlers,
                                    id,
                                    Event::Drag(MouseDrag::Started {
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
                        ui.dispatch(handlers, id, Event::MouseUp(mouse_up_event));
                        break;
                    }
                }
                OSMouseEvent::Move(ev) => {
                    ui.dispatch(handlers, id, Event::MouseMove(ev));
                    break;
                }
                OSMouseEvent::Wheel(ev) => {
                    ui.dispatch(handlers, id, Event::Wheel(ev));
                    break;
                }
            }
        }
    }
}

pub fn handle_key_event(
    ui: &mut UiState,
    handlers: &mut WidgetHandlers,
    window_id: WindowId,
    key_event: KeyEvent,
) {
    let mut event_status = EventResult::Continue;
    let mut key_widget = ui
        .focus_widget_id(window_id)
        .unwrap_or(ui.window(window_id).root_widget);

    // We start from the current focus widget, and work our way down until either
    // the event is handled, or we have reached a parentless widget
    while !slotmap::Key::is_null(&key_widget) && event_status != EventResult::Stop {
        event_status = ui
            .dispatch(handlers, key_widget, Event::Key(key_event.clone()))
            .or_else(|| handle_text_edit_event(ui, handlers, window_id, key_widget, &key_event));

        key_widget = ui.parent(key_widget);
    }

    if event_status == EventResult::Continue
        && let KeyEvent::KeyDown { key, modifiers, .. } = key_event
    {
        match key {
            Key::Escape if modifiers.is_empty() => ui.set_gesture(None),
            Key::Tab if modifiers.is_empty() => {
                // FOCUSABLE should probably move out of the Interaction and into the widget data,
                // so that we don't have to pass the handler here
                let new_focus_widget =
                    ui.next_focusable_widget(&handlers.event_handlers, window_id);
                ui.set_focus_widget(window_id, new_focus_widget);
            }
            Key::Tab if modifiers == Modifiers::SHIFT => {
                let new_focus_widget =
                    ui.prev_focusable_widget(&handlers.event_handlers, window_id);
                ui.set_focus_widget(window_id, new_focus_widget);
            }
            _ => {}
        }
    }
}

fn handle_text_edit_event(
    ui: &mut UiState,
    handlers: &mut WidgetHandlers,
    window_id: WindowId,
    widget_id: WidgetId,
    key_event: &KeyEvent,
) -> EventResult {
    #[cfg(target_os = "macos")]
    const WORD_MODIFIER: Modifiers = Modifiers::OPT;
    #[cfg(target_os = "windows")]
    const WORD_MODIFIER: Modifiers = Modifiers::CONTROL;

    if let Some(text) = ui.texts.get_mut(widget_id)
        && text.is_editable()
        && let KeyEvent::KeyDown {
            key,
            modifiers,
            str,
            ..
        } = key_event
    {
        let mut invalidation = Invalidation::empty();
        let result = match (key, str) {
            (Key::BackSpace, _) if *modifiers == WORD_MODIFIER => {
                if text.remove_word_left() {
                    invalidation = Invalidation::LAYOUT;
                }
                EventResult::Stop
            }
            (Key::BackSpace, _) if modifiers.is_empty() => {
                if text.remove_left() {
                    invalidation = Invalidation::LAYOUT;
                }
                EventResult::Stop
            }
            (Key::Delete, _) if *modifiers == WORD_MODIFIER => {
                if text.remove_word_right() {
                    invalidation = Invalidation::LAYOUT;
                }
                EventResult::Stop
            }
            (Key::Delete, _) if modifiers.is_empty() => {
                if text.remove_right() {
                    invalidation = Invalidation::LAYOUT;
                }
                EventResult::Stop
            }
            (Key::Left, _) if modifiers.contains(WORD_MODIFIER) => {
                if text.move_word_left(modifiers.contains(Modifiers::SHIFT)) {
                    invalidation = Invalidation::RENDER;
                }
                EventResult::Stop
            }
            (Key::Left, _) => {
                if text.move_left(modifiers.contains(Modifiers::SHIFT)) {
                    invalidation = Invalidation::RENDER;
                }
                EventResult::Stop
            }
            (Key::Right, _) if modifiers.contains(WORD_MODIFIER) => {
                if text.move_word_right(modifiers.contains(Modifiers::SHIFT)) {
                    invalidation = Invalidation::RENDER;
                }
                EventResult::Stop
            }
            (Key::Right, _) => {
                if text.move_right(modifiers.contains(Modifiers::SHIFT)) {
                    invalidation = Invalidation::RENDER;
                }
                EventResult::Stop
            }
            (Key::C, _) if *modifiers == Modifiers::CONTROL => {
                if let Some(selected_text) = text.selected_text() {
                    // Need to clone here because of borrowing issues, can we resolve? Maybe put the clipboard with the handlers?
                    let selected_text = selected_text.to_string();
                    ui.clipboard(window_id).set_text(&selected_text);
                }
                EventResult::Stop
            }
            (Key::V, _) if *modifiers == Modifiers::CONTROL => {
                if let Ok(Some(text_to_insert)) = ui.windows[window_id].handle.get_clipboard() {
                    text.insert_or_replace(text_to_insert.as_str());
                    invalidation = Invalidation::LAYOUT;
                }
                EventResult::Stop
            }
            (Key::X, _) if *modifiers == Modifiers::CONTROL => EventResult::Stop,
            (Key::Tab, _) | (Key::Escape, _) | (Key::Enter, _) | (Key::Up, _) | (Key::Down, _) => {
                EventResult::Continue
            }
            (_, Some(str)) if !modifiers.intersects(Modifiers::COMMAND) => {
                text.insert_or_replace(str.as_str());
                invalidation = Invalidation::LAYOUT;
                EventResult::Stop
            }
            _ => EventResult::Continue,
        };
        ui.invalidate_widget(widget_id, invalidation);
        result
    } else {
        EventResult::Continue
    }
}

/*pub fn clear_focus_and_mouse_capture(app_state: &mut AppState, widget_id: WidgetId) {
    if app_state.mouse_capture_widget == Some(widget_id) {
        set_mouse_capture_widget(app_state, None);
    }

    let window_id = app_state.get_window_id_for_widget(widget_id);
}*/
