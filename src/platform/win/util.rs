use windows::Win32::UI::WindowsAndMessaging::GetClientRect;
use windows::Win32::Foundation::{HWND, RECT};

use crate::core::Rectangle;

pub(super) fn get_client_rect(hwnd: HWND) -> Rectangle<i32> {
    let mut rect: RECT = RECT::default();
    unsafe { GetClientRect(hwnd, &mut rect) };
    Rectangle::from_ltrb(rect.left, rect.top, rect.right, rect.bottom)
}