use std::{rc::Rc, sync::Once};

use windows::{core::{w, Result, PCWSTR}, Win32::{Foundation::{HWND, LPARAM, LRESULT, WPARAM}, System::LibraryLoader::GetModuleHandleW, UI::WindowsAndMessaging::{CreateWindowExW, DefWindowProcW, GetWindowLongPtrW, RegisterClassW, SetWindowLongPtrW, CREATESTRUCTW, CW_USEDEFAULT, GWLP_USERDATA, HWND_MESSAGE, WINDOW_EX_STYLE, WINDOW_STYLE, WM_NCCREATE, WM_NCDESTROY, WNDCLASSW}}};

const WINDOW_CLASS: PCWSTR = w!("executor_window");
static REGISTER_WINDOW_CLASS: Once = Once::new();

struct Inner {

}

pub struct Executor {
    hwnd: HWND
}

impl Executor {
    pub fn new() -> Result<Self> {
        let instance = unsafe { GetModuleHandleW(None)? };
        REGISTER_WINDOW_CLASS.call_once(|| {
            let class = WNDCLASSW {
                lpszClassName: WINDOW_CLASS,
                hInstance: instance.into(),
                lpfnWndProc: Some(wndproc),
                ..WNDCLASSW::default()
            };
            assert_ne!(unsafe { RegisterClassW(&class) }, 0, "Unable to register window class");
        });

        let inner = Rc::new(Inner{});

        let hwnd = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(), 
                WINDOW_CLASS, 
                w!("dummy title"), 
                WINDOW_STYLE::default(), 
                0, 
                0, 
                0, 
                0, 
                HWND_MESSAGE, 
                None, 
                None, 
                Some(Rc::into_raw(inner) as _))?
        };
    }
}

unsafe extern "system" fn wndproc(hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if message == WM_NCCREATE {
        let create_struct = &*(lparam.0 as *const CREATESTRUCTW);
        let inner_ptr = create_struct.lpCreateParams;
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, inner_ptr as _);
    }

    let inner_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Inner;
    if !inner_ptr.is_null() {
        if message == WM_NCDESTROY {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            drop(Rc::from_raw(inner_ptr));
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