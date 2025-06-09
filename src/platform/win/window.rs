use std::{
    cell::{Cell, RefCell},
    marker::PhantomData,
    mem::MaybeUninit,
    rc::Rc,
    sync::Once,
};

use windows::{
    core::{w, Result, BOOL, PCWSTR},
    Win32::{
        Foundation::*,
        Graphics::{
            Dwm::{DwmSetWindowAttribute, DWMWA_USE_IMMERSIVE_DARK_MODE},
            Gdi::{self, InvalidateRect, ScreenToClient},
        },
        System::{LibraryLoader::GetModuleHandleW, Performance},
        UI::{
            Input::{
                KeyboardAndMouse::{TrackMouseEvent, VIRTUAL_KEY},
                *,
            },
            WindowsAndMessaging::*,
        },
    },
};
use KeyboardAndMouse::{ReleaseCapture, SetCapture};

use super::{
    com, cursors::get_cursor, keyboard::{get_modifiers, vk_to_key, KeyFlags}, renderer::RendererGeneration, util::{get_scale_factor_for_window, get_theme}, Handle, Renderer
};
use crate::event::MouseButton;
use crate::{
    core::{Color, Key, Point, Rectangle, Size, Vec2, WindowTheme},
    event::{AnimationFrame, KeyEvent, MouseEvent},
    platform::{WindowEvent, WindowHandler},
    MouseButtons,
};

const WINDOW_CLASS: PCWSTR = w!("my_window");
static REGISTER_WINDOW_CLASS: Once = Once::new();
const ANIMATION_FRAME_TIMER: usize = 10;

pub struct Window {
    handle: HWND,
    // Ensure handle is !Send
    _phantom: PhantomData<*mut ()>,
}

struct TmpKeyEvent {
    chars: Vec<u16>,
    key: Key,
}

struct WindowState {
    renderer: RefCell<Option<Renderer>>,
    renderer_generation: Cell<RendererGeneration>,
    handler: RefCell<Box<dyn WindowHandler>>,
    last_mouse_pos: RefCell<Option<Point<i32>>>,
    ticks_per_second: f64,
    current_key_event: RefCell<Option<TmpKeyEvent>>,
    scale_factor: Rc<Cell<f64>>,
    captured_mouse_buttons: RefCell<MouseButtons>,
    theme: Rc<Cell<WindowTheme>>,
    quit_app_on_exit: bool,
}

impl WindowState {
    fn publish_event(&self, _hwnd: HWND, event: WindowEvent) {
        self.handler.borrow_mut().event(event);
    }

