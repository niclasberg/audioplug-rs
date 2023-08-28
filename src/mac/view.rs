use icrate::AppKit::{NSView, NSEvent};
use icrate::Foundation::NSRect;
use objc2::rc::{Id, WeakId};
use objc2::declare::IvarDrop;
use objc2::{declare_class, mutability, ClassType, msg_send_id};

use super::Window;
use super::appkit::NSGraphicsContext;
use super::core_graphics::CGColor;

trait ViewHandler {

}

declare_class!(
	pub(crate) struct View {
		window: IvarDrop<Box<WeakId<Window>>, "__ns_window">
	}
	mod ivars;

	unsafe impl ClassType for View {
		type Super = NSView;
		type Mutability = mutability::InteriorMutable;
		const NAME: &'static str = "MyView";
	}

	unsafe impl View {
		#[method(keyDown:)]
        fn key_down(&self, event: &NSEvent) {
			println!("assd");
		}

		#[method(mouseDown:)]
        fn mouse_down(&self, event: &NSEvent) {
			let pos = unsafe { event.locationInWindow() };
			println!("{:?}", pos);
		}

		#[method(mouseUp:)]
        fn mouse_up(&self, event: &NSEvent) {
			let pos = unsafe { event.locationInWindow() };
			println!("{:?}", pos);
		}

		#[method(drawRect:)]
		fn draw_rect(&self, rect: NSRect) {
			let a = unsafe { self.frame() };
			println!("Draw: {:?}", a);
			let graphics_context = NSGraphicsContext::current().unwrap();
			let context = graphics_context.cg_context();
			let color = CGColor::from_rgba(1.0, 0.0, 1.0, 1.0);
			context.set_fill_color(&color);
			context.fill_rect(rect);
			//let rect = CGRect::new(&CGPoint::new(0.0, 0.0), &CGSize::new(100.0, 100.0));
			//
			//context.fill_rect(rect);
		}
	}
);

impl View {
	pub(crate) fn new() -> Id<Self> {
		unsafe {
			msg_send_id![Self::alloc(), init]
		}
	}
}