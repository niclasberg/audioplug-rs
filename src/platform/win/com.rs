use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};
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

static CONTEXT: OnceLock<&'static mut COMContext> = OnceLock::new();

pub(crate) fn get_com_context() -> &'static COMContext {
    CONTEXT.get_or_init(|| {
        Box::leak(Box::new(
            COMContext::new().expect("Could not initialize COM objects"),
        ))
    })
}

static COM_CONTEXT_DROPPED: AtomicBool = AtomicBool::new(false);

pub(crate) fn drop_com_context() {
    if COM_CONTEXT_DROPPED.swap(true, Ordering::AcqRel) {
        return; // already shut down
    }

    if let Some(context) = CONTEXT.get() {
        unsafe {
            // SAFETY: context is a reference to a leaked Box, and we're only reclaiming it once
            let ptr = *context as *const COMContext as *mut COMContext;
            drop(Box::from_raw(ptr));
        }
    }
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
