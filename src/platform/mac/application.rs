use objc2::rc::Retained;
use objc2::runtime::{AnyObject, NSObjectProtocol, ProtocolObject};
use objc2::{MainThreadOnly, define_class, msg_send, sel};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSApplicationDelegate, NSMenu, NSMenuItem,
};
use objc2_foundation::{MainThreadMarker, NSObject, NSProcessInfo, NSString, ns_string};

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "MyApplicationDelegate"]
    struct MyApplicationDelegate;

    unsafe impl NSApplicationDelegate for MyApplicationDelegate {
        #[unsafe(method(applicationDidFinishLaunching:))]
        unsafe fn did_finish_launching(&self, _sender: *mut AnyObject) {
            let mtm = MainThreadMarker::new().expect("Should be called on the main thread");
            let main_menu = NSMenu::new(mtm);
            NSApplication::sharedApplication(mtm).setMainMenu(Some(&main_menu));

            let app_menu_item = NSMenuItem::new(mtm);
            main_menu.addItem(&app_menu_item);

            let app_menu =
                unsafe { NSMenu::initWithTitle(NSMenu::alloc(mtm), ns_string!("Application")) };
            app_menu_item.setSubmenu(Some(&app_menu));

            let process_name = NSProcessInfo::processInfo().processName();

            let quit_title = NSString::from_str(&format!("Quit {process_name}"));
            let quit_item = unsafe {
                NSMenuItem::initWithTitle_action_keyEquivalent(
                    NSMenuItem::alloc(mtm),
                    &quit_title,
                    Some(sel!(terminate:)),
                    ns_string!("q"),
                )
            };
            app_menu.addItem(&quit_item);
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
