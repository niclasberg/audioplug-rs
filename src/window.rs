use std::any::Any;
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::rc::Rc;

use raw_window_handle::RawWindowHandle;

use crate::core::{Point, Rectangle};
use crate::event::KeyEvent;
use crate::keyboard::Key;
use crate::platform::{WindowEvent, WindowHandler};
use crate::app::{self, AppState, Signal, SignalContext};
use crate::view::{BuildContext, EventContext, EventStatus, LayoutNodeRef, RenderContext, View, ViewFlags, ViewMessage, Widget, WidgetData, WidgetNode};
use crate::{platform, App, Cursor, IdPath, MouseEvent};

pub struct WindowState {
    pub(crate) mouse_capture_view: Option<IdPath>,
    pub(crate) focus_view: Option<IdPath>,
    pub(crate) cursor: Cursor
}

struct MyHandler {
    widget_node: WidgetNode,
    state: WindowState,
    app_state: Rc<RefCell<AppState>>
}

impl MyHandler {
    pub fn new<F, V>(app_state: Rc<RefCell<AppState>>, view_factory: F) -> Self 
    where 
        F: FnOnce(&mut AppContext) -> V,
        V: View
    {
        let mut data = WidgetData::new(IdPath::root());
        let mut window_state = WindowState {
            mouse_capture_view: None,
            focus_view: None,
            cursor: Cursor::Arrow
        };

        let view = {
            let mut app_state = RefCell::borrow_mut(&app_state);
            let mut app_context = AppContext {
                window_state: &mut window_state,
                app_state: &mut app_state
            };
            view_factory(&mut app_context)
        };

        let widget: Box<dyn Widget> = {
            let mut app_state = RefCell::borrow_mut(&app_state);
            let mut build_context = BuildContext::root(&mut data, &mut app_state);
            Box::new(view.build(&mut build_context))
        };

        let data = data.with_style(|style| *style = widget.style());
        let widget_node = WidgetNode { 
            widget,
            data
        };
        
        Self { 
            widget_node, 
            state: window_state,
            app_state
        }
    }

    fn do_layout(&mut self, handle: &mut platform::HandleRef) {
        let bounds = handle.global_bounds().size();
        {
            let mut ctx = LayoutNodeRef { handle, widget: &mut self.widget_node.widget, data: &mut self.widget_node.data };
            let available_space = taffy::Size { 
                width: taffy::AvailableSpace::Definite(bounds.width as f32), 
                height: taffy::AvailableSpace::Definite(bounds.height as f32)
            };
            taffy::compute_root_layout(&mut ctx, taffy::NodeId::from(usize::MAX), available_space);
        }
        self.widget_node.set_origin(Point::ZERO);
    }

    fn default_handle_key_event(&mut self, event: KeyEvent, handle: &mut platform::HandleRef) {
        match event {
            KeyEvent::KeyDown { key, modifiers, .. } => {
                match key {
                    Key::Escape if modifiers.is_empty() => {
                        self.set_focus_view(None, handle)
                    },
                    _ => {}
                }
            },
            _ => {}
        }
    }

    fn set_focus_view(&mut self, new_focus_view: Option<IdPath>, handle: &mut platform::HandleRef) {
        if new_focus_view != self.state.focus_view {
            println!("Focus change {:?}, {:?}", self.state.focus_view, new_focus_view);
            if let Some(focus_lost_view) = self.state.focus_view.as_ref() {
                let message = ViewMessage::FocusChanged(false);
                let mut app_state = RefCell::borrow_mut(&self.app_state);
                self.widget_node.handle_message(&focus_lost_view.clone(), message, &mut self.state, handle, &mut app_state);
            }

            self.state.focus_view = new_focus_view.clone();

            if let Some(focus_gained_view) = new_focus_view {
                let message = ViewMessage::FocusChanged(true);
                let mut app_state = RefCell::borrow_mut(&self.app_state);
                self.widget_node.handle_message(&focus_gained_view, message, &mut self.state, handle, &mut app_state)
            }
        }
    }
}

impl WindowHandler for MyHandler {
    fn event(&mut self, event: WindowEvent, mut handle: platform::HandleRef) {
		match event {
			WindowEvent::Resize { .. } => {
                self.do_layout(&mut handle);
                handle.invalidate(handle.global_bounds());
			},
            WindowEvent::Mouse(mouse_event) => {
				match mouse_event {
					MouseEvent::Down { position, .. } => {
						let new_focus_view = find_focus_view_at(position, &self.widget_node);
						self.set_focus_view(new_focus_view, &mut handle);
					},
					_ => {}
				};

                if let Some(mut capture_view) = self.state.mouse_capture_view.clone() {
                    let message = ViewMessage::Mouse(mouse_event);
                    let mut app_state = RefCell::borrow_mut(&self.app_state);
                    self.widget_node.handle_message(&mut capture_view, message, &mut self.state, &mut handle, &mut app_state);

                    match mouse_event {
                        MouseEvent::Up { .. } => {
                            self.state.mouse_capture_view = None
                        },
                        _ => {}
                    }
                } else {
                    let mut app_state = RefCell::borrow_mut(&self.app_state);
                    let mut ctx = EventContext::new(&mut self.widget_node.data, &mut self.state, &mut handle, &mut app_state);
                    self.widget_node.widget.mouse_event(mouse_event, &mut ctx);
                }
            },
            WindowEvent::Key(key_event) => {
				let mut event_status = EventStatus::Ignored;
                if let Some(mut focus_view) = self.state.focus_view.clone() {
                    let mut app_state = RefCell::borrow_mut(&self.app_state);
                    let mut ctx = EventContext::new(&mut self.widget_node.data, &mut self.state, &mut handle, &mut app_state);
                    focus_view.pop_root();
                    event_status = handle_key_event(&mut focus_view, key_event.clone(), &mut self.widget_node.widget, &mut ctx);
                }

				if event_status == EventStatus::Ignored {
					self.default_handle_key_event(key_event, &mut handle);
				}
            },
            WindowEvent::Unfocused => {
                if let Some(mut focus_view) = self.state.focus_view.take() {
                    let message = ViewMessage::FocusChanged(false);
                    let mut app_state = RefCell::borrow_mut(&self.app_state);
					self.widget_node.handle_message(&mut focus_view, message, &mut self.state, &mut handle, &mut app_state)
                }
            },
			_ => {}
		};

        {
            let mut app_state = RefCell::borrow_mut(&self.app_state);
            app_state.run_effects(&mut self.widget_node, &mut self.state, &mut handle)
        }
        
        if self.widget_node.layout_requested() {
            self.do_layout(&mut handle);
        }
    }

