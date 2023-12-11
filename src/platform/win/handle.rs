use std::ffi::{CString, c_char, CStr};
use std::mem::size_of;

use windows::Win32::System::DataExchange::CloseClipboard;
use windows::Win32::System::Ole::CF_TEXT;
use windows::core::Result;
use windows::Win32::Foundation::{HWND, RECT, HGLOBAL, HANDLE};
use windows::Win32::System::Memory::GMEM_MOVEABLE;
use windows::Win32::System::{DataExchange, Memory};
use windows::Win32::Graphics::Gdi::InvalidateRect;
use crate::core::Rectangle;

use super::util::{get_client_rect, utf16_ptr_to_string};

pub type HandleRef<'a> = &'a mut Handle;

struct ScopeExit<F: Fn()>(F);

impl<F: Fn()> Drop for ScopeExit<F> {
    fn drop(&mut self) {
        self.0();
    }
}

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

    pub fn set_clipboard(&self, string: &str) -> Result<()>{
        unsafe { DataExchange::OpenClipboard(self.hwnd) }?;
        let _close_clipboard = ScopeExit(|| unsafe { CloseClipboard().expect("Error while closing clipboard") });
        unsafe { DataExchange::EmptyClipboard() }?;

        if string.len() > 0 {
            let chars = CString::new(string).unwrap();
            let chars = chars.as_bytes_with_nul();
            unsafe {
                let hmem: HGLOBAL =  Memory::GlobalAlloc(GMEM_MOVEABLE, chars.len() * size_of::<u8>())?;
                let mem_loc = Memory::GlobalLock(hmem);
                std::ptr::copy_nonoverlapping(chars.as_ptr(), mem_loc as *mut u8, chars.len());
                let _ = Memory::GlobalUnlock(hmem);
                DataExchange::SetClipboardData(CF_TEXT.0.into(), HANDLE(hmem.0 as isize))?;
            };
        }

        Ok(())
    }

    pub fn get_clipboard(&self) -> Result<Option<String>> {
        let available = unsafe { DataExchange::IsClipboardFormatAvailable(CF_TEXT.0.into()) }.is_ok();
        if !available {
            return Ok(None); 
        }

        unsafe { DataExchange::OpenClipboard(self.hwnd) }?;
        let _close_clipboard = ScopeExit(|| unsafe { CloseClipboard().expect("Error while closing clipboard") });

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