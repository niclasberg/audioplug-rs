use icrate::Foundation::{NSObject, NSPoint, NSRect, NSSize};
use icrate::AppKit::{NSWindow, NSApplication, NSApplicationDelegate, NSView, NSEvent, NSBackingStoreBuffered, NSWindowStyleMaskClosable, NSWindowStyleMaskResizable, NSWindowStyleMaskTitled, NSApplicationActivationPolicyRegular};
use objc2::rc::Id;
use objc2::runtime::{NSObjectProtocol, ProtocolObject, AnyObject};
use objc2::{declare_class, mutability, ClassType, msg_send_id};

declare_class!(
	struct MyApplicationDelegate {

	}

	unsafe impl ClassType for MyApplicationDelegate {
		type Super = NSObject;
		type Mutability = mutability::InteriorMutable;
		const NAME: &'static str = "MyApplicationDelegate";
	}

	unsafe impl MyApplicationDelegate {
		#[method(applicationDidFinishLaunching:)]
        unsafe fn did_finish_launching(&self, sender: *mut AnyObject) {
            println!("Did finish launching!");
            // Do something with `sender`
            dbg!(sender);
        }
	}

	unsafe impl NSApplicationDelegate for MyApplicationDelegate {}

	unsafe impl NSObjectProtocol for MyApplicationDelegate {}
);

impl MyApplicationDelegate {
	fn new() -> Id<Self> {
		unsafe {
			msg_send_id![Self::alloc(), init]
		}
	}
}

pub(crate) struct Application {
	app: Id<NSApplication>,
	delegate: Id<MyApplicationDelegate>
}

impl Application {
	pub fn new() -> Self {
		let app: Id<NSApplication> = unsafe { NSApplication::sharedApplication() };
		unsafe { app.setActivationPolicy(NSApplicationActivationPolicyRegular) };

		let delegate: Id<MyApplicationDelegate> = MyApplicationDelegate::new();

		unsafe { 
			let object = ProtocolObject::from_ref(&*delegate);
			app.setDelegate(Some(object));
		};

		Self { app, delegate }
	}

	pub fn run(&mut self) {
		unsafe { self.app.run() };
	}
}