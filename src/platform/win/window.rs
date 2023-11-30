use std::{sync::Once, cell::RefCell, marker::PhantomData, rc::Rc};

use windows::{core::{PCWSTR, w, Result, Error}, 
    Win32::{
        Foundation::*, 
        System::LibraryLoader::GetModuleHandleW, 
        UI::{WindowsAndMessaging::*, Input::KeyboardAndMouse::TrackMouseEvent}, 
        UI::Input::{*, KeyboardAndMouse::VIRTUAL_KEY},
        Graphics::Gdi::{self, InvalidateRect}}};

use super::{com, Renderer, keyboard::{vk_to_key, get_modifiers}, Handle};
use crate::{core::{Color, Point, Rectangle}, Event, event::{MouseEvent, WindowEvent, KeyEvent}, window::WindowHandler};
use crate::event::MouseButton;

const WINDOW_CLASS: PCWSTR = w!("my_window");
static REGISTER_WINDOW_CLASS: Once = Once::new();

pub trait CheckOk {
    type Output: Sized;
    fn ok(self) -> Result<Self::Output>;
}

impl CheckOk for HWND {
    type Output = Self;
    fn ok(self) -> Result<Self::Output> {
        if self.0 == 0 {
            Err(Error::from_win32())
        } else {
            Ok(self)
        }
    }
}

impl CheckOk for BOOL {
    type Output = ();
    fn ok(self) -> Result<Self::Output> {
        if self == BOOL(0) {
            Err(Error::from_win32())
        } else {
            Ok(())
        }
    }
}

pub struct Window {
    handle: HWND,
    // Ensure handle is !Send
    _phantom: PhantomData<*mut ()>,
}

struct WindowState {
    renderer: RefCell<Option<Renderer>>,
    handler: RefCell<Box<dyn WindowHandler>>,
    last_mouse_pos: RefCell<Option<Point<i32>>>
}

impl WindowState {
    fn publish_event(&self, hwnd: HWND, event: Event) {
        let mut handle = Handle::new(hwnd);
        self.handler.borrow_mut().event(event, &mut handle);
    }

    fn handle_message(&self, hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        match message {
            WM_PAINT => {   
                let mut renderer_ref = self.renderer.borrow_mut();
                let renderer = renderer_ref.get_or_insert_with(|| {
                    Renderer::new(hwnd).unwrap()
                });
    
                let mut ps = Gdi::PAINTSTRUCT::default();
                unsafe { 
                    Gdi::BeginPaint(hwnd, &mut ps);
    
                    renderer.begin_draw();
                    renderer.clear(Color::WHITE);
                }
    
                {
                    let rect: Rectangle = super::util::get_client_rect(hwnd).into();
                    self.handler.borrow_mut()
                        .render(Rectangle::new(Point::ZERO, rect.size()), renderer);
                }
    
                unsafe {
                    if let Err(error) = renderer.end_draw() {
                        if error.code() == D2DERR_RECREATE_TARGET {
                            // Set renderer to None to force rebuild
                        }
                    }
    
                    Gdi::EndPaint(hwnd, &mut ps).ok().unwrap();
                }
    
                Some(LRESULT(0))
            },

            WM_SIZE => {
                let width = loword(lparam) as u32;
                let height = hiword(lparam) as u32;

                if let Some(renderer) = self.renderer.borrow_mut().as_ref() {    
                    renderer.resize(width, height).unwrap();
                }

                let window_event = WindowEvent::Resize { new_size: [width, height].into() };
                self.publish_event(hwnd, Event::Window(window_event));
                unsafe { InvalidateRect(hwnd, None, false) };
                Some(LRESULT(0))
            },

            WM_SETFOCUS => {
                self.publish_event(hwnd, Event::Window(WindowEvent::Focused));
                Some(LRESULT(0))
            },

            WM_KILLFOCUS => {
                self.publish_event(hwnd, Event::Window(WindowEvent::Unfocused));
                Some(LRESULT(0))
            },
            
            WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN | 
            WM_LBUTTONUP | WM_RBUTTONUP | WM_MBUTTONUP | 
            WM_LBUTTONDBLCLK | WM_RBUTTONDBLCLK | WM_MBUTTONDBLCLK => {
                let mouse_event = self.get_mouse_event(message, wparam, lparam);

                self.publish_event(hwnd, Event::Mouse(mouse_event));
                Some(LRESULT(0))
            },

            WM_MOUSEMOVE => {
                let phys_pos = point_from_lparam(lparam);
                let position: Point = phys_pos.into();
                let last_mouse_pos = self.last_mouse_pos.replace(Some(phys_pos));

                if let Some(last_mouse_pos) = last_mouse_pos {
                    // Filter out spurious mouse move events
                    if phys_pos != last_mouse_pos {
                        self.publish_event(hwnd, Event::Mouse(MouseEvent::Moved { position }));
                    }
                } else {
                    unsafe { 
                        // Setup tracking so that we get notified when the mouse leaves the client area
                        let mut ev = KeyboardAndMouse::TRACKMOUSEEVENT {
                            cbSize: std::mem::size_of::<KeyboardAndMouse::TRACKMOUSEEVENT>() as u32,
                            dwFlags: KeyboardAndMouse::TME_LEAVE,
                            hwndTrack: hwnd,
                            dwHoverTime: 0,
                        };
                        TrackMouseEvent(&mut ev);
                    };
                    self.publish_event(hwnd, Event::Mouse(MouseEvent::Enter));
                    self.publish_event(hwnd, Event::Mouse(MouseEvent::Moved { position }));
                }
                Some(LRESULT(0))
            },

            0x02A3 /* WM_MOUSELEAVE */ => {
                self.last_mouse_pos.replace(None);
                self.publish_event(hwnd, Event::Mouse(MouseEvent::Exit));
                Some(LRESULT(0))
            },

            WM_KEYDOWN => {
                let key = vk_to_key(VIRTUAL_KEY(wparam.0 as u16));
                let modifiers = get_modifiers();
                self.publish_event(hwnd, Event::Keyboard(KeyEvent::KeyDown { key, modifiers }));
                Some(LRESULT(0))
            },

            WM_CHAR => {
                match wparam.0 as u16 {
                    0x08 | 0x0A | 0x1B  => { // backspace, linefeed, escape
                        None
                    },
                    str => {
                        if let Ok(str) = String::from_utf16(&[str]) {
                            self.publish_event(hwnd, Event::Keyboard(KeyEvent::Characters { str }));
                            Some(LRESULT(0))
                        } else {
                            None
                        }
                    }
                }
            },

            WM_DESTROY => {
                unsafe { PostQuitMessage(0) };
                Some(LRESULT(0))
            },
            _ => None
        }
    }

