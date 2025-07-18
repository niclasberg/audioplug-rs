use std::sync::{Mutex, OnceLock, RwLock};
use windows::Win32::Graphics::{Direct2D, DirectWrite, Imaging};
use windows::Win32::System::Com::{self, CLSCTX_INPROC_SERVER, CoCreateInstance};
use windows_core::Result;

struct ComInitialized;

impl Drop for ComInitialized {
    #[inline]
    fn drop(&mut self) {
        unsafe { Com::CoUninitialize() };
    }
}

struct WICImagingFactory(Imaging::IWICImagingFactory);
unsafe impl Sync for WICImagingFactory {}
unsafe impl Send for WICImagingFactory {}

pub struct COMContext {
    direct_write_factory: DirectWrite::IDWriteFactory,
    direct2d_factory: Direct2D::ID2D1Factory1,
    wic_factory: WICImagingFactory,
    _com_init: ComInitialized,
}

impl COMContext {
    pub fn new() -> Result<Self> {
        unsafe { Com::CoInitializeEx(None, Com::COINIT_APARTMENTTHREADED) }.ok()?;
        let direct_write_factory =
            unsafe { DirectWrite::DWriteCreateFactory(DirectWrite::DWRITE_FACTORY_TYPE_SHARED) }?;
        let direct2d_factory = unsafe {
            Direct2D::D2D1CreateFactory::<Direct2D::ID2D1Factory1>(
                Direct2D::D2D1_FACTORY_TYPE_MULTI_THREADED,
                Some(&Direct2D::D2D1_FACTORY_OPTIONS {
                    debugLevel: Direct2D::D2D1_DEBUG_LEVEL_INFORMATION,
                }),
            )
        }?;
        let wic_factory = WICImagingFactory(unsafe {
            CoCreateInstance(
                &Imaging::CLSID_WICImagingFactory,
                None,
                CLSCTX_INPROC_SERVER,
            )?
        });

        Ok(Self {
            direct_write_factory,
            direct2d_factory,
            wic_factory,
            _com_init: ComInitialized,
        })
    }
}

static CONTEXT: RwLock<Option<COMContext>> = RwLock::new(None);

pub(super) fn get_com_context() -> &'static COMContext {
    let cx = CONTEXT.read().unwrap();
    cx.as_ref().expect("Com should have been initialized")
}

pub(crate) fn init_com() {
    let context = CONTEXT.lock();
}

pub(super) fn direct_write_factory() -> &'static DirectWrite::IDWriteFactory {
    &get_com_context().direct_write_factory
}

pub(super) fn direct2d_factory() -> &'static Direct2D::ID2D1Factory1 {
    &get_com_context().direct2d_factory
}

pub(super) fn wic_factory() -> &'static Imaging::IWICImagingFactory {
    &get_com_context().wic_factory.0
}