    fn render(&mut self, _: Rectangle, mut renderer: platform::RendererRef<'_>) {
        {
            let mut ctx = RenderContext::new(&mut self.widget_node.data, &mut renderer, &mut self.state);
            self.widget_node.widget.render(&mut ctx);
        }
    }

    fn init(&mut self, mut handle: platform::HandleRef) {
        self.do_layout(&mut handle);
    }
    
    fn get_cursor(&self, point: Point) -> Option<Cursor> {
        let mut cursor = None;
        self.widget_node.for_each_view_at(point, &mut |widget_node| {
            if let Some(c) = widget_node.widget.cursor() {
                cursor = Some(c);
                false
            } else {
                true
            }
        });

        cursor
    }
}

fn find_focus_view_at(position: Point, widget_node: &WidgetNode) -> Option<IdPath> {
	if !widget_node.data().global_bounds().contains(position) {
		return None;
	}

	let child_focus_view = (0..widget_node.widget.child_count()).rev().find_map(|i| {
		find_focus_view_at(position, widget_node.widget.get_child(i))
	});

	if child_focus_view.is_some() {
		child_focus_view
	} else if widget_node.data().flag_is_set(ViewFlags::FOCUSABLE) {
		Some(widget_node.data().id_path().clone())
	} else {
		None
	}
}

fn handle_key_event(id_path: &mut IdPath, event: KeyEvent, widget: &mut dyn Widget, ctx: &mut EventContext) -> EventStatus {
    let mut status = EventStatus::Ignored;
    if let Some(child_id) = id_path.pop_root() {
        if child_id.0 < widget.child_count() {
            let child = widget.get_child_mut(child_id.0);
            ctx.with_child(&mut child.data, |ctx| {
                status = handle_key_event(id_path, event.clone(), &mut child.widget, ctx);
            });
        }
    } 
    
    if status == EventStatus::Handled {
        EventStatus::Handled
    } else {
        widget.key_event(event, ctx)
    }
}

pub struct AppContext<'a> {
    window_state: &'a mut WindowState,
    app_state: &'a mut AppState
}

impl<'a> AppContext<'a> {
    pub fn create_signal<T: Any>(&mut self, value: T) -> Signal<T> {
        self.app_state.create_signal(value)
    }

    pub fn create_effect(&mut self, f: impl Fn(&mut AppState) + 'static) {
        self.app_state.create_effect(f)
    }
}

impl<'b> SignalContext for AppContext<'b> {
    fn get_signal_value_ref_untracked<'a, T: Any>(&'a self, signal: &Signal<T>) -> &'a T {
        self.app_state.get_signal_value_ref_untracked(signal)
    }

    fn get_signal_value_ref<'a, T: Any>(&'a mut self, signal: &Signal<T>) -> &'a T {
        self.app_state.get_signal_value_ref(signal)
    }
    
    fn set_signal_value<T: Any>(&mut self, signal: &Signal<T>, value: T) {
        self.app_state.set_signal_value(signal, value)
    }
}

pub struct Window(platform::Window);

impl Window {
    pub fn open<F, V>(app: &mut App, view_factory: F) -> Self 
    where 
        F: FnOnce(&mut AppContext) -> V,
        V: View
    {
        let handler = MyHandler::new(app.state.clone(), view_factory);
        Self(platform::Window::open(handler).unwrap())
    }

    pub fn attach<F, V>(app_state: Rc<RefCell<AppState>>, handle: RawWindowHandle, view_factory: F) -> Self 
    where 
        F: FnOnce(&mut AppContext) -> V,
        V: View
    {
        let handler = MyHandler::new(app_state, view_factory);
        let window: Result<platform::Window, platform::Error> = match handle {
            #[cfg(target_os = "windows")]
            RawWindowHandle::Win32(handle) => {
                let hwnd = handle.hwnd.get() as *mut std::ffi::c_void;
                platform::Window::attach(windows::Win32::Foundation::HWND(hwnd), handler)
            },
			#[cfg(target_os = "macos")]
			RawWindowHandle::AppKit(handle) => {
				todo!()
			}
            _ => panic!("Unsupported window type"),
        };

        Self(window.unwrap())
    }

    pub fn set_size(&self, size: Rectangle<i32>) {
        self.0.set_size(size).unwrap()
    }
}