use std::cell::{OnceCell, RefCell};

use objc2::rc::{Retained, Weak};
use objc2::runtime::{NSObject, NSObjectProtocol};
use objc2::{define_class, msg_send, sel, AllocAnyThread, DefinedClass, MainThreadOnly};
use objc2_app_kit::{
    NSEvent, NSGraphicsContext, NSResponder, NSTrackingArea, NSTrackingAreaOptions, NSView,
    NSViewFrameDidChangeNotification,
};
use objc2_core_image::CIContext;
use objc2_foundation::{MainThreadMarker, NSDate, NSNotificationCenter, NSRect, NSTimer};
use objc2_metal::MTLCreateSystemDefaultDevice;

use super::{Handle, RendererRef};
use crate::core::{Point, Vec2};
use crate::event::{KeyEvent, MouseButton, MouseEvent};
use crate::platform::mac::keyboard::{get_modifiers, key_from_code};
use crate::platform::WindowEvent;
use crate::platform::WindowHandler;
use crate::AnimationFrame;

pub struct Ivars {
    handler: RefCell<Box<dyn WindowHandler>>,
    tracking_area: RefCell<Option<Retained<NSTrackingArea>>>,
    timer: RefCell<Option<Retained<NSTimer>>>,
    animation_start: OnceCell<Retained<NSDate>>,
    ci_context: OnceCell<Retained<CIContext>>,
}

const CLASS_NAME: &'static str = match option_env!("AUDIOPLUG_VIEW_CLASS_NAME") {
    Some(name) => name,
    None => "AudioPlug_View",
};

// There is a problem in objc2 that we need to address. If we have two different plugins
// in different dylibs and we try to instantiate a view for both, then they will both
// try to register the class with the objc runtime. This will panic. We could version mark
// the class name, and if the class already is registered, we could just look it up.
// We cannot use declare_class! in that case, but have to write the boilerplate ourselves.
define_class!(
    #[unsafe(super(NSView, NSResponder, NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = CLASS_NAME]
    #[ivars = Ivars]
    pub struct View;

    impl View {
        #[unsafe(method(isFlipped))]
        fn is_flipped(&self) -> bool {
            true
        }

        #[unsafe(method(keyDown:))]
        fn key_down(&self, event: &NSEvent) {
            let key = key_from_code(unsafe { event.keyCode() });
            let modifiers = get_modifiers(unsafe { event.modifierFlags() });
            let str = unsafe { event.characters() };
            let str = str.map(|str| str.to_string()).filter(|str| str.len() > 0);
            let repeat_count = if unsafe { event.isARepeat() } { 1 } else { 0 };
            let key_event = KeyEvent::KeyDown { key, modifiers, str, repeat_count };
            self.dispatch_event(WindowEvent::Key(key_event));
        }

        #[unsafe(method(mouseDown:))]
        fn mouse_down(&self, event: &NSEvent) {
            if let (Some(button), Some(position)) = (mouse_button(event), self.mouse_position(event)) {
                let click_count = unsafe { event.clickCount() };
                let modifiers = get_modifiers(unsafe { event.modifierFlags() });
                self.dispatch_event(WindowEvent::Mouse(
                    MouseEvent::Down { button, position, modifiers, is_double_click: click_count >= 2 }
                ))
            }
        }

        #[unsafe(method(mouseUp:))]
        fn mouse_up(&self, event: &NSEvent) {
            if let (Some(button), Some(position)) = (mouse_button(event), self.mouse_position(event)) {
                let modifiers = get_modifiers(unsafe { event.modifierFlags() });
                self.dispatch_event(WindowEvent::Mouse(
                    MouseEvent::Up { button, position, modifiers }
                ))
            }
        }

        #[unsafe(method(mouseMoved:))]
        fn mouse_moved(&self, event: &NSEvent) {
            if let Some(position) = self.mouse_position(event) {
                let modifiers = get_modifiers(unsafe { event.modifierFlags() });
                self.dispatch_event(WindowEvent::Mouse(
                    MouseEvent::Moved { position, modifiers }
                ))
            }
        }

        #[unsafe(method(mouseDragged:))]
        fn mouse_dragged(&self, event: &NSEvent) {
            if let Some(position) = self.mouse_position(event) {
                let modifiers = get_modifiers(unsafe { event.modifierFlags() });
                self.dispatch_event(WindowEvent::Mouse(
                    MouseEvent::Moved { position, modifiers }
                ))
            }
        }

        #[unsafe(method(scrollWheel:))]
        fn scroll_wheel(&self, event: &NSEvent) {
            if let Some(position) = self.mouse_position(event) {
                let modifiers = get_modifiers(unsafe { event.modifierFlags() });
                // These are reported in pixels. To get it consistent with Windows,
                // convert to fractional lines (divide by 15 px). Can we do this better?
                let delta_x = unsafe { event.scrollingDeltaX() };
                let delta_y = unsafe { event.scrollingDeltaY() };
                self.dispatch_event(WindowEvent::Mouse(
                    MouseEvent::Wheel {
                        delta: Vec2 { x: delta_x / 15.0, y: delta_y / 15.0 },
                        position,
                        modifiers
                    }
                ))
            }
        }

        #[unsafe(method(acceptsFirstResponder))]
        fn accepts_first_responder(&self) -> bool {
            true
        }

        #[unsafe(method(viewDidMoveToWindow))]
        fn view_did_move_to_window(&self) {
            let visible_rect = self.visibleRect();
            self.set_tracking_area(visible_rect);
        }

        #[unsafe(method(updateTrackingAreas))]
        fn update_tracking_areas(&self) {
            let visible_rect = self.visibleRect();
            self.set_tracking_area(visible_rect);
        }

        #[unsafe(method(frameDidChange:))]
        fn frame_did_change(&self, _event: &NSEvent) {
            let visible_rect = self.visibleRect();
            let new_size = visible_rect.size.into();
            self.dispatch_event(WindowEvent::Resize { new_size });
        }

        #[unsafe(method(acceptsFirstMouse:))]
        fn accepts_first_mouse(&self, _event: &NSEvent) -> bool {
            true
        }

        #[unsafe(method(drawRect:))]
        fn draw_rect(&self, rect: NSRect) {
            let graphics_context = unsafe { NSGraphicsContext::currentContext() }.unwrap();
            let context = unsafe { graphics_context.CGContext() };
            let ci_context = self.ivars().ci_context.get_or_init(|| unsafe {
                let metal_device = MTLCreateSystemDefaultDevice().unwrap();
                CIContext::contextWithMTLDevice(&metal_device)
            });

            let renderer = RendererRef::new(&context, &ci_context, rect);

            self.ivars().handler.borrow_mut().render(renderer);
        }

        #[unsafe(method(onAnimationTimer:))]
        fn on_animation_timer(&self, _timer: &NSTimer) {
            let animation_start = self.ivars().animation_start.get_or_init(|| {
                unsafe { NSDate::now() }
            });

            let animation_frame = AnimationFrame {
                timestamp: -unsafe { animation_start.timeIntervalSinceNow() }
            };
            self.dispatch_event(WindowEvent::Animation(animation_frame));
        }
    }

    unsafe impl NSObjectProtocol for View {}
);

