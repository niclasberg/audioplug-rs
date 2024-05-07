use raw_window_handle::RawWindowHandle;

use crate::core::{Point, Rectangle};
use crate::event::WindowEvent;
use crate::view::{BuildContext, EventContext, LayoutContext, LayoutNodeRef, RenderContext, View, Widget, WidgetData, WidgetNode};
use crate::{platform, Event, IdPath};

pub trait WindowHandler {
	fn init(&mut self, handle: platform::HandleRef);
    fn event(&mut self, event: Event, handle: platform::HandleRef);
    fn render(&mut self, bounds: Rectangle, renderer: platform::RendererRef);
}

pub struct Window(platform::Window);

struct MyHandler {
    widget_node: WidgetNode,
}

impl MyHandler {
    pub fn new<V: View>(view: V) -> Self {
        let mut build_context = BuildContext::root();
        let widget: Box<dyn Widget> = Box::new(view.build(&mut build_context));
        let data = WidgetData::new(IdPath::root())
            .with_style(|style| *style = widget.style());
        let widget_node = WidgetNode { 
            widget,
            data
        };
        
        Self { widget_node }
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
}

impl WindowHandler for MyHandler {
    fn event(&mut self, event: Event, mut handle: platform::HandleRef) {
		match event {
			Event::Window(window_event) => match window_event {
				WindowEvent::Resize { .. } => {
					self.do_layout(&mut handle);
					handle.invalidate(handle.global_bounds());
				},
				_ => {}
			},
			_ => {}
		};

        {
            let mut is_handled = false;
            let mut ctx = EventContext::new(&mut self.widget_node.data, &mut is_handled, &mut handle);
            self.widget_node.widget.event(event, &mut ctx);
        }

        if self.widget_node.layout_requested() {
            self.do_layout(&mut handle);
        }
    }

    fn render(&mut self, _: Rectangle, mut renderer: platform::RendererRef<'_>) {
        {
            let mut ctx = RenderContext::new(&mut self.widget_node.data, &mut renderer);
            self.widget_node.widget.render(&mut ctx);
        }
    }

    fn init(&mut self, mut handle: platform::HandleRef) {
        self.do_layout(&mut handle);
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