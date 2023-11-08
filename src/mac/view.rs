use std::cell::RefCell;
use std::os::raw::c_void;
use std::ptr::NonNull;

use icrate::AppKit::{NSView, NSEvent};
use icrate::Foundation::{NSRect, CGPoint};
use objc2::rc::{Id, WeakId};
use objc2::declare::{IvarDrop, Ivar};
use objc2::{declare_class, mutability, ClassType, msg_send_id, msg_send};

use crate::core::Point;
use crate::event::MouseButton;
use crate::mac::core_graphics::CGAffineTransform;
use crate::{MouseEvent, Event};
use crate::mac::RendererRef;
use crate::window::WindowHandler;

use super::appkit::NSGraphicsContext;
use super::core_graphics::CGColor;

pub struct ViewState {
	handler: RefCell<Box<dyn WindowHandler>>
}

declare_class!(
	pub(crate) struct View {
		state: IvarDrop<Box<ViewState>, "_state">
	}
	mod ivars;

	unsafe impl ClassType for View {
		type Super = NSView;
		type Mutability = mutability::InteriorMutable;
		const NAME: &'static str = "MyView";
	}

	unsafe impl View {
		#[method(initWithHandler:)]
		unsafe fn init_with_handler(this: *mut Self, state_ptr: *mut c_void) -> Option<NonNull<Self>> {
			let this: Option<&mut Self> = unsafe { msg_send![super(this), init] };
			this.map(|this| {
				let state_ptr = state_ptr as *mut ViewState;
				let state = Box::from_raw(state_ptr);

				Ivar::write(&mut this.state, state);

				NonNull::from(this)
			})
		}

		#[method(isFlipped)]
		fn is_flipped(&self) -> bool {
			true
		}

		#[method(keyDown:)]
        fn key_down(&self, event: &NSEvent) {
			println!("assd");
		}

		#[method(mouseDown:)]
        fn mouse_down(&self, event: &NSEvent) {
			if let (Some(button), Some(position)) = (mouse_button(event), self.mouse_position(event)) {
				self.state.handler.borrow_mut().event(Event::Mouse(
					MouseEvent::Down { button, position }
				))
			}
		}

		#[method(mouseUp:)]
        fn mouse_up(&self, event: &NSEvent) {
			if let (Some(button), Some(position)) = (mouse_button(event), self.mouse_position(event)) {
				self.state.handler.borrow_mut().event(Event::Mouse(
					MouseEvent::Up { button, position }
				))
			}
		}

		#[method(mouseMoved:)]
        fn mouse_moved(&self, event: &NSEvent) {
			println!("Moved");
            if let Some(position) = self.mouse_position(event) {
				self.state.handler.borrow_mut().event(Event::Mouse(
					MouseEvent::Moved { position }
				))
			}
        }

		#[method(drawRect:)]
		fn draw_rect(&self, rect: NSRect) {
			let a = unsafe { self.frame() };
			println!("Draw: {:?}", a);

			let graphics_context = NSGraphicsContext::current().unwrap();
			let context = graphics_context.cg_context();

			context.set_text_matrix(CGAffineTransform::scale(1.0, -1.0));

			let color = CGColor::from_rgba(1.0, 0.0, 1.0, 1.0);
			context.set_fill_color(&color);
			context.fill_rect(rect);
			
			let renderer = RendererRef { context };

			self.state.handler.borrow_mut().render(
				rect.into(),
				renderer
			);
		}
	}
);

impl View {
	pub(crate) fn new(handler: impl WindowHandler + 'static) -> Id<Self> {
		let state = Box::new(ViewState { handler: RefCell::new(Box::new(handler)) });

		unsafe {
			msg_send_id![
				Self::alloc(), 
				initWithHandler: Box::into_raw(state) as *mut c_void
			]
		}
	}

	fn mouse_position(&self, event: &NSEvent) -> Option<Point> {
		let pos = unsafe { event.locationInWindow() };
		let frame = unsafe { self.frame() };
		let pos = unsafe { self.convertPoint_fromView(pos, None) };

		if pos.x.is_sign_negative() ||
			pos.y.is_sign_negative() ||
			pos.x > frame.size.width ||
			pos.y > frame.size.height 
		{
			None
		} else {
			let x = pos.x;
			let y = frame.size.height - pos.y;
			Some(Point::new(x, y))
		}
	}
}

fn mouse_button(event: &NSEvent) -> Option<MouseButton> {
	match unsafe { event.buttonNumber() } {
		0 => Some(MouseButton::LEFT),
		1 => Some(MouseButton::RIGHT),
		2 => Some(MouseButton::MIDDLE),
		_ => None
	}
}