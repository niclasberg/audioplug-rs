use crate::{core::{Cursor, Rectangle}, keyboard::Key, platform::{self, WindowEvent}, view::EventStatus, KeyEvent, MouseEvent};

use super::{invalidate_window, layout_window, widget_node::WidgetFlags, AppState, WidgetId, WindowId};

pub fn handle_window_event(app_state: &mut AppState, window_id: WindowId, event: WindowEvent) {
    match event {
        WindowEvent::Resize { .. } => {
            layout_window(app_state, window_id);
            invalidate_window(app_state, window_id);
        }
        WindowEvent::Mouse(mouse_event) => {
            match mouse_event {
                MouseEvent::Down { position, .. } => {
                    let new_focus_view = app_state.find_map_widget_at(window_id, position, |data| {
                        data.flag_is_set(WidgetFlags::FOCUSABLE).then(|| data.id)
                    });
                    set_focus_widget(app_state, new_focus_view);
                }
                _ => {}
            };

            if let Some(capture_view) = app_state.mouse_capture_widget {

                let message = ViewMessage::Mouse(mouse_event);
                self.widget_node.handle_message(
                    &mut capture_view,
                    message,
                    &mut self.state,
                    &mut handle,
                    &mut app_state,
                );

                match mouse_event {
                    MouseEvent::Up { .. } => self.state.mouse_capture_view = None,
                    _ => {}
                }
            } else {
                let mut ctx = EventContext::new(
                    &mut self.widget_node.data,
                    &mut self.state,
                    &mut handle,
                    &mut app_state,
                );
                self.widget_node.widget.mouse_event(mouse_event, &mut ctx);
            }
        }
        WindowEvent::Key(key_event) => {
            let mut event_status = EventStatus::Ignored;
            if let Some(focus_widget) = app_state.focus_widget {
                let mut ctx = EventContext::new(focus_widget, app_state);
                event_status = app_state.widgets[focus_widget].key_event(key_event, &mut ctx);
            }

            if event_status == EventStatus::Ignored {
                match key_event {
                    KeyEvent::KeyDown { key, modifiers, .. } => match key {
                        Key::Escape if modifiers.is_empty() => set_focus_widget(app_state, None),
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
        WindowEvent::Unfocused => {
            set_focus_widget(app_state, None);
        }
        _ => {}
    };

    /*if app_state.widget_data_ref(self.state.root_widget()).layout_requested() {
        self.do_layout(&mut handle);
    }*/
}

pub fn set_focus_widget(app_state: &mut AppState, new_focus_widget: Option<WidgetId>) {
    if new_focus_widget != app_state.focus_widget {
        println!("Focus change {:?}, {:?}", app_state.focus_widget, new_focus_widget);

        if let Some(focus_lost_widget) = app_state.focus_widget {
            let mut ctx = EventContext::new(focus_lost_widget, app_state);
            app_state.widgets[focus_lost_widget].focus_changed(false, &mut ctx);
        }

        app_state.focus_widget = new_focus_widget.clone();

        if let Some(focus_gained_widget) = new_focus_widget {
            let mut ctx = EventContext::new(focus_gained_widget, app_state);
            app_state.widgets[focus_gained_widget].focus_changed(true, &mut ctx);
        }
    }
}


/*fn find_focus_view_at(position: Point, widget_node: &WidgetNode) -> Option<IdPath> {
    if !widget_node.data().global_bounds().contains(position) {
        return None;
    }

    let child_focus_view = (0..widget_node.widget.child_count())
        .rev()
        .find_map(|i| find_focus_view_at(position, widget_node.widget.get_child(i)));

    if child_focus_view.is_some() {
        child_focus_view
    } else if widget_node.data().flag_is_set(ViewFlags::FOCUSABLE) {
        Some(widget_node.data().id_path().clone())
    } else {
        None
    }
}*/

pub struct MouseEventContext<'a> {
    id: WidgetId, 
    app_state: &'a mut AppState,
}

impl<'a> MouseEventContext<'a> {
    fn dispatch(&mut self, event: MouseEvent) -> EventStatus {
        let mut widget = self.app_state.widgets.remove(self.id).expect("Widget not found");
        let old_id = self.id;
        let status = widget.mouse_event(event, self);
        self.id = old_id;
        self.app_state.widgets.insert(old_id, widget);
        status
    }

    pub fn capture_mouse(&mut self) {
        self.app_state.mouse_capture_widget = Some(self.id)
    }

    pub fn request_render(&mut self) {
        invalidate_window(&mut self.app_state, self.app_state.get_window_id_for_widget(self.id));
    }
}


pub struct EventContext<'a> {
    id: WidgetId, 
    app_state: &'a mut AppState,
}

impl<'a> EventContext<'a> {
    pub fn new(id: WidgetId, app_state: &'a mut AppState) -> Self{
        Self { id, app_state }
    }

    pub fn bounds(&self) -> Rectangle {
        self.app_state.widget_data_ref(self.id).global_bounds()
    }

    /*pub(crate) fn with_child<'d, T>(&mut self, widget_data: &'d mut WidgetData, f: impl FnOnce(&mut EventContext<'d, '_, '_>) -> T) -> T {
        let (value, flags) = {
            let mut ctx = EventContext { 
                widget_data,
                window_state: self.window_state,
                handle: self.handle,
                app_state: self.app_state
            };
            let value = f(&mut ctx);
            (value, ctx.view_flags())
        };

        self.widget_data.flags |= flags & (WidgetFlags::NEEDS_LAYOUT | WidgetFlags::NEEDS_RENDER);
		value
    }*/

    pub fn take_focus(&mut self) {
        self.app_state.focus_widget = Some(self.id)
    }

    pub fn request_layout(&mut self) {
        self.app_state.widget_data_mut(self.id).set_flag(WidgetFlags::NEEDS_LAYOUT);
    }

    pub fn request_render(&mut self) {
        invalidate_window(&mut self.app_state, self.app_state.get_window_id_for_widget(self.id));
    }

    pub fn get_clipboard(&mut self) -> Option<String> {
        None
        //self.handle.get_clipboard().ok().flatten()
    }

    pub fn set_clipboard(&mut self, string: &str) {
        //self.handle.set_clipboard(string).unwrap();
    }

    pub fn set_cursor(&mut self, cursor: Cursor) {
        //self.app_state.cursor = cursor;
    }
}