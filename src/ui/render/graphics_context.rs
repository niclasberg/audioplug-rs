use thiserror::Error;
use wgpu::{DeviceDescriptor, RequestAdapterOptionsBase};

use crate::{core::Size, platform};

#[derive(Error, Debug)]
pub enum GraphicsInitError {
    #[error("Could not create the wgpu surface")]
    CreateSurface(#[from] wgpu::CreateSurfaceError),
    #[error("Could not get the wgpu adapter")]
    RequestAdapter(#[from] wgpu::RequestAdapterError),
    #[error("Could not get the wgpu device")]
    RequestDevice(#[from] wgpu::RequestDeviceError),
}

pub struct GraphicsContext<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: Size<u32>,
}

impl<'a> GraphicsContext<'a> {
    pub(crate) async fn new(handle: &'a platform::Handle) -> Result<Self, GraphicsInitError> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let surface = instance.create_surface(handle)?;
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
            width: size.width,
            height: size.height,
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
        })
    }

    pub fn resize(&mut self, new_size: Size<u32>) {
        if new_size.height > 0 && new_size.width > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }
}
