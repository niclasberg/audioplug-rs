use std::mem::MaybeUninit;
use std::sync::LazyLock;

use windows::core::{s, PCSTR};
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryA};
use windows::Win32::UI::Accessibility::{HCF_HIGHCONTRASTON, HIGHCONTRASTW};
use windows::Win32::UI::HiDpi::GetDpiForWindow;
use windows::Win32::UI::WindowsAndMessaging::{
    GetClientRect, SystemParametersInfoW, SPI_GETHIGHCONTRAST, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS,
};

use crate::core::{Rectangle, WindowTheme};

pub(super) fn get_client_rect(hwnd: HWND) -> Rectangle<i32> {
    let mut rect: RECT = RECT::default();
    if let Ok(()) = unsafe { GetClientRect(hwnd, &mut rect) } {
        Rectangle::from_ltrb(rect.left, rect.top, rect.right, rect.bottom)
    } else {
        Rectangle::default()
    }
}

const BASE_DPI: u32 = 96;
pub fn dpi_to_scale_factor(dpi: u32) -> f64 {
    dpi as f64 / BASE_DPI as f64
}

pub fn get_scale_factor_for_window(hwnd: HWND) -> f64 {
    let dpi = unsafe { GetDpiForWindow(hwnd) };
    dpi_to_scale_factor(dpi)
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

pub(crate) fn count_until_utf16(s: &str, utf16_text_position: usize) -> Option<usize> {
    let mut utf16_count = 0;
    for (utf8_count, &b) in s.as_bytes().iter().enumerate() {
        if (b as i8) >= -0x40 {
            utf16_count += 1;
        }
        if b >= 0xf0 {
            utf16_count += 1;
        }

        if utf16_count > utf16_text_position {
            return Some(utf8_count);
        }
    }

    None
}

pub(crate) unsafe fn utf16_ptr_to_string(ptr: *const u16) -> Option<String> {
    let len = (0..).take_while(|&i| *ptr.offset(i) != 0).count();
    let slice = std::slice::from_raw_parts(ptr, len);

    String::from_utf16(slice).ok()
}

pub fn get_theme() -> WindowTheme {
    if should_apps_use_dark_mode() && !is_high_contrast() {
        WindowTheme::Dark
    } else {
        WindowTheme::Light
    }
}

fn should_apps_use_dark_mode() -> bool {
    type ShouldAppsUseDarkMode = unsafe extern "system" fn() -> bool;
    static SHOULD_APPS_USE_DARK_MODE: LazyLock<Option<ShouldAppsUseDarkMode>> =
        LazyLock::new(|| unsafe {
            const UXTHEME_SHOULDAPPSUSEDARKMODE_ORDINAL: PCSTR = PCSTR::from_raw(132 as *mut u8);

            let Ok(module) = LoadLibraryA(s!("uxtheme.dll")) else {
                return None;
            };
            let handle = GetProcAddress(module, UXTHEME_SHOULDAPPSUSEDARKMODE_ORDINAL);
            handle.map(|handle| std::mem::transmute(handle))
        });

    SHOULD_APPS_USE_DARK_MODE
        .map(|should_apps_use_dark_mode| unsafe { (should_apps_use_dark_mode)() })
        .unwrap_or(false)
}

fn is_high_contrast() -> bool {
    let hc = unsafe {
        let mut hc = MaybeUninit::<HIGHCONTRASTW>::uninit();
        let status = SystemParametersInfoW(
            SPI_GETHIGHCONTRAST,
            std::mem::size_of_val(&hc) as _,
            Some(hc.as_mut_ptr() as _),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        );

        if status.is_err() {
            return false;
        }

        hc.assume_init()
    };
    (hc.dwFlags & HCF_HIGHCONTRASTON).0 != 0
}
