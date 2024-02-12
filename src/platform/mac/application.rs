use icrate::Foundation::{NSObject, MainThreadMarker};
use icrate::AppKit::{NSApplication, NSApplicationDelegate, NSApplicationActivationPolicyRegular};
use objc2::rc::Id;
use objc2::runtime::{NSObjectProtocol, ProtocolObject, AnyObject};
use objc2::{declare_class, DeclaredClass, mutability, ClassType, msg_send_id};

declare_class!(
	struct MyApplicationDelegate;

	unsafe impl ClassType for MyApplicationDelegate {
		type Super = NSObject;
		type Mutability = mutability::MainThreadOnly;
		const NAME: &'static str = "MyApplicationDelegate";
	}

	impl DeclaredClass for MyApplicationDelegate {
        type Ivars = ();
    }

	unsafe impl NSApplicationDelegate for MyApplicationDelegate {
		#[method(applicationDidFinishLaunching:)]
        unsafe fn did_finish_launching(&self, sender: *mut AnyObject) {
            println!("Did finish launching!");
            // Do something with `sender`
            dbg!(sender);
        }
	}

	unsafe impl NSObjectProtocol for MyApplicationDelegate {}
);

impl MyApplicationDelegate {
	fn new(mtm: MainThreadMarker) -> Id<Self> {
		unsafe {
			msg_send_id![mtm.alloc(), init]
		}
	}
}

pub(crate) struct Application {
	app: Id<NSApplication>,
	delegate: Id<MyApplicationDelegate>
}

impl Application {
	pub fn new() -> Self {
		let mtm = MainThreadMarker::new().unwrap();
		let app: Id<NSApplication> = NSApplication::sharedApplication(mtm);
		app.setActivationPolicy(NSApplicationActivationPolicyRegular);

		let delegate: Id<MyApplicationDelegate> = MyApplicationDelegate::new(mtm);

		let object = ProtocolObject::from_ref(&*delegate);
		app.setDelegate(Some(object));

		Self { app, delegate }
	}

	pub fn run(&mut self) {
		unsafe { self.app.run() };
	}
}