    fn handle_message(
        &self,
        hwnd: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> Option<LRESULT> {
        match message {
            WM_CREATE => {
                self.scale_factor.replace(get_scale_factor_for_window(hwnd));

                {
                    self.handler.borrow_mut().init(Handle::new(hwnd, self.scale_factor.clone(), self.theme.clone()));
                }

                unsafe {
                    SetTimer(Some(hwnd), ANIMATION_FRAME_TIMER, 1000 / 60, None);
                    Some(LRESULT(0))
                }
            },

            WM_DESTROY => {
                unsafe {
                    KillTimer(Some(hwnd), ANIMATION_FRAME_TIMER).unwrap();
                    if self.quit_app_on_exit {
                        PostQuitMessage(0);
                    }
                };
                Some(LRESULT(0))
            },

            WM_PAINT => {
                let mut ps = Gdi::PAINTSTRUCT::default();
                unsafe {
                    Gdi::BeginPaint(hwnd, &mut ps);
                }
                
                let rect: Rectangle = super::util::get_client_rect(hwnd).into();
                self.render(hwnd, rect.size());

                unsafe {
                    Gdi::EndPaint(hwnd, &ps).ok().unwrap();
                }

                Some(LRESULT(0))
            },

            WM_SIZE => {
                let width = loword(lparam) as u32;
                let height = hiword(lparam) as u32;

                if let Some(renderer) = self.renderer.borrow_mut().as_ref() {
                    renderer.resize(width, height).unwrap();
                }

                let new_size: Size = [width, height].into();
                let new_size = new_size.scale(self.scale_factor.get());
                let window_event = WindowEvent::Resize { new_size };
                self.publish_event(hwnd, window_event);
                let _ = unsafe { InvalidateRect(Some(hwnd), None, false) };
                //self.render(hwnd, Size::new(width, height).into());
                Some(LRESULT(0))
            },

            WM_SETFOCUS => {
                self.publish_event(hwnd, WindowEvent::Focused);
                Some(LRESULT(0))
            },

            WM_KILLFOCUS => {
                self.publish_event(hwnd, WindowEvent::Unfocused);
                Some(LRESULT(0))
            },

            WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN |
            WM_LBUTTONUP | WM_RBUTTONUP | WM_MBUTTONUP |
            WM_LBUTTONDBLCLK | WM_RBUTTONDBLCLK | WM_MBUTTONDBLCLK => {
                let mouse_event = self.get_mouse_event(message, wparam, lparam);
                if let MouseEvent::Down { button, .. } = mouse_event {
                    if self.captured_mouse_buttons.borrow().is_empty() {
                        unsafe { SetCapture(hwnd) };
                    }
                    self.captured_mouse_buttons.borrow_mut().insert(button);
                } else if let MouseEvent::Up { button, .. } = mouse_event {
                    self.captured_mouse_buttons.borrow_mut().remove(button);
                    if self.captured_mouse_buttons.borrow().is_empty() {
                        drop(unsafe { ReleaseCapture() });
                    }
                }

                self.publish_event(hwnd, WindowEvent::Mouse(mouse_event));
                Some(LRESULT(0))
            },

            WM_CAPTURECHANGED => {
                if !self.captured_mouse_buttons.borrow().is_empty() {
                    self.captured_mouse_buttons.borrow_mut().clear();
                    self.publish_event(hwnd, WindowEvent::MouseCaptureEnded);
                }

                Some(LRESULT(0))
            },

            WM_MOUSEMOVE => {
                let phys_pos = point_from_lparam(lparam);
                let position = self.position_from_lparam(lparam);
                let last_mouse_pos = self.last_mouse_pos.replace(Some(phys_pos));

                if let Some(last_mouse_pos) = last_mouse_pos {
                    // Filter out spurious mouse move events
                    if phys_pos != last_mouse_pos {
                        self.publish_event(hwnd, WindowEvent::Mouse(MouseEvent::Moved { position, modifiers: get_modifiers() }));
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
                        TrackMouseEvent(&mut ev).unwrap();
                    };
                    self.publish_event(hwnd, WindowEvent::MouseEnter);
                    self.publish_event(hwnd, WindowEvent::Mouse(MouseEvent::Moved { position, modifiers: get_modifiers() }));
                }
                Some(LRESULT(0))
            },

            WM_MOUSEWHEEL | WM_MOUSEHWHEEL => {
                let lines = (wparam.0 >> 16) as i16;
                let lines = lines as f64 / WHEEL_DELTA as f64;
                let delta = if message == WM_MOUSEWHEEL {
                    Vec2::new(0.0, lines)
                } else {
                    Vec2::new(lines, 0.0)
                };

                let position = self.position_from_screen_lparam(hwnd, lparam);
                self.publish_event(hwnd, WindowEvent::Mouse(MouseEvent::Wheel {
                    delta,
                    position,
                    modifiers: get_modifiers()
                }));

                Some(LRESULT(0))
            },

            0x02A3 /* WM_MOUSELEAVE */ => {
                self.last_mouse_pos.replace(None);
                self.publish_event(hwnd, WindowEvent::MouseExit);
                Some(LRESULT(0))
            },

            WM_TIMER => {
                if wparam.0 == ANIMATION_FRAME_TIMER {
                    if let Some(timestamp) = self.current_timestamp() {
                        self.publish_event(hwnd, WindowEvent::Animation(AnimationFrame { timestamp }));
                    }
                };

                Some(LRESULT(0))
            },

            WM_KEYDOWN => {
                let key = vk_to_key(VIRTUAL_KEY(wparam.0 as u16));
                let modifiers = get_modifiers();
                let flags = KeyFlags::from_lparam(lparam);
                // If a keydown message can be translated into a character, a WM_CHAR message
                // will follow directly after the WM_KEYDOWN message (with the same scancode).
                // We collapse the KEYDOWN and CHAR message into the same KeyEvent (the WM_CHAR message
                // is used to construct the string representation)
                if has_wm_char_message(hwnd, flags) {
                    debug_assert!(self.current_key_event.borrow().is_none());
                    *(self.current_key_event.borrow_mut()) = Some(TmpKeyEvent { chars: Vec::new(), key });
                } else {
                    self.publish_event(hwnd, WindowEvent::Key(KeyEvent::KeyDown { key, modifiers, str: None, repeat_count: flags.repeat_count as usize }));
                }

                Some(LRESULT(0))
            },

            WM_CHAR => {
                let flags = KeyFlags::from_lparam(lparam);
                self.current_key_event.borrow_mut().as_mut().unwrap()
                    .chars.push(wparam.0 as u16);

                if !has_wm_char_message(hwnd, flags) {
                    let current_key_event = self.current_key_event.borrow_mut().take().unwrap();
                    let modifiers = get_modifiers();
                    let str = String::from_utf16(&current_key_event.chars).ok();
                    let key_event = KeyEvent::KeyDown {
                        key: current_key_event.key,
                        modifiers,
                        str,
                        repeat_count: flags.repeat_count as usize,
                    };

                    self.publish_event(hwnd, WindowEvent::Key(key_event));
                }
                Some(LRESULT(0))
            },

            WM_SETCURSOR => {
                let pos: Point = get_message_pos(hwnd).into();
                let pos = pos.scale(self.scale_factor.get());
                if let Some(cursor) = self.handler.borrow_mut().get_cursor(pos) {
                    unsafe { SetCursor(Some(get_cursor(cursor))) };
                    Some(LRESULT(0))
                } else {
                    None
                }
            },

            WM_DPICHANGED => {
                let scale_factor = get_scale_factor_for_window(hwnd);
                if let Some(renderer) = self.renderer.borrow_mut().as_mut() {
                    renderer.update_scale_factor(scale_factor as _).unwrap();
                }
                self.scale_factor.replace(scale_factor);
                self.publish_event(hwnd, WindowEvent::ScaleFactorChanged { scale_factor: self.scale_factor.get() });
                let _ = unsafe { InvalidateRect(Some(hwnd), None, false) };
                Some(LRESULT(0))
            },

            WM_SETTINGCHANGE => {
                let new_theme = get_theme();
                if new_theme != self.theme.get() {
                    self.theme.replace(new_theme);
                    self.publish_event(hwnd, WindowEvent::ThemeChanged(self.theme.get()));
                }
                None
            },

            _ => None
        }
    }

    fn render(&self, hwnd: HWND, size: Size) {
        let mut renderer_ref = self.renderer.borrow_mut();
        let renderer = renderer_ref.get_or_insert_with(|| {
            Renderer::new(hwnd).unwrap()
        });

        renderer.begin_draw();
        renderer.clear(Color::OFF_WHITE);

        self.handler.borrow_mut()
            .render(Rectangle::from_origin(Point::ZERO, size), renderer);
        
        if let Err(error) = renderer.end_draw() {
            // Clear the renderer so that it's recreated on next render
            if error.code() == D2DERR_RECREATE_TARGET {
                self.renderer.replace(None);
                self.renderer_generation.set(self.renderer_generation.get().next());
            }
        }
    }

    fn get_mouse_event(&self, message: u32, _: WPARAM, lparam: LPARAM) -> MouseEvent {
        let button = match message {
            WM_LBUTTONDOWN | WM_LBUTTONUP | WM_LBUTTONDBLCLK => MouseButton::LEFT,
            WM_RBUTTONDOWN | WM_RBUTTONUP | WM_RBUTTONDBLCLK => MouseButton::RIGHT,
            WM_MBUTTONDOWN | WM_MBUTTONUP | WM_MBUTTONDBLCLK => MouseButton::MIDDLE,
            _ => unreachable!(),
        };

        let position = self.position_from_lparam(lparam);
        let modifiers = get_modifiers();

        match message {
            WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN => MouseEvent::Down {
                button,
                position,
                modifiers,
            },
            WM_LBUTTONUP | WM_RBUTTONUP | WM_MBUTTONUP => MouseEvent::Up {
                button,
                position,
                modifiers,
            },
            WM_LBUTTONDBLCLK | WM_RBUTTONDBLCLK | WM_MBUTTONDBLCLK => MouseEvent::DoubleClick {
                button,
                position,
                modifiers,
            },
            _ => unreachable!(),
        }
    }

    fn current_timestamp(&self) -> Option<f64> {
        let mut lpperformancecount: i64 = 0;
        unsafe { Performance::QueryPerformanceCounter(&mut lpperformancecount as *mut _) }
            .map(|_| (lpperformancecount as f64) / self.ticks_per_second)
            .ok()
    }

    fn position_from_lparam(&self, lparam: LPARAM) -> Point {
        let position: Point = point_from_lparam(lparam).into();
        position.scale(1.0 / self.scale_factor.get())
    }

    fn position_from_screen_lparam(&self, hwnd: HWND, lparam: LPARAM) -> Point {
        let screen_pos = point_from_lparam(lparam);
        let client_pos: Point = screen_to_client_pos(hwnd, screen_pos.x, screen_pos.y).into();
        client_pos.scale(1.0 / self.scale_factor.get())
    }
}

impl Window {
    pub fn open(handler: impl WindowHandler + 'static) -> Result<Self> {
        let this = Self::create(None, WS_OVERLAPPEDWINDOW, handler, true)?;

        unsafe {
            let value: BOOL = match get_theme() {
                WindowTheme::Light => false,
                WindowTheme::Dark => true,
            }
            .into();
            DwmSetWindowAttribute(
                this.handle,
                DWMWA_USE_IMMERSIVE_DARK_MODE,
                &value as *const _ as *const _,
                std::mem::size_of_val(&value) as _,
            )?;
        }

        let _ = unsafe { ShowWindow(this.handle, SW_SHOW) };
        Ok(this)
    }

