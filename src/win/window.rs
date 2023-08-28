use std::{sync::Once, cell::RefCell, marker::PhantomData, rc::Rc};

use windows::{core::{PCWSTR, w, Result, Error}, 
    Win32::{
        Foundation::*, 
        System::LibraryLoader::GetModuleHandleW, 
        UI::WindowsAndMessaging::*, 
        Graphics::Direct2D,
        Graphics::DirectWrite,
        Graphics::Gdi}};

use super::{com, Renderer};
use crate::{core::{Color, Point}, Event, widget::Widget, event::{MouseEvent, WindowEvent}};
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

#[derive(Debug, PartialEq, Copy, Clone)]
struct PhysPoint {
    x: i32,
    y: i32
}

impl PhysPoint {
    pub fn from_lparam(lparam: LPARAM) -> Self {
        Self { x: loword(lparam) as i32, y: hiword(lparam) as i32 }    
    }
}

pub struct Window {
    handle: HWND,
    // Ensure handle is !Send
    _phantom: PhantomData<*mut ()>,
}

struct WindowState {
    d2d_factory: Direct2D::ID2D1Factory,
    dw_factory: DirectWrite::IDWriteFactory,
    renderer: RefCell<Option<Renderer>>,
    widget: RefCell<Box<dyn Widget>>,
    last_mouse_pos: RefCell<Option<PhysPoint>>
}

impl WindowState {
    fn handle_message(&self, hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        match message {
            WM_PAINT => {   
                let mut renderer_ref = self.renderer.borrow_mut();
                let renderer = renderer_ref.get_or_insert_with(|| {
                    Renderer::new(hwnd, &self.d2d_factory).unwrap()
                });
    
                unsafe { 
                    let mut ps = Gdi::PAINTSTRUCT::default();
                    Gdi::BeginPaint(hwnd, &mut ps);
    
                    renderer.begin_draw();
                    renderer.clear(Color::BLUE);
    
                    self.widget.borrow().render(&mut crate::window::Renderer(renderer));
    
                    // TODO: Handle error here
                    renderer.end_draw().unwrap();
    
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
                self.widget.borrow_mut().event(Event::Window(window_event));
                Some(LRESULT(0))
            },

            WM_SETFOCUS => {
                self.widget.borrow_mut().event(Event::Window(WindowEvent::Focused));
                Some(LRESULT(0))
            },

            WM_KILLFOCUS => {
                self.widget.borrow_mut().event(Event::Window(WindowEvent::Unfocused));
                Some(LRESULT(0))
            },
            
            WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN | 
            WM_LBUTTONUP | WM_RBUTTONUP | WM_MBUTTONUP | 
            WM_LBUTTONDBLCLK | WM_RBUTTONDBLCLK | WM_MBUTTONDBLCLK => {
                let mouse_event = self.get_mouse_event(message, wparam, lparam);

                self.widget.borrow_mut().event(Event::Mouse(mouse_event));
                Some(LRESULT(0))
            },

            WM_MOUSEMOVE => {
                let phys_pos = PhysPoint::from_lparam(lparam);
                let position: Point = [phys_pos.x, phys_pos.y].into();
                let last_mouse_pos = self.last_mouse_pos.replace(Some(phys_pos));

                if let Some(last_mouse_pos) = last_mouse_pos {
                    // Filter out spurious mouse move events
                    if phys_pos != last_mouse_pos {
                        self.widget.borrow_mut().event(Event::Mouse(MouseEvent::Moved { position }));
                    }
                } else {
                    self.widget.borrow_mut().event(Event::Mouse(MouseEvent::Enter));
                    self.widget.borrow_mut().event(Event::Mouse(MouseEvent::Moved { position }));
                }
                
                Some(LRESULT(0))
            },

            0x02A3 /* WM_MOUSELEAVE */ => {
                self.last_mouse_pos.replace(None);
                self.widget.borrow_mut().event(Event::Mouse(MouseEvent::Exit));
                Some(LRESULT(0))
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
    pub fn new(widget: impl Widget + 'static) -> Result<Self> {
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
            d2d_factory: unsafe { Direct2D::D2D1CreateFactory::<Direct2D::ID2D1Factory>(Direct2D::D2D1_FACTORY_TYPE_SINGLE_THREADED, None)? },
            dw_factory: unsafe { DirectWrite::DWriteCreateFactory(DirectWrite::DWRITE_FACTORY_TYPE_SHARED)? },
            renderer: RefCell::new(None),
            widget: RefCell::new(Box::new(widget)),
            last_mouse_pos: RefCell::new(None),
        });

        let hwnd = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(), 
                WINDOW_CLASS, 
                w!("My window"), 
                WS_OVERLAPPEDWINDOW, 
                CW_USEDEFAULT, 
                CW_USEDEFAULT, 
                CW_USEDEFAULT, 
                CW_USEDEFAULT, 
                None, 
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

unsafe extern "system" fn wndproc(hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if message == WM_NCCREATE {
        let create_struct = &*(lparam.0 as *const CREATESTRUCTW);
        let window_state_ptr = create_struct.lpCreateParams;
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, window_state_ptr as _);
    }
    let window_state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const WindowState;

    let result = if !window_state_ptr.is_null() {
        (&*window_state_ptr).handle_message(hwnd, message, wparam, lparam)
    } else {
        None
    };

    result.unwrap_or_else(|| {
        DefWindowProcW(hwnd, message, wparam, lparam)
    })
}