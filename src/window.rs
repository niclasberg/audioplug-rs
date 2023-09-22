use std::any::Any;

use raw_window_handle::RawWindowHandle;

use crate::core::{Rectangle, Color, Size, Transform, Constraint};
use crate::view::{View, ViewNode, EventContext};
use crate::{Event, Message, ViewMessage, Id, IdPath, LayoutContext};

#[cfg(target_os = "windows")]
use crate::win as platform;
#[cfg(target_os = "macos")]
use crate::mac as platform;

pub trait WindowHandler {
    fn event(&mut self, event: Event);
    fn render(&mut self, bounds: Rectangle, renderer: &mut platform::Renderer);
}

pub struct Renderer<'a>(pub(crate) &'a mut platform::Renderer);

impl<'a> Renderer<'a> {
    pub fn draw_rectangle(&mut self, rect: Rectangle, color: Color, line_width: f32) {
        self.0.draw_rectangle(rect, color, line_width)
    }

    pub fn fill_rectangle(&mut self, rect: Rectangle, color: Color) {
        self.0.fill_rectangle(rect, color);
    }

    pub fn fill_rounded_rectangle(&mut self, rect: Rectangle, radius: Size, color: Color) {
        self.0.fill_rounded_rectangle(rect, radius, color);
    }

    pub fn draw_text(&mut self, text: &str, bounds: Rectangle) {
        self.0.draw_text(text, bounds)
    }

    pub fn use_transform(&mut self, transform: Transform, f: impl FnOnce(&mut Renderer) -> ()) {
        //self.0.use_transform(transform, |);
        f(self)
    }
}

pub struct Window(platform::Window);

struct MyHandler<V: View> {
    view: V,
    view_state: V::State,
    view_meta: ViewNode,
    messages: Vec<ViewMessage<Box<dyn Any>>>,
}

impl<V: View> MyHandler<V> {
    pub fn new(mut view: V) -> Self {
        let view_meta = ViewNode::new();
        let view_state = view.build(&IdPath::root());
        
        Self { view, view_state, view_meta, messages: Vec::new() }
    }

    fn dispatch_messages_to_views(&mut self) {
        for message in self.messages.iter() {
            println!("Message to; {:?}", message.view_id)
        }
        self.messages.clear();
    }
}

impl<V: View + 'static> WindowHandler for MyHandler<V> {
    fn event(&mut self, event: Event) {
        let mut ctx = EventContext::<V::Message>::new(&mut self.view_meta, &mut self.messages);
        self.view.event(&mut self.view_state, event, &mut ctx);
        self.dispatch_messages_to_views();
    }

    fn render(&mut self, bounds: Rectangle, renderer: &mut platform::Renderer) {
        let constraint = Constraint::exact(bounds.size());
		let mut ctx = LayoutContext::new(&mut self.view_meta);
        let layout = self.view.layout(&self.view_state, constraint, &mut ctx);

        let mut ctx = Renderer(renderer);
        self.view.render(&self.view_state, bounds, &mut ctx);
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