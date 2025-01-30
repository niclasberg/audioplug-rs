use std::{future::Future, mem::MaybeUninit, ops::Deref, rc::Rc, sync::{mpsc::{self, Receiver, Sender}, Once}};
use async_task::{self, Task, Runnable};

use windows::{core::{w, Result, PCWSTR}, Win32::{Foundation::{HWND, LPARAM, LRESULT, WPARAM}, System::LibraryLoader::GetModuleHandleW, UI::WindowsAndMessaging::{CreateWindowExW, DefWindowProcW, GetClassInfoW, GetWindowLongPtrW, PostMessageA, RegisterClassW, SetWindowLongPtrW, CREATESTRUCTW, CW_USEDEFAULT, GWLP_USERDATA, HWND_MESSAGE, WINDOW_EX_STYLE, WINDOW_STYLE, WM_NCCREATE, WM_NCDESTROY, WM_USER, WNDCLASSW}}};

const WINDOW_CLASS: PCWSTR = w!("executor_window");
static REGISTER_WINDOW_CLASS: Once = Once::new();
const WAKER_MSG_ID: u32 = WM_USER + 1;

pub enum Work {
    Runnable(Runnable),
    Fn(Box<dyn FnOnce()>)
}
unsafe impl Send for Work {}
unsafe impl Sync for Work {}

struct Inner {
    receiver: Receiver<Work>,
}

impl Inner {
    fn poll(&self) {
        for work in self.receiver.iter() {
            match work {
                Work::Runnable(runnable) => { runnable.run(); },
                Work::Fn(f) => f(),
            }
        }
    }
}

// Safety: We only used this hwnd to call PostMessage, which is thread safe
struct SafeHWND(HWND);
unsafe impl Send for SafeHWND {}
unsafe impl Sync for SafeHWND {}
impl Deref for SafeHWND {
    type Target = HWND;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct Executor {
    hwnd: HWND,
    sender: Sender<Work>
}

impl Executor {
    pub fn new() -> Result<Self> {
        let instance = unsafe { GetModuleHandleW(None)? };
        REGISTER_WINDOW_CLASS.call_once(|| {
            {
                // See if the window class already has been registered
                let mut class = MaybeUninit::uninit();
                let result = unsafe { GetClassInfoW(Some(instance.into()), WINDOW_CLASS, class.as_mut_ptr())};
                if result.is_ok() {
                    return;
                }
            }

            let class = WNDCLASSW {
                lpszClassName: WINDOW_CLASS,
                hInstance: instance.into(),
                lpfnWndProc: Some(wndproc),
                ..WNDCLASSW::default()
            };
            assert_ne!(unsafe { RegisterClassW(&class) }, 0, "Unable to register window class");
        });

        let (sender, receiver) = mpsc::channel();

        let inner = Rc::new(Inner{receiver});
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
                Some(HWND_MESSAGE), 
                None, 
                None, 
                Some(Rc::into_raw(inner.clone()) as _))?
        };

        Ok(Self {
            hwnd,
            sender
        })
    }

    pub fn dispatch(&self, f: impl Fn()) {
        
    }

    pub fn dispatch_on_main(&self, f: impl FnOnce() + 'static) {
        self.sender.send(Work::Fn(Box::new(f))).expect("Failed to enqueue task");
        unsafe { PostMessageA(Some(self.hwnd), WAKER_MSG_ID, WPARAM(0), LPARAM(0)) }.unwrap();
    }

    pub fn dispatch_future_on_main<T: 'static>(&self, f: impl Future<Output = T> + 'static) -> Task<T> {
        let sender = self.sender.clone();
        let hwnd = SafeHWND(self.hwnd);
        let schedule = move |runnable| {
            sender.send(Work::Runnable(runnable)).expect("Failed to enqueue task");
            unsafe { PostMessageA(Some(*hwnd), WAKER_MSG_ID, WPARAM(0), LPARAM(0)) }.unwrap();
        };
        let (runnable, task) = async_task::spawn_local(f, schedule);
        runnable.schedule();
        task
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
        if message == WAKER_MSG_ID {
            (*inner_ptr).poll();
            return LRESULT(0);
        }
        
        if message == WM_NCDESTROY {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            drop(Rc::from_raw(inner_ptr));
        }
    } 

    DefWindowProcW(hwnd, message, wparam, lparam)
}