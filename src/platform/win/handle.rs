use std::ffi::{CStr, CString};
use std::num::NonZeroIsize;

use crate::core::{PhysicalSize, Rect, ScaleFactor, WindowTheme};
use crate::platform::win::util::{self, get_theme};
use raw_window_handle::{
    HandleError, RawDisplayHandle, RawWindowHandle, Win32WindowHandle, WindowsDisplayHandle,
};
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

#[repr(transparent)]
pub struct Handle {
    hwnd: HWND,
}

// WGPU requires the handle to be sync + send
unsafe impl Send for Handle {}
unsafe impl Sync for Handle {}

impl Handle {
    pub(crate) fn new(hwnd: HWND) -> Self {
        Self { hwnd }
    }

    pub fn physical_size(&self) -> PhysicalSize {
        let client_rect = get_client_rect(self.hwnd);
        let scale_factor = self.scale_factor();
        PhysicalSize::from_logical(client_rect.size().into(), scale_factor)
    }

    pub fn theme(&self) -> WindowTheme {
        get_theme()
    }

    pub fn invalidate_window(&self) {
        let _ = unsafe { InvalidateRect(Some(self.hwnd), None, false) };
    }

    pub fn scale_factor(&self) -> ScaleFactor {
        util::get_scale_factor_for_window(self.hwnd)
    }

    pub fn invalidate(&self, rect: Rect) {
        let rect = rect.scale(self.scale_factor().0);
        let rect = RECT {
            left: rect.left.floor() as i32,
            top: rect.top.floor() as i32,
            right: rect.right.ceil() as i32,
            bottom: rect.bottom.ceil() as i32,
        };
        let _ = unsafe { InvalidateRect(Some(self.hwnd), Some(&rect as _), false) };
    }

    pub fn global_bounds(&self) -> Rect {
        let rect: Rect = get_client_rect(self.hwnd).into();
        rect.scale(1.0 / self.scale_factor().0)
    }

    pub fn set_clipboard(&self, string: &str) -> Result<(), windows_core::Error> {
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

    pub fn get_clipboard(&self) -> Result<Option<String>, windows_core::Error> {
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

    pub fn raw_window_handle(&self) -> Result<RawWindowHandle, HandleError> {
        let hwnd_isize = NonZeroIsize::new(self.hwnd.0 as _).unwrap();
        Ok(RawWindowHandle::Win32(Win32WindowHandle::new(hwnd_isize)))
    }

    pub fn raw_display_handle(&self) -> Result<RawDisplayHandle, HandleError> {
        Ok(RawDisplayHandle::Windows(WindowsDisplayHandle::new()))
    }
}