    pub fn attach(parent: HWND, handler: impl WindowHandler + 'static) -> Result<Self> {
        let this = Self::create(Some(parent), WS_CHILD, handler, false)?;
        let _ = unsafe { ShowWindow(this.handle, SW_SHOW) };
        Ok(this)
    }

    pub fn set_size(&self, size: Rectangle<i32>) -> Result<()> {
        unsafe {
            SetWindowPos(
                self.handle,
                None,
                size.left(),
                size.top(),
                size.width(),
                size.height(),
                SET_WINDOW_POS_FLAGS::default(),
            )
        }
    }

    fn create(
        parent: Option<HWND>,
        style: WINDOW_STYLE,
        handler: impl WindowHandler + 'static,
        quit_app_on_exit: bool,
    ) -> Result<Self> {
        let instance = unsafe { GetModuleHandleW(None)? };
        REGISTER_WINDOW_CLASS.call_once(|| {
            let class = WNDCLASSW {
                lpszClassName: WINDOW_CLASS,
                hCursor: unsafe { LoadCursorW(None, IDC_ARROW).unwrap() },
                hInstance: instance.into(),
                lpfnWndProc: Some(wndproc),
                style: CS_HREDRAW | CS_VREDRAW | CS_DBLCLKS,
                ..WNDCLASSW::default()
            };
            assert_ne!(
                unsafe { RegisterClassW(&class) },
                0,
                "Unable to register window class"
            );
        });

        com::com_initialized();

        let ticks_per_second = unsafe {
            let mut frequency: i64 = 0;
            Performance::QueryPerformanceFrequency(&mut frequency as *mut _)
                .map(|_| frequency as f64)
        }?;

        let window_state = Rc::new(WindowState {
            renderer: RefCell::new(None),
            renderer_generation: Cell::new(RendererGeneration(0)),
            handler: RefCell::new(Box::new(handler)),
            last_mouse_pos: RefCell::new(None),
            ticks_per_second,
            current_key_event: RefCell::new(None),
            scale_factor: Rc::new(Cell::new(1.0)),
            captured_mouse_buttons: Default::default(),
            theme: Rc::new(Cell::new(get_theme())),
            quit_app_on_exit,
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
                parent,
                None,
                Some(instance.into()),
                Some(Rc::into_raw(window_state) as _),
            )?
        };

        Ok(Window {
            handle: hwnd,
            _phantom: PhantomData,
        })
    }
}

fn loword(lparam: LPARAM) -> u16 {
    (lparam.0 & 0xFFFF) as u16
}

fn hiword(lparam: LPARAM) -> u16 {
    ((lparam.0 >> 16) & 0xFFFF) as u16
}

fn point_from_lparam(lparam: LPARAM) -> Point<i32> {
    Point::new(
        (lparam.0 & 0xFFFF) as i16 as i32,
        ((lparam.0 >> 16) & 0xFFFF) as i16 as i32,
    )
}

fn screen_to_client_pos(hwnd: HWND, x: i32, y: i32) -> Point<i32> {
    let mut screen_pos = POINT { x, y };
    if unsafe { ScreenToClient(hwnd, &mut screen_pos as *mut _) }.as_bool() {
        Point::new(screen_pos.x, screen_pos.y)
    } else {
        Point::new(0, 0)
    }
}

fn get_message_pos(hwnd: HWND) -> Point<i32> {
    let pos = unsafe { GetMessagePos() };
    let x = ((pos & 0xFFFF) as u16) as i32;
    let y = (((pos >> 16) & 0xFFFF) as u16) as i32;
    screen_to_client_pos(hwnd, x, y)
}

fn peek_message(hwnd: HWND, msgmin: u32, msgmax: u32) -> Option<MSG> {
    let mut msg = MaybeUninit::uninit();
    let avail = unsafe { PeekMessageW(msg.as_mut_ptr(), Some(hwnd), msgmin, msgmax, PM_NOREMOVE) };
    if avail.into() {
        Some(unsafe { msg.assume_init() })
    } else {
        None
    }
}

fn has_wm_char_message(hwnd: HWND, last_flags: KeyFlags) -> bool {
    peek_message(hwnd, WM_CHAR, WM_CHAR)
        .filter(|msg| {
            let flags = KeyFlags::from_lparam(msg.lParam);
            flags.scan_code == last_flags.scan_code
        })
        .is_some()
}

unsafe extern "system" fn wndproc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
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
            let result = (*window_state_ptr).handle_message(hwnd, message, wparam, lparam);
            result.unwrap_or_else(|| DefWindowProcW(hwnd, message, wparam, lparam))
        }
    } else {
        DefWindowProcW(hwnd, message, wparam, lparam)
    }
}
