use std::cell::RefCell;
use std::rc::Rc;

use pollster::FutureExt;
use raw_window_handle::RawWindowHandle;

use super::{AppState, View, WindowId};
use crate::App;
use crate::core::{Cursor, PhysicalRect, Point, Rect, ScaleFactor};
use crate::platform::{self, WindowEvent};
use crate::ui::render::WGPUSurface;

enum WindowState<V> {
    PreInit { view: V },
    Initializing,
    Initialized { window_id: WindowId },
}

impl<V: View> WindowState<V> {
    fn window_id(&self) -> WindowId {
        match self {
            WindowState::PreInit { .. } | WindowState::Initializing => {
                panic!("Root widget requested before window was initialized")
            }
            WindowState::Initialized { window_id } => *window_id,
        }
    }

    fn initialize(self, app_state: &mut AppState, handle: platform::Handle) -> Self {
        match self {
            WindowState::PreInit { view } => {
                // It would be neat to run this on the executor
                let surface = WGPUSurface::new(&handle)
                    .block_on()
                    .expect("Graphics initialization failed");

                let window_id = app_state.add_window(handle, surface, view);
                Self::Initialized { window_id }
            }
            WindowState::Initialized { .. } | WindowState::Initializing => {
                panic!("Multiple calls to window initialize")
            }
        }
    }
}

pub(crate) struct MyHandler<F> {
    state: WindowState<F>,
    app_state: Rc<RefCell<AppState>>,
}

impl<V: View> MyHandler<V> {
    pub fn new(app_state: Rc<RefCell<AppState>>, view: V) -> Self {
        let window_state = WindowState::PreInit { view };

        Self {
            state: window_state,
            app_state,
        }
    }
}

impl<V: View> platform::WindowHandler for MyHandler<V> {
    fn init(&mut self, handle: platform::Handle) {
        let mut app_state = self.app_state.borrow_mut();

        let state = std::mem::replace(&mut self.state, WindowState::Initializing);
        let state = state.initialize(&mut app_state, handle);
        let _ = std::mem::replace(&mut self.state, state);
    }

    fn event(&mut self, event: WindowEvent) {
        let mut app_state = self.app_state.borrow_mut();
        super::event_handling::handle_window_event(&mut app_state, self.state.window_id(), event)
    }

    fn paint(&mut self, dirty_rect: Rect) {
        let mut app_state = self.app_state.borrow_mut();
        super::render::paint_window(&mut app_state, self.state.window_id(), dirty_rect)
    }

    fn get_cursor(&self, pos: Point) -> Option<Cursor> {
        let app_state = self.app_state.borrow();
        let mut selected_cursor = None;
        app_state.for_each_widget_at_rev(self.state.window_id(), pos, |app_state, widget_id| {
            if let Some(cursor) = app_state.widget_data_ref(widget_id).style.cursor {
                selected_cursor = Some(cursor);
                false
            } else {
                true
            }
        });
        selected_cursor
    }
}

impl<F> Drop for MyHandler<F> {
    fn drop(&mut self) {
        match self.state {
            WindowState::PreInit { .. } | WindowState::Initializing => {}
            WindowState::Initialized { window_id } => {
                let mut app_state = self.app_state.borrow_mut();
                app_state.remove_window(window_id);
            }
        }
    }
}

pub struct Window(platform::Window);

impl Window {
    pub fn open<V: View + 'static>(app: &mut App, view: V) -> Self {
        let handler = Box::new(MyHandler::new(app.state.clone(), view));
        Self(platform::Window::open(handler).unwrap())
    }

    pub fn attach<V: View + 'static>(
        app_state: Rc<RefCell<AppState>>,
        parent_handle: RawWindowHandle,
        view: V,
    ) -> Self {
        let handler = Box::new(MyHandler::new(app_state, view));
        let window: Result<platform::Window, platform::Error> = match parent_handle {
            #[cfg(target_os = "windows")]
            RawWindowHandle::Win32(handle) => {
                let hwnd = handle.hwnd.get() as *mut std::ffi::c_void;
                platform::Window::attach(windows::Win32::Foundation::HWND(hwnd), handler)
            }
            #[cfg(target_os = "macos")]
            RawWindowHandle::AppKit(handle) => platform::Window::attach(handle, handler),
            #[cfg(target_os = "linux")]
            RawWindowHandle::Wayland(handle) => platform::Window::attach_wayland(handle, handler),
            #[cfg(target_os = "linux")]
            RawWindowHandle::Xcb(handle) => platform::Window::attach_xcb(handle, handler),
            _ => panic!("Unsupported window type"),
        };

        Self(window.unwrap())
    }

    pub fn set_physical_size(&self, size: PhysicalRect) {
        self.0.set_physical_size(size).unwrap()
    }

    pub fn set_logical_size(&self, size: Rect) {
        self.0.set_logical_size(size).unwrap()
    }

    pub fn set_scale_factor(&self, scale_factor: ScaleFactor) {
        self.0.set_scale_factor(scale_factor);
    }

    pub fn scale_factor(&self) -> ScaleFactor {
        self.0.scale_factor()
    }
}
