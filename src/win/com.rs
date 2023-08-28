use windows::Win32::System::Com;

thread_local! { static COM_INITIALIZED: ComInitialized = {
    unsafe { Com::CoInitializeEx(None, Com::COINIT_APARTMENTTHREADED) }.unwrap();
    ComInitialized
}}

struct ComInitialized;

impl Drop for ComInitialized {
    #[inline]
    fn drop(&mut self) {
        unsafe { Com::CoUninitialize() };
    }
}

pub fn com_initialized() {
    COM_INITIALIZED.with(|_| { })
}