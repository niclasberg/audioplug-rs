use windows::Win32::UI::WindowsAndMessaging::GetClientRect;
use windows::Win32::Foundation::{HWND, RECT};

use crate::core::Rectangle;

pub(super) fn get_client_rect(hwnd: HWND) -> Rectangle<i32> {
    let mut rect: RECT = RECT::default();
    unsafe { GetClientRect(hwnd, &mut rect) };
    Rectangle::from_ltrb(rect.left, rect.top, rect.right, rect.bottom)
}

/// Counts the number of utf-16 code units in the given string.
/// from xi-editor
pub(crate) fn count_utf16(s: &str) -> usize {
    let mut utf16_count = 0;
    for &b in s.as_bytes() {
        if (b as i8) >= -0x40 {
            utf16_count += 1;
        }
        if b >= 0xf0 {
            utf16_count += 1;
        }
    }
    utf16_count
}

pub(crate) unsafe fn utf16_ptr_to_string(ptr: *const u16) -> Option<String> {
    let len = (0..).take_while(|&i| *ptr.offset(i) != 0).count();
    let slice = std::slice::from_raw_parts(ptr, len);

    String::from_utf16(slice).ok()
}