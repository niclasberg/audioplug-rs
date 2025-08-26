use bytemuck::{Pod, Zeroable};
use thiserror::Error;
use wgpu::util::DeviceExt;

use crate::core::{PhysicalCoord, PhysicalSize};

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod)]
struct Params {
    width: u32,
    height: u32,
}

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
    // Blit pipeline
    pub blit_pipeline: wgpu::RenderPipeline,
    pub blit_bind_group_layout: wgpu::BindGroupLayout,
    pub blit_bind_group: wgpu::BindGroup,
    pub output_sampler: wgpu::Sampler,
    // Render tiles pipeline
    pub render_tiles_pipeline: wgpu::ComputePipeline,
    pub render_tiles_bind_group_layout: wgpu::BindGroupLayout,
    pub render_tiles_bind_group: wgpu::BindGroup,
    pub output_texture: wgpu::Texture,
    pub params_buffer: wgpu::Buffer,
}

impl WGPUSurface {
    pub async fn new(handle: &crate::platform::Handle) -> Result<Self, GraphicsInitError> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        // SAFETY: This struct is owned by the WindowHandler, whose lifetime is shorter than the OS window itself.
        let surface_target = wgpu::SurfaceTargetUnsafe::RawHandle {
            raw_display_handle: handle.raw_display_handle()?,
            raw_window_handle: handle.raw_window_handle()?,
        };
        let surface = unsafe { instance.create_surface_unsafe(surface_target) }?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
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
        let width = size.width.0 as u32;
        let height = size.height.0 as u32;

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            present_mode,
            desired_maximum_frame_latency: 2,
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let params = Params { width, height };
        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Params buffer"),
            contents: bytemuck::bytes_of(&params),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        // Output texture for compute shader
        let output_texture = create_output_texture(&device, width, height);
        let output_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Output texture sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let tex_view = output_texture.create_view(&Default::default());

        let (blit_pipeline, blit_bind_group_layout) = create_blit_pipeline(&device, format);
        let blit_bind_group =
            create_blit_bind_group(&device, &blit_bind_group_layout, &tex_view, &output_sampler);
        let (render_tiles_pipeline, render_tiles_bind_group_layout) =
            create_render_tiles_pipeline(&device, width, height);
        let render_tiles_bind_group = create_render_tiles_bind_group(
            &device,
            &render_tiles_bind_group_layout,
            &tex_view,
            &params_buffer,
        );

        Ok(Self {
            surface,
            device,
            queue,
            config,
            size,
            surface_format: format,
            blit_pipeline,
            blit_bind_group_layout,
            blit_bind_group,
            render_tiles_pipeline,
            render_tiles_bind_group_layout,
            render_tiles_bind_group,
            output_texture,
            output_sampler,
            params_buffer,
        })
    }

    pub fn resize(&mut self, new_size: PhysicalSize) {
        if new_size.height > PhysicalCoord::ZERO && new_size.width > PhysicalCoord::ZERO {
            self.size = new_size;
            let width = new_size.width.0 as _;
            let height = new_size.height.0 as _;
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.output_texture = create_output_texture(&self.device, width, height);

            let texture_view = self
                .output_texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            self.blit_bind_group = create_blit_bind_group(
                &self.device,
                &self.blit_bind_group_layout,
                &texture_view,
                &self.output_sampler,
            );
            self.render_tiles_bind_group = create_render_tiles_bind_group(
                &self.device,
                &self.render_tiles_bind_group_layout,
                &texture_view,
                &self.params_buffer,
            );
        }
    }
}

fn create_output_texture(device: &wgpu::Device, width: u32, height: u32) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Output texture"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    })
}

fn create_blit_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
) -> (wgpu::RenderPipeline, wgpu::BindGroupLayout) {
    let shader = device.create_shader_module(wgpu::include_wgsl!("shaders/blit.wgsl"));
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Blit bind group layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                count: None,
            },
        ],
    });
    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Blit pipeline layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Blit pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            compilation_options: Default::default(),
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Cw,
            cull_mode: Some(wgpu::Face::Back),
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    });
    (pipeline, bind_group_layout)
}

fn create_blit_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    tex_view: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Blit bind group"),
        layout: &layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&tex_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    })
}

fn create_render_tiles_pipeline(
    device: &wgpu::Device,
    width: u32,
    height: u32,
) -> (wgpu::ComputePipeline, wgpu::BindGroupLayout) {
    let shader = device.create_shader_module(wgpu::include_wgsl!("shaders/render_tiles.wgsl"));
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("render_tiles bind group layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            },
        ],
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render tiles layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Render tiles compute pipeline"),
        layout: Some(&layout),
        module: &shader,
        entry_point: Some("main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });
    (pipeline, bind_group_layout)
}

fn create_render_tiles_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    tex_view: &wgpu::TextureView,
    params_buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Render tiles bind group"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: params_buffer,
                    offset: 0,
                    size: None,
                }),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(tex_view),
            },
        ],
    })
}
