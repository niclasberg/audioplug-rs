use std::cell::RefCell;
use std::rc::Rc;

use pollster::FutureExt;
use raw_window_handle::RawWindowHandle;

use super::View;
use crate::App;
use crate::core::{Cursor, Point, Rect};
use crate::platform::{self, WindowEvent};
use crate::ui::paint::{WGPUSurface, paint_window};
use crate::ui::{AppState, WindowId, handle_window_event};

struct PreInit<F>(F);
struct Constructed(WindowId);

enum WindowState<V> {
    PreInit(PreInit<V>),
    Initializing,
    Initialized(Constructed),
}

impl<V: View> WindowState<V> {
    fn window_id(&self) -> WindowId {
        match self {
            WindowState::PreInit(_) | WindowState::Initializing => {
                panic!("Root widget requested before window was initialized")
            }
            WindowState::Initialized(Constructed(root_widget)) => *root_widget,
        }
    }

    fn initialize(self, app_state: &mut AppState, handle: platform::Handle) -> Self {
        match self {
            WindowState::PreInit(PreInit(view)) => {
                // It would be neat to run this on the executor
                let surface = WGPUSurface::new(&handle)
                    .block_on()
                    .expect("Graphics initialization failed");

                let window_id = app_state.add_window(handle, surface, view);
                Self::Initialized(Constructed(window_id))
            }
            WindowState::Initialized(_) | WindowState::Initializing => {
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
        let window_state = WindowState::PreInit(PreInit(view));

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
        handle_window_event(&mut app_state, self.state.window_id(), event)
    }

    fn paint(&mut self, dirty_rect: Rect) {
        let mut app_state = self.app_state.borrow_mut();
        paint_window(&mut app_state, self.state.window_id(), dirty_rect)
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
            WindowState::PreInit(_) | WindowState::Initializing => {}
            WindowState::Initialized(Constructed(window_id)) => {
                let mut app_state = self.app_state.borrow_mut();
                app_state.remove_window(window_id);
            }
        }
    }
}

pub struct Window(platform::Window);

impl Window {
    pub fn open<V: View + 'static>(app: &mut App, view: V) -> Self {
        let handler = MyHandler::new(app.state.clone(), view);
        Self(platform::Window::open(handler).unwrap())
    }

    pub fn attach<V: View + 'static>(
        app_state: Rc<RefCell<AppState>>,
        parent_handle: RawWindowHandle,
        view: V,
    ) -> Self {
        let handler = MyHandler::new(app_state, view);
        let window: Result<platform::Window, platform::Error> = match parent_handle {
            #[cfg(target_os = "windows")]
            RawWindowHandle::Win32(handle) => {
                let hwnd = handle.hwnd.get() as *mut std::ffi::c_void;
                platform::Window::attach(windows::Win32::Foundation::HWND(hwnd), handler)
            }
            #[cfg(target_os = "macos")]
            RawWindowHandle::AppKit(handle) => platform::Window::attach(handle, handler),
            _ => panic!("Unsupported window type"),
        };

        Self(window.unwrap())
    }

    pub fn set_size(&self, size: Rect<i32>) {
        self.0.set_size(size).unwrap()
    }

    pub fn set_scale_factor(&self, scale_factor: f32) {
        self.0.set_scale_factor(scale_factor);
    }
}
