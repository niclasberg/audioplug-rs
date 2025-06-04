use objc2::rc::Retained;
use objc2::runtime::{AnyObject, NSObjectProtocol, ProtocolObject};
use objc2::{define_class, msg_send, MainThreadOnly};
use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy, NSApplicationDelegate};
use objc2_foundation::{MainThreadMarker, NSObject};

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "MyApplicationDelegate"]
    struct MyApplicationDelegate;

    unsafe impl NSApplicationDelegate for MyApplicationDelegate {
        #[unsafe(method(applicationDidFinishLaunching:))]
        unsafe fn did_finish_launching(&self, sender: *mut AnyObject) {
            println!("Did finish launching!");
            // Do something with `sender`
            dbg!(sender);
        }
    }

    unsafe impl NSObjectProtocol for MyApplicationDelegate {}
);

impl MyApplicationDelegate {
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send![mtm.alloc(), init] }
    }
}

pub(crate) struct Application {
    app: Retained<NSApplication>,
    _delegate: Retained<MyApplicationDelegate>,
}

impl Application {
    pub fn new() -> Self {
        let mtm = MainThreadMarker::new().unwrap();
        let app = NSApplication::sharedApplication(mtm);
        app.setActivationPolicy(NSApplicationActivationPolicy::Regular);

        let _delegate = MyApplicationDelegate::new(mtm);
        let object = ProtocolObject::from_ref(&*_delegate);
        app.setDelegate(Some(object));

        Self { app, _delegate }
    }

    pub fn run(&mut self) {
        self.app.run();
    }
}
