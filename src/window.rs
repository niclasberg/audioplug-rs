use raw_window_handle::RawWindowHandle;

use crate::core::{Point, Rectangle};
use crate::event::KeyEvent;
use crate::platform::WindowEvent;
use crate::view::{BuildContext, EventContext, EventStatus, LayoutNodeRef, RenderContext, View, ViewMessage, ViewMessageBody, Widget, WidgetData, WidgetNode};
use crate::{platform, IdPath, MouseEvent};

pub trait WindowHandler {
	fn init(&mut self, handle: platform::HandleRef);
    fn event(&mut self, event: WindowEvent, handle: platform::HandleRef);
    fn render(&mut self, bounds: Rectangle, renderer: platform::RendererRef);
}

pub struct Window(platform::Window);

pub struct WindowState {
    pub(crate) mouse_capture_view: Option<IdPath>,
    pub(crate) focus_view: Option<IdPath>
}

struct MyHandler {
    widget_node: WidgetNode,
    window_state: WindowState
}

impl MyHandler {
    pub fn new<V: View>(view: V) -> Self {
        let mut data = WidgetData::new(IdPath::root());
        let mut build_context = BuildContext::root(&mut data);
        let widget: Box<dyn Widget> = Box::new(view.build(&mut build_context));
        let data = data.with_style(|style| *style = widget.style());
        let widget_node = WidgetNode { 
            widget,
            data
        };
        let window_state = WindowState {
            mouse_capture_view: None,
            focus_view: None
        };
        
        Self { widget_node, window_state }
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
        //self.view_node.clear_flag_recursive(ViewFlags::NEEDS_LAYOUT);
    }

    fn default_handle_key_event(&mut self, event: KeyEvent) {

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
                if let Some(mut capture_view) = self.window_state.mouse_capture_view.clone() {
                    let mut ctx = EventContext::new(&mut self.widget_node.data, &mut self.window_state, &mut handle);
                    capture_view.pop_root();
                    let mut msg = ViewMessage {
                        destination: capture_view,
                        body: ViewMessageBody::Mouse(mouse_event)
                    };
                    msg.handle(&mut self.widget_node.widget, &mut ctx);

                    match mouse_event {
                        MouseEvent::Up { .. } => {
                            self.window_state.mouse_capture_view = None
                        },
                        _ => {}
                    }
                } else {
                    let mut ctx = EventContext::new(&mut self.widget_node.data, &mut self.window_state, &mut handle);
                    self.widget_node.widget.mouse_event(mouse_event, &mut ctx);
                }
            },
            WindowEvent::Key(key_event) => {
                if let Some(mut focus_view) = self.window_state.focus_view.clone() {
                    let mut ctx = EventContext::new(&mut self.widget_node.data, &mut self.window_state, &mut handle);
                    focus_view.pop_root();
                    let event_status = handle_key_event(&mut focus_view, key_event.clone(), &mut self.widget_node.widget, &mut ctx);
                    if event_status == EventStatus::Ignored {
                        self.default_handle_key_event(key_event);
                    }
                }
            },
            WindowEvent::Unfocused => {
                if let Some(focus_view) = self.window_state.focus_view.take() {
                    let mut ctx = EventContext::new(&mut self.widget_node.data, &mut self.window_state, &mut handle);
                    let mut msg = ViewMessage {
                        destination: focus_view,
                        body: ViewMessageBody::FocusChanged(false),
                    };
                    msg.handle(&mut self.widget_node.widget, &mut ctx)
                }
            },
			_ => {}
		};

        if self.widget_node.layout_requested() {
            self.do_layout(&mut handle);
        }
    }

    fn render(&mut self, _: Rectangle, mut renderer: platform::RendererRef<'_>) {
        {
            let mut ctx = RenderContext::new(&mut self.widget_node.data, &mut renderer, &mut self.window_state);
            self.widget_node.widget.render(&mut ctx);
        }
    }

    fn init(&mut self, mut handle: platform::HandleRef) {
        self.do_layout(&mut handle);
    }
}

pub(crate) fn handle_key_event(id_path: &mut IdPath, event: KeyEvent, widget: &mut dyn Widget, ctx: &mut EventContext) -> EventStatus {
    let mut status = EventStatus::Ignored;
    if let Some(child_id) = id_path.pop_root() {
        if child_id.0 < widget.child_count() {
            let child = widget.get_child_mut(child_id.0);
            ctx.with_child(&mut child.data, |ctx| {
                status = handle_key_event(id_path, event.clone(), &mut child.widget, ctx);
                ctx.view_flags()
            });
        }
    } 
    
    if status == EventStatus::Handled {
        EventStatus::Handled
    } else {
        widget.key_event(event, ctx)
    }
}

impl Window {
    pub fn open(view: impl View + 'static) -> Self {
        let handler = MyHandler::new(view);
        Self(platform::Window::open(handler).unwrap())
    }

    pub fn attach(handle: RawWindowHandle, view: impl View + 'static) -> Self {
        let handler = MyHandler::new(view);
        let window: Result<platform::Window, platform::Error> = match handle {
            #[cfg(target_os = "windows")]
            RawWindowHandle::Win32(handle) => {
                platform::Window::attach(windows::Win32::Foundation::HWND(handle.hwnd as isize), handler)
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