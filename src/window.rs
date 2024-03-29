use raw_window_handle::RawWindowHandle;

use crate::core::{Rectangle, Constraint, Point};
use crate::event::WindowEvent;
use crate::view::{View, ViewNode, EventContext};
use crate::{Event, ViewMessage, LayoutContext, BuildContext, RenderContext, ViewFlags, platform};

pub trait WindowHandler {
	fn init(&mut self, handle: platform::HandleRef);
    fn event(&mut self, event: Event, handle: platform::HandleRef);
    fn render(&mut self, bounds: Rectangle, renderer: platform::RendererRef);
}

pub struct Window(platform::Window);

struct MyHandler<V: View> {
    view: V,
    view_node: ViewNode,
    messages: Vec<ViewMessage<()>>,
}

impl<V: View> MyHandler<V> {
    pub fn new(mut view: V) -> Self {
        let mut view_meta = ViewNode::new();
        let mut build_context = BuildContext::root(&mut view_meta);
        let view_state = view.build(&mut build_context);
        
        Self { view, view_node: view_meta, messages: Vec::new() }
    }

    fn dispatch_messages_to_views(&mut self) {
        for message in self.messages.iter() {
            println!("Message to; {:?}", message.view_id)
        }
        self.messages.clear();
    }

    fn do_layout(&mut self, handle: &mut platform::HandleRef) {
        let constraint = Constraint::exact(handle.global_bounds().size());
        let size = {
            let mut ctx = LayoutContext::new(&mut self.view_node, handle);
            let size = self.view.layout(&mut self.view_state, constraint, &mut ctx);
            constraint.clamp(size)
        };
        self.view_node.set_size(size);
        self.view_node.set_origin(Point::ZERO);
        self.view_node.clear_flag_recursive(ViewFlags::NEEDS_LAYOUT);
    }
}

impl<V: View + 'static> WindowHandler for MyHandler<V> {
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
            let mut ctx = EventContext::new(&mut self.view_node, &mut self.messages, &mut is_handled, &mut handle);
            self.view.event(&mut self.view_state, event, &mut ctx);
        }
        {
            let mut ctx = BuildContext::root(&mut self.view_node);
            self.view.rebuild(&mut self.view_state, &mut ctx)
        }
		{
			self.do_layout(&mut handle);
		}
        // 1. Dispatch messages, may update the state
        // 2. Rebuild if was requested, or the state was updated, rebuild
        // 3. Perform layout, if requested
        // 4. Render
        self.dispatch_messages_to_views();
    }

    fn render(&mut self, _: Rectangle, mut renderer: platform::RendererRef<'_>) {
        {
            let mut ctx = RenderContext::new(&mut self.view_node, &mut renderer);
            self.view.render(&mut ctx);
        }
        self.view_node.clear_flag_recursive(ViewFlags::NEEDS_RENDER);
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