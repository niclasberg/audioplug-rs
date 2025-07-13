use std::cell::Cell;
use std::ffi::{CStr, CString};
use std::rc::Rc;

use crate::core::{Rectangle, WindowTheme};
use crate::platform::win::util;
use windows::core::Result;
use windows::Win32::Foundation::{HANDLE, HGLOBAL, HWND, RECT};
use windows::Win32::Graphics::Gdi::InvalidateRect;
use windows::Win32::System::DataExchange::CloseClipboard;
use windows::Win32::System::Memory::GMEM_MOVEABLE;
use windows::Win32::System::Ole::CF_TEXT;
use windows::Win32::System::{DataExchange, Memory};

use super::util::get_client_rect;

struct ScopeExit<F: Fn()>(F);

impl<F: Fn()> Drop for ScopeExit<F> {
    fn drop(&mut self) {
        self.0();
    }
}

pub struct Handle {
    hwnd: HWND,
    theme: Rc<Cell<WindowTheme>>,
}

impl Handle {
    pub(crate) fn new(hwnd: HWND, theme: Rc<Cell<WindowTheme>>) -> Self {
        Self { hwnd, theme }
    }

    pub fn theme(&self) -> WindowTheme {
        self.theme.get()
    }

    pub fn invalidate_window(&self) {
        let _ = unsafe { InvalidateRect(Some(self.hwnd), None, false) };
    }

    pub fn scale_factor(&self) -> f64 {
        util::get_scale_factor_for_window(self.hwnd)
    }

    pub fn invalidate(&self, rect: Rectangle) {
        let rect = rect.scale(self.scale_factor());
        let rect = RECT {
            left: rect.left().floor() as i32,
            top: rect.top().floor() as i32,
            right: rect.right().ceil() as i32,
            bottom: rect.bottom().ceil() as i32,
        };
        let _ = unsafe { InvalidateRect(Some(self.hwnd), Some(&rect as _), false) };
    }

    pub fn global_bounds(&self) -> Rectangle {
        let rect: Rectangle = get_client_rect(self.hwnd).into();
        rect.scale(1.0 / self.scale_factor())
    }

    pub fn set_clipboard(&self, string: &str) -> Result<()> {
        unsafe { DataExchange::OpenClipboard(Some(self.hwnd)) }?;
        let _close_clipboard =
            ScopeExit(|| unsafe { CloseClipboard().expect("Error while closing clipboard") });
        unsafe { DataExchange::EmptyClipboard() }?;

        if !string.is_empty() {
            let chars = CString::new(string).unwrap();
            let chars = chars.as_bytes_with_nul();
            unsafe {
                let hmem: HGLOBAL =
                    Memory::GlobalAlloc(GMEM_MOVEABLE, std::mem::size_of_val(chars))?;
                let mem_loc = Memory::GlobalLock(hmem);
                std::ptr::copy_nonoverlapping(chars.as_ptr(), mem_loc as *mut u8, chars.len());
                let _ = Memory::GlobalUnlock(hmem);
                DataExchange::SetClipboardData(CF_TEXT.0.into(), Some(HANDLE(hmem.0)))?;
            };
        }

        Ok(())
    }

    pub fn get_clipboard(&self) -> Result<Option<String>> {
        let available =
            unsafe { DataExchange::IsClipboardFormatAvailable(CF_TEXT.0.into()) }.is_ok();
        if !available {
            return Ok(None);
        }

        unsafe { DataExchange::OpenClipboard(Some(self.hwnd)) }?;
        let _close_clipboard =
            ScopeExit(|| unsafe { CloseClipboard().expect("Error while closing clipboard") });

        unsafe {
            let hmem: HANDLE = DataExchange::GetClipboardData(CF_TEXT.0.into())?;
            let hmem = HGLOBAL(hmem.0 as *mut _);
            let str_handle = Memory::GlobalLock(hmem);

            assert!(!str_handle.is_null());

            let str = CStr::from_ptr(str_handle as *mut _);
            let result = str.to_str().map(str::to_owned);

            let _ = Memory::GlobalUnlock(hmem);

            Ok(result.ok())
        }
    }
}
