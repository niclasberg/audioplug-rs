use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Gdi::InvalidateRect;
use crate::core::Rectangle;

use super::util::get_client_rect;

pub type HandleRef<'a> = &'a mut Handle;

pub struct Handle {
    hwnd: HWND
}

impl Handle {
    pub(crate) fn new(hwnd: HWND) -> Self {
        Self { hwnd }
    }

    pub fn invalidate(&self, rect: Rectangle) {
        let rect = RECT {
            left: rect.left().floor() as i32, 
            top: rect.top().floor() as i32, 
            right: rect.right().ceil() as i32, 
            bottom: rect.bottom().ceil() as i32
        };
        unsafe { InvalidateRect(self.hwnd, Some(&rect as _), false) };
    }

    pub fn global_bounds(&self) -> Rectangle {
        let rect = get_client_rect(self.hwnd);
        rect.into()
    }
}