    fn get_mouse_event(&self, message: u32, _: WPARAM, lparam: LPARAM) -> MouseEvent {
        let button = match message {
            WM_LBUTTONDOWN | WM_LBUTTONUP | WM_LBUTTONDBLCLK => MouseButton::LEFT,
            WM_RBUTTONDOWN | WM_RBUTTONUP | WM_RBUTTONDBLCLK => MouseButton::RIGHT,
            WM_MBUTTONDOWN | WM_MBUTTONUP | WM_MBUTTONDBLCLK => MouseButton::MIDDLE,
            _ => unreachable!()
        };

        let position: Point = [loword(lparam) as i16, hiword(lparam) as i16].into();

        match message {
            WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN => 
                MouseEvent::Down { button, position },
            WM_LBUTTONUP | WM_RBUTTONUP | WM_MBUTTONUP =>
                MouseEvent::Up { button, position },
            WM_LBUTTONDBLCLK | WM_RBUTTONDBLCLK | WM_MBUTTONDBLCLK =>
                MouseEvent::DoubleClick { button, position },
            _ => unreachable!()
        }
    }
}

impl Window {
    pub fn open(handler: impl WindowHandler + 'static) -> Result<Self> {
        Self::create(None, WS_OVERLAPPEDWINDOW, handler)
    }

    pub fn attach(parent: HWND, handler: impl WindowHandler + 'static) -> Result<Self> {
        Self::create(Some(parent), WS_CHILD, handler)
    }

    pub fn set_size(&self, size: Rectangle<i32>) -> Result<()> {
        unsafe { SetWindowPos(
            self.handle, 
            None, 
            size.left(), 
            size.top(), 
            size.width(), 
            size.height(), 
            SET_WINDOW_POS_FLAGS::default()).ok()
        }
    }

    fn create(parent: Option<HWND>, style: WINDOW_STYLE, handler: impl WindowHandler + 'static) -> Result<Self> {
        let instance = unsafe { GetModuleHandleW(None)? };
        REGISTER_WINDOW_CLASS.call_once(|| {
            let class = WNDCLASSW {
                lpszClassName: WINDOW_CLASS,
                hCursor: unsafe { LoadCursorW(None, IDC_ARROW).ok().unwrap() },
                hInstance: instance,
                lpfnWndProc: Some(wndproc),
                style: CS_HREDRAW | CS_VREDRAW | CS_DBLCLKS,
                ..WNDCLASSW::default()
            };
            assert_ne!(unsafe { RegisterClassW(&class) }, 0, "Unable to register window class");
        });

        com::com_initialized();

        let window_state = Rc::new(WindowState {
            renderer: RefCell::new(None),
            handler: RefCell::new(Box::new(handler)),
            last_mouse_pos: RefCell::new(None),
        });

        let hwnd = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(), 
                WINDOW_CLASS, 
                w!("My window"), 
                style, 
                CW_USEDEFAULT, 
                CW_USEDEFAULT, 
                CW_USEDEFAULT, 
                CW_USEDEFAULT, 
                parent.unwrap_or(HWND(0)), 
                None, 
                instance, 
                Some(Rc::into_raw(window_state) as _)).ok()?
        };

        let result = Window {
            handle: hwnd,
            _phantom: PhantomData
        };

        unsafe { ShowWindow(hwnd, SW_SHOW) };

        Ok(result)
    }
}

fn loword(lparam: LPARAM) -> u16 {
    (lparam.0 & 0xFFFF) as u16
}

fn hiword(lparam: LPARAM) -> u16 {
    ((lparam.0 >> 16) & 0xFFFF) as u16
}

fn point_from_lparam(lparam: LPARAM) -> Point<i32> {
    Point::new(loword(lparam) as i32, hiword(lparam) as i32)
}

unsafe extern "system" fn wndproc(hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if message == WM_NCCREATE {
        let create_struct = &*(lparam.0 as *const CREATESTRUCTW);
        let window_state_ptr = create_struct.lpCreateParams;
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, window_state_ptr as _);
    }

    let window_state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const WindowState;
    if !window_state_ptr.is_null() {
        if message == WM_NCDESTROY {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            drop(Rc::from_raw(window_state_ptr));
            DefWindowProcW(hwnd, message, wparam, lparam)
        } else {
            let result = (&*window_state_ptr).handle_message(hwnd, message, wparam, lparam);
            result.unwrap_or_else(|| {
                DefWindowProcW(hwnd, message, wparam, lparam)
            })
        }
    } else {
        DefWindowProcW(hwnd, message, wparam, lparam)
    }
}