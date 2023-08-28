use windows::{
    core::*,
    Win32::UI::WindowsAndMessaging::*
};

pub struct Application {

}

impl Application {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(&mut self) {
        let mut msg = MSG::default();
        unsafe {
            while GetMessageW(&mut msg, None, 0, 0).into() {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }
}