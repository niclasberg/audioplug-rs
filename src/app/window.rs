use std::cell::RefCell;
use std::rc::Rc;

use raw_window_handle::RawWindowHandle;

use crate::app::{handle_window_event, render_window, AppState, WindowId};
use crate::core::{Cursor, Point, Rectangle};
use crate::platform::{WindowEvent, WindowHandler};
use crate::view::View;
use crate::{platform, App};
use super::ViewContext;

struct PreInit<F>(F);
struct Constructed(WindowId);

enum WindowState<F> {
    PreInit(PreInit<F>),
	Initializing,
    Initialized(Constructed)
}

impl<F, V> WindowState<F>
where
	F: FnOnce(&mut ViewContext) -> V,
	V: View,
{
    fn window_id(&self) -> WindowId {
        match self {
            WindowState::PreInit(_) | WindowState::Initializing => panic!("Root widget requested before window was initialized"),
            WindowState::Initialized(Constructed(root_widget)) => *root_widget,
        }
    }

    fn initialize(self, app_state: &mut AppState, handle: platform::Handle) -> Self {
        match self {
            WindowState::PreInit(PreInit(view_factory)) => {
                let view = view_factory(&mut ViewContext::new(app_state));
				let window_id = app_state.add_window(handle, view);
                Self::Initialized(Constructed(window_id))
            }
            WindowState::Initialized(_) | WindowState::Initializing => panic!("Multiple calls to window initialize"),
        }
    }
}

pub(crate) struct MyHandler<F> {
    state: WindowState<F>,
    app_state: Rc<RefCell<AppState>>,
}

impl<F, V> MyHandler<F>
where 
	F: FnOnce(&mut ViewContext) -> V,
	V: View,
{
    pub fn new(app_state: Rc<RefCell<AppState>>, view_factory: F) -> Self{
        let window_state = WindowState::PreInit(PreInit(view_factory));

        Self {
            state: window_state,
            app_state,
        }
    }
}

impl<F, V> WindowHandler for MyHandler<F>
where 
	F: FnOnce(&mut ViewContext) -> V,
	V: View,
{
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

    fn render(&mut self, _: Rectangle, mut renderer: platform::RendererRef<'_>) {
        let mut app_state = self.app_state.borrow_mut();
        render_window(&mut app_state, self.state.window_id(), &mut renderer)
    }

    fn get_cursor(&self, _point: Point) -> Option<Cursor> {
        let cursor = None;
        /*self.widget_node
            .for_each_view_at(point, &mut |widget_node| {
                if let Some(c) = widget_node.widget.cursor() {
                    cursor = Some(c);
                    false
                } else {
                    true
                }
            });*/

        cursor
    }
}

impl<F> Drop for MyHandler<F> {
    fn drop(&mut self) {
        match self.state {
            WindowState::PreInit(_) | WindowState::Initializing => {},
            WindowState::Initialized(Constructed(window_id)) => {
                let mut app_state = self.app_state.borrow_mut();
                app_state.remove_window(window_id);
            }
        }
    }
}

pub struct Window(platform::Window);

impl Window {
    pub fn open<F, V>(app: &mut App, view_factory: F) -> Self
    where
        F: FnOnce(&mut ViewContext) -> V + 'static,
        V: View,
    {
        let handler = MyHandler::new(app.state.clone(), view_factory);
        Self(platform::Window::open(handler).unwrap())
    }

    pub fn attach<F, V>(
        app_state: Rc<RefCell<AppState>>,
        handle: RawWindowHandle,
        view_factory: F,
    ) -> Self
    where
        F: FnOnce(&mut ViewContext) -> V + 'static,
        V: View,
    {
        let handler = MyHandler::new(app_state, view_factory);
        let window: Result<platform::Window, platform::Error> = match handle {
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

    pub fn set_size(&self, size: Rectangle<i32>) {
        self.0.set_size(size).unwrap()
    }
}
