use thiserror::Error;
use wgpu::{DeviceDescriptor, RequestAdapterOptionsBase, SurfaceTargetUnsafe};

use crate::{
    core::{PhysicalCoord, PhysicalSize, Rect, Size, WindowTheme},
    platform,
};

#[derive(Error, Debug)]
pub enum GraphicsInitError {
    #[error("Could not get target surface")]
    WindowHandleError(#[from] raw_window_handle::HandleError),
    #[error("Could not create the wgpu surface")]
    CreateSurface(#[from] wgpu::CreateSurfaceError),
    #[error("Could not get the wgpu adapter")]
    RequestAdapter(#[from] wgpu::RequestAdapterError),
    #[error("Could not get the wgpu device")]
    RequestDevice(#[from] wgpu::RequestDeviceError),
}

pub struct WindowHandle {
    pub handle: super::Handle,
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: PhysicalSize,
    pub surface_format: wgpu::TextureFormat,
}

impl WindowHandle {
    pub(super) async fn new(handle: platform::Handle) -> Result<Self, GraphicsInitError> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        // SAFETY: This struct is owned by the WindowHandler, whose lifetime is shorter than the OS window itself.
        let surface_target = SurfaceTargetUnsafe::RawHandle {
            raw_display_handle: handle.raw_display_handle(),
            raw_window_handle: handle.raw_window_handle(),
        };
        let surface = unsafe { instance.create_surface_unsafe(surface_target) }?;
        let adapter = instance
            .request_adapter(&RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await?;

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: Some("device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::defaults(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })
            .await?;

        let surface_capabilities = surface.get_capabilities(&adapter);
        let format = surface_capabilities
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_capabilities.formats[0]);
        // Prefer Mailbox for lower latency, otherwise fallback to FIFO
        let present_mode = surface_capabilities
            .present_modes
            .iter()
            .copied()
            .find(|pm| *pm == wgpu::PresentMode::Mailbox)
            .unwrap_or(wgpu::PresentMode::Fifo);

        let size = handle.physical_size();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.0 as _,
            height: size.height.0 as _,
            present_mode,
            desired_maximum_frame_latency: 2,
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        Ok(Self {
            handle,
            surface,
            device,
            queue,
            config,
            size,
            surface_format: format,
        })
    }

    pub fn resize(&mut self, new_size: PhysicalSize) {
        if new_size.height > PhysicalCoord::ZERO && new_size.width > PhysicalCoord::ZERO {
            self.size = new_size;
            self.config.width = new_size.width.0 as _;
            self.config.height = new_size.height.0 as _;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn global_bounds(&self) -> Rect {
        self.handle.global_bounds()
    }

    pub fn invalidate_window(&self) {
        self.handle.invalidate_window();
    }

    pub fn invalidate(&self, rect: Rect) {
        self.handle.invalidate(rect);
    }

    pub fn theme(&self) -> WindowTheme {
        self.handle.theme()
    }

    pub fn set_clipboard(&self, string: &str) -> Result<(), super::Error> {
        self.handle.set_clipboard(string)
    }

    pub fn get_clipboard(&self) -> Result<Option<String>, super::Error> {
        self.handle.get_clipboard()
    }
}

impl raw_window_handle::HasWindowHandle for WindowHandle {
    fn window_handle(
        &self,
    ) -> std::result::Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError>
    {
        let handle =
            unsafe { raw_window_handle::WindowHandle::borrow_raw(self.handle.raw_window_handle()) };
        Ok(handle)
    }
}

impl raw_window_handle::HasDisplayHandle for WindowHandle {
    fn display_handle(
        &self,
    ) -> std::result::Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError>
    {
        let display_handle = unsafe {
            raw_window_handle::DisplayHandle::borrow_raw(self.handle.raw_display_handle())
        };
        Ok(display_handle)
    }
}
