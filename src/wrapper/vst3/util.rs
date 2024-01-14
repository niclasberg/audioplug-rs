use std::ffi::c_char;
use vst3_sys::vst::TChar;

pub fn strcpy(src: &str, dst: &mut [c_char]) {
    let src = src.as_bytes();
    let src = unsafe { &*(src as *const [u8] as *const [c_char]) };
    let len = std::cmp::min(dst.len() - 1, src.len());
    dst[..len].copy_from_slice(&src[..len]);
    dst[len] = 0;
}

pub fn strcpyw(src: &str, dst: &mut [TChar]) {
    let mut src: Vec<u16> = src.encode_utf16().collect();
    src.push(0);
    let src = src.as_slice();
    let src = unsafe { &*(src as *const [u16] as *const [TChar]) };
    let len = std::cmp::min(dst.len() - 1, src.len());
    dst[..len].copy_from_slice(&src[..len]);
    dst[len] = 0;
}