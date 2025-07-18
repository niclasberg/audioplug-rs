use windows::Win32::UI::{
    HiDpi::{DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, SetProcessDpiAwarenessContext},
    WindowsAndMessaging::*,
};

pub struct Application {}

impl Application {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(&mut self) {
        let mut msg = MSG::default();
        unsafe {
            SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2).unwrap();
            while GetMessageW(&mut msg, None, 0, 0).into() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }
}