impl View {
    pub(crate) fn new(
        mtm: MainThreadMarker,
        handler: impl WindowHandler + 'static,
        frame: Option<NSRect>,
    ) -> Retained<Self> {
        let handler = RefCell::new(Box::new(handler));
        let tracking_area = RefCell::new(None);

        let this = mtm.alloc();
        let this = this.set_ivars(Ivars {
            handler,
            tracking_area,
            timer: RefCell::new(None),
            animation_start: OnceCell::new(),
            ci_context: OnceCell::new(),
        });

        let this: Retained<Self> = if let Some(frame) = frame {
            unsafe { msg_send![super(this), initWithFrame: frame] }
        } else {
            unsafe { msg_send![super(this), init] }
        };

        this.setPostsFrameChangedNotifications(true);
        let notification_center = unsafe { NSNotificationCenter::defaultCenter() };
        unsafe {
            notification_center.addObserver_selector_name_object(
                &this,
                sel!(frameDidChange:),
                Some(NSViewFrameDidChangeNotification),
                Some(&this),
            )
        }

        this.ivars()
            .handler
            .borrow_mut()
            .init(Handle::new(Weak::from_retained(&this)));

        // Initialize animation timer
        let timer = unsafe {
            NSTimer::scheduledTimerWithTimeInterval_target_selector_userInfo_repeats(
                1.0 / 60.0,
                &this,
                sel!(onAnimationTimer:),
                None,
                true,
            )
        };

        this.ivars().timer.replace(Some(timer));
        this
    }

    fn dispatch_event(&self, event: WindowEvent) {
        self.ivars().handler.borrow_mut().event(event)
    }

    fn set_tracking_area(&self, rect: NSRect) {
        // Use try-borrow here to avoid re-entrancy problems
        if let Ok(mut tracking_area_ref) = self.ivars().tracking_area.try_borrow_mut() {
            if let Some(tracking_area) = tracking_area_ref.as_ref() {
                unsafe { self.removeTrackingArea(tracking_area) };
                *tracking_area_ref = None;
            }

            let tracking_area = unsafe {
                let tracking_area = NSTrackingArea::alloc();
                NSTrackingArea::initWithRect_options_owner_userInfo(
                    tracking_area,
                    rect,
                    NSTrackingAreaOptions::ActiveAlways
                        | NSTrackingAreaOptions::MouseEnteredAndExited
                        | NSTrackingAreaOptions::MouseMoved,
                    Some(self),
                    None,
                )
            };

            let tracking_area = tracking_area_ref.insert(tracking_area);
            unsafe { self.addTrackingArea(tracking_area) };
        }
    }

    fn mouse_position(&self, event: &NSEvent) -> Option<Point> {
        let pos = unsafe { event.locationInWindow() };
        let frame = self.frame();
        let pos = self.convertPoint_fromView(pos, None);

        if pos.x.is_sign_negative()
            || pos.y.is_sign_negative()
            || pos.x > frame.size.width
            || pos.y > frame.size.height
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
        _ => None,
    }
}
