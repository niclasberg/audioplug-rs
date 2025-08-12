use thiserror::Error;
use wgpu::{DeviceDescriptor, RequestAdapterOptionsBase, SurfaceTargetUnsafe};

use crate::core::{PhysicalCoord, PhysicalSize};

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

pub struct WGPUSurface {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: PhysicalSize,
    pub surface_format: wgpu::TextureFormat,
}

impl WGPUSurface {
    pub async fn new(handle: &crate::platform::Handle) -> Result<Self, GraphicsInitError> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        // SAFETY: This struct is owned by the WindowHandler, whose lifetime is shorter than the OS window itself.
        let surface_target = SurfaceTargetUnsafe::RawHandle {
            raw_display_handle: handle.raw_display_handle()?,
            raw_window_handle: handle.raw_window_handle()?,
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
}
