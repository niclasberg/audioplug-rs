use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

use raw_window_handle::RawWindowHandle;

use crate::app::{handle_window_event, invalidate_window, layout_window, render_window, AppState, Signal, SignalContext, WindowId};
use crate::core::{Cursor, Point, Rectangle};
use crate::platform::{WindowEvent, WindowHandler};
use crate::view::View;
use crate::{platform, App};

struct PreInit(Box<dyn FnOnce(&mut AppState, platform::Handle) -> Constructed>);
struct Constructed(WindowId);

pub enum WindowState {
    PreInit(PreInit),
    Initialized(Constructed)
}

impl WindowState {
    fn window_id(&self) -> WindowId {
        match self {
            WindowState::PreInit(_) => panic!("Root widget requested before window was initialized"),
            WindowState::Initialized(Constructed(root_widget)) => *root_widget,
        }
    }

    fn initialize(self, app_state: &mut AppState, handle: platform::Handle) -> Self {
        match self {
            WindowState::PreInit(PreInit(initializer)) => {
                Self::Initialized(initializer(app_state, handle))
            }
            WindowState::Initialized(_) => panic!("Multiple calls to window initialize"),
        }
    }
}

pub(crate) struct MyHandler {
    state: WindowState,
    app_state: Rc<RefCell<AppState>>,
}

impl MyHandler {
    pub fn new<F, V>(app_state: Rc<RefCell<AppState>>, view_factory: F) -> Self
    where
        F: FnOnce(&mut AppContext) -> V + 'static,
        V: View,
    {
        let window_state = WindowState::PreInit(PreInit(Box::new(|app_state, handle| -> Constructed {
            let window_id = {
                app_state.add_window(handle, move |mut build_ctx| {
                    let view = {
                        let mut app_context = AppContext {
                            app_state: &mut build_ctx.app_state,
                        };
                        view_factory(&mut app_context)
                    };
        
                    view.build(&mut build_ctx)
                })
            };
            Constructed(window_id)
        })));

        Self {
            state: window_state,
            app_state,
        }
    }

    fn with_app_state(&self, f: impl FnOnce(&AppState)) {
        let app_state = self.app_state.borrow();
        f(&app_state);
    }

    fn with_app_state_mut(&mut self, f: impl FnOnce(&mut AppState)) {
        let mut app_state = self.app_state.borrow_mut();
        f(&mut app_state);
    }
}

impl WindowHandler for MyHandler {
    fn init(&mut self, handle: platform::Handle) {
        let mut app_state = self.app_state.borrow_mut();
        self.state = self.state.initialize(&mut app_state, handle);
        layout_window(&mut app_state, self.state.window_id());
    }

    fn event(&mut self, event: WindowEvent, _handle: platform::HandleRef) {
        let mut app_state = self.app_state.borrow_mut();
        handle_window_event(&mut app_state, self.state.window_id(), event)
    }

    fn render(&mut self, _: Rectangle, mut renderer: platform::RendererRef<'_>) {
        let mut app_state = self.app_state.borrow_mut();
        render_window(&mut app_state, self.state.window_id(), &mut renderer)
    }

    fn get_cursor(&self, point: Point) -> Option<Cursor> {
        let mut cursor = None;
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

impl Drop for MyHandler {
    fn drop(&mut self) {
        match self.state {
            WindowState::PreInit(_) => {},
            WindowState::Initialized(Constructed(window_id)) => {
                let mut app_state = self.app_state.borrow_mut();
                app_state.remove_window(window_id);
            }
        }
    }
}

pub struct AppContext<'a> {
    app_state: &'a mut AppState,
}

impl<'a> AppContext<'a> {
    pub fn create_signal<T: Any>(&mut self, value: T) -> Signal<T> {
        self.app_state.create_signal(value)
    }

    pub fn create_effect(&mut self, f: impl Fn(&mut AppState) + 'static) {
        self.app_state.create_effect(f)
    }
}

impl<'b> SignalContext for AppContext<'b> {
    fn get_signal_value_ref_untracked<'a, T: Any>(&'a self, signal: &Signal<T>) -> &'a T {
        self.app_state.get_signal_value_ref_untracked(signal)
    }

    fn get_signal_value_ref<'a, T: Any>(&'a mut self, signal: &Signal<T>) -> &'a T {
        self.app_state.get_signal_value_ref(signal)
    }

    fn set_signal_value<T: Any>(&mut self, signal: &Signal<T>, value: T) {
        self.app_state.set_signal_value(signal, value)
    }
}

pub struct Window(platform::Window);

impl Window {
    pub fn open<F, V>(app: &mut App, view_factory: F) -> Self
    where
        F: FnOnce(&mut AppContext) -> V,
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
        F: FnOnce(&mut AppContext) -> V,
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
