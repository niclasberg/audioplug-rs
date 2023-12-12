use std::cell::RefCell;
use std::os::raw::c_void;
use std::ptr::NonNull;

use icrate::AppKit::{NSView, NSEvent, NSTrackingArea, NSTrackingActiveAlways, NSTrackingMouseEnteredAndExited, NSTrackingMouseMoved};
use icrate::Foundation::{NSRect, NSDictionary, CGRect};
use objc2::rc::Id;
use objc2::declare::{IvarDrop, Ivar};
use objc2::exception;
use objc2::{declare_class, mutability, ClassType, msg_send_id, msg_send};

use crate::core::Point;
use crate::event::{MouseButton, KeyEvent};
use crate::platform::mac::keyboard::{key_from_code, get_modifiers};
use crate::{MouseEvent, Event};
use super::{RendererRef, HandleRef};
use crate::window::WindowHandler;

use super::appkit::NSGraphicsContext;
use super::core_graphics::CGColor;

pub struct ViewState {
	handler: RefCell<Box<dyn WindowHandler>>,
	tracking_area: RefCell<Option<Id<NSTrackingArea>>>
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

				let this = NonNull::from(this);

				this.as_ref().state.handler.borrow_mut().init(HandleRef::new(this.as_ref()));

				this
			})
		}

		#[method(isFlipped)]
		fn is_flipped(&self) -> bool {
			true
		}

		#[method(keyDown:)]
        fn key_down(&self, event: &NSEvent) {
			let key = key_from_code(unsafe { event.keyCode() });
			let modifiers = get_modifiers(unsafe { event.modifierFlags() });
			let str = unsafe { event.characters() };
			let str = str.map(|str| str.to_string()).filter(|str| str.len() > 0);
			let key_event = KeyEvent::KeyDown { key, modifiers, str };
			self.dispatch_event(Event::Keyboard(key_event));
		}

		#[method(mouseDown:)]
        fn mouse_down(&self, event: &NSEvent) {
			if let (Some(button), Some(position)) = (mouse_button(event), self.mouse_position(event)) {
				self.dispatch_event(Event::Mouse(
					MouseEvent::Down { button, position }
				))
			}
		}

		#[method(mouseUp:)]
        fn mouse_up(&self, event: &NSEvent) {
			if let (Some(button), Some(position)) = (mouse_button(event), self.mouse_position(event)) {
				self.dispatch_event(Event::Mouse(
					MouseEvent::Up { button, position }
				))
			}
		}

		#[method(mouseMoved:)]
        fn mouse_moved(&self, event: &NSEvent) {
            if let Some(position) = self.mouse_position(event) {
				self.dispatch_event(Event::Mouse(
					MouseEvent::Moved { position }
				))
			}
        }

		#[method(acceptsFirstResponder)]
        fn accepts_first_responder(&self) -> bool {
            true
        }

		#[method(viewDidMoveToWindow)]
		fn view_did_move_to_window(&self) {
			let visible_rect = unsafe { self.visibleRect() };
			self.set_tracking_area(visible_rect);
		}

		#[method(updateTrackingAreas)]
		fn update_tracking_areas(&self) {
			let visible_rect = unsafe { self.visibleRect() };
			self.set_tracking_area(visible_rect);
		}

		#[method(acceptsFirstMouse:)]
        fn accepts_first_mouse(&self, _event: &NSEvent) -> bool {
            true
        }

		#[method(drawRect:)]
		fn draw_rect(&self, rect: NSRect) {
			let graphics_context = NSGraphicsContext::current().unwrap();
			let context = graphics_context.cg_context();

			let color = CGColor::from_rgba(1.0, 0.0, 1.0, 1.0);
			context.set_fill_color(&color);
			context.fill_rect(rect);
			
			let renderer = RendererRef::new(context);

			self.state.handler.borrow_mut().render(
				rect.into(),
				renderer
			);
		}
	}
);

impl View {
	pub(crate) fn new(handler: impl WindowHandler + 'static) -> Id<Self> {
		let handler = RefCell::new(Box::new(handler));
		let tracking_area = RefCell::new(None);
		let state = Box::new(ViewState { handler, tracking_area });

		unsafe {
			msg_send_id![
				Self::alloc(), 
				initWithHandler: Box::into_raw(state) as *mut c_void
			]
		}
	}

	fn dispatch_event(&self, event: Event) {
		self.state.handler.borrow_mut().event(event, HandleRef::new(&self) )
	}

	fn set_tracking_area(&self, rect: CGRect) {
		// Use try-borrow here to avoid re-entrancy problems
		if let Ok(mut tracking_area_ref) = self.state.tracking_area.try_borrow_mut() {
			if let Some(tracking_area) = tracking_area_ref.as_ref() {
				unsafe { self.removeTrackingArea(tracking_area) };
				*tracking_area_ref = None;
			}
	
			let tracking_area = unsafe { 
				let tracking_area = NSTrackingArea::alloc();
				NSTrackingArea::initWithRect_options_owner_userInfo(tracking_area, 
					rect, 
					NSTrackingActiveAlways | NSTrackingMouseEnteredAndExited | NSTrackingMouseMoved, 
					Some(self), 
					None)
			};
	
			let tracking_area = tracking_area_ref.insert(tracking_area);
			unsafe { self.addTrackingArea(tracking_area) };
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
			let y = pos.y;
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