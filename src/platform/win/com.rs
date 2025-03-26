use std::sync::OnceLock;
use windows::Win32::System::Com::{self, CoCreateInstance, CLSCTX_INPROC_SERVER};
use windows::Win32::Graphics::{DirectWrite, Direct2D, Imaging};

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

pub(super) fn direct_write_factory() -> &'static DirectWrite::IDWriteFactory {
    static INSTANCE: OnceLock<DirectWrite::IDWriteFactory> = OnceLock::new();
    INSTANCE.get_or_init(|| {
        com_initialized();
        unsafe { DirectWrite::DWriteCreateFactory(DirectWrite::DWRITE_FACTORY_TYPE_SHARED).unwrap() }
    })
}

pub(super) fn direct2d_factory() -> &'static Direct2D::ID2D1Factory {
    static INSTANCE: OnceLock<Direct2D::ID2D1Factory> = OnceLock::new();
    INSTANCE.get_or_init(|| {
        com_initialized();
        unsafe { Direct2D::D2D1CreateFactory::<Direct2D::ID2D1Factory>(Direct2D::D2D1_FACTORY_TYPE_MULTI_THREADED, None).unwrap() }
    })
}

struct WICImagingFactory(Imaging::IWICImagingFactory);
unsafe impl Sync for WICImagingFactory {}
unsafe impl Send for WICImagingFactory {}

pub(super) fn wic_factory() -> &'static Imaging::IWICImagingFactory {
    static INSTANCE: OnceLock<WICImagingFactory> = OnceLock::new();
    let factory = INSTANCE.get_or_init(|| {
        com_initialized();
        WICImagingFactory(unsafe { CoCreateInstance(&Imaging::CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER).unwrap() })
    });
    &factory.0
}