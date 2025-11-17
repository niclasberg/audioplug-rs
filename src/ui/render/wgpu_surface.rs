use std::num::NonZero;

use bytemuck::{Pod, Zeroable};
use thiserror::Error;
use wgpu::util::DeviceExt;

use super::tiles::TILE_SIZE;
use crate::{
    core::{
        Color, ColorMap, Ellipse, FillRule, Path, PhysicalCoord, PhysicalSize, Point, Rect,
        RoundedRect, Size, Vec2f,
    },
    ui::render::gpu_scene::{GpuFill, GpuScene},
};

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
    pub blit_program: BlitProgram,
    pub blit_bind_group: wgpu::BindGroup,
    pub output_sampler: wgpu::Sampler,
    // Render tiles pipeline
    pub render_tiles_program: RenderTilesProgram,
    pub render_tiles_bind_group0: wgpu::BindGroup,
    pub render_tiles_bind_group1: wgpu::BindGroup,
    pub output_texture: wgpu::Texture,
    pub params_buffer: wgpu::Buffer,
    pub shapes_data_buffer: wgpu::Buffer,
    pub fill_ops_buffer: wgpu::Buffer,
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
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let mut gpu_scene = GpuScene::new();
        {
            let rect = gpu_scene.add_rect(Rect {
                left: 10.0,
                top: 10.0,
                right: 150.0,
                bottom: 200.0,
            });
            let rounded_rect = gpu_scene.add_rounded_rect(RoundedRect::new(
                Rect {
                    left: 50.3,
                    top: 100.2,
                    right: 500.0,
                    bottom: 400.3,
                },
                Size::new(40.0, 30.0),
            ));
            let path = gpu_scene.add_path(
                &Path::new()
                    .move_to(Point::new(100.0, 100.0))
                    .line_to(Point::new(100.0, 800.0))
                    .line_to(Point::new(800.0, 800.0))
                    .line_to(Point::new(700.0, 400.0))
                    .close_path(),
                FillRule::NonZero,
            );
            let ellipse = gpu_scene.add_ellipse(Ellipse::from_rectangle(Rect {
                left: 650.3,
                top: 200.2,
                right: 900.0,
                bottom: 300.3,
            }));

            gpu_scene.fill_shape(
                rect,
                GpuFill::Blur {
                    color: Color::BLACK.with_alpha(0.9),
                    radius: 50.0,
                },
            );
            gpu_scene.fill_shape(rect, GpuFill::Solid(Color::RED));
            gpu_scene.fill_shape(
                path,
                GpuFill::LinearGradient {
                    start: Vec2f { x: 100.0, y: 100.0 },
                    end: Vec2f { x: 800.0, y: 800.0 },
                    color_stops: ColorMap::new([]),
                },
            );
            gpu_scene.fill_shape(
                ellipse,
                GpuFill::Blur {
                    color: Color::RED,
                    radius: 30.0,
                },
            );

            gpu_scene.fill_shape(
                rounded_rect,
                GpuFill::Blur {
                    color: Color::BLACK.with_alpha(0.6),
                    radius: 50.0,
                },
            );
            gpu_scene.fill_shape(
                rounded_rect,
                GpuFill::Solid(Color::CHAMOISEE.with_alpha(0.7)),
            );
        }

        let shapes_data_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Shapes data buffer"),
            contents: bytemuck::cast_slice(gpu_scene.shape_data.as_slice()),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let fill_ops_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("FillOps buffer"),
            contents: bytemuck::cast_slice(gpu_scene.fill_ops.as_slice()),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
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

        let blit_program = BlitProgram::new(&device, format);
        let blit_bind_group = blit_program.create_bind_group(&device, &tex_view, &output_sampler);

        let render_tiles_program = RenderTilesProgram::new(&device);
        let render_tiles_bind_group0 =
            render_tiles_program.create_bind_group0(&device, &tex_view, &params_buffer);
        let render_tiles_bind_group1 =
            render_tiles_program.create_bind_group1(&device, &shapes_data_buffer, &fill_ops_buffer);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            size,
            surface_format: format,
            blit_program,
            blit_bind_group,
            render_tiles_program,
            render_tiles_bind_group0,
            render_tiles_bind_group1,
            output_texture,
            output_sampler,
            params_buffer,
            shapes_data_buffer,
            fill_ops_buffer,
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
            self.blit_bind_group = self.blit_program.create_bind_group(
                &self.device,
                &texture_view,
                &self.output_sampler,
            );
            self.render_tiles_bind_group0 = self.render_tiles_program.create_bind_group0(
                &self.device,
                &texture_view,
                &self.params_buffer,
            );
            self.queue.write_buffer(
                &self.params_buffer,
                0,
                bytemuck::bytes_of(&Params { height, width }),
            );
        }
    }

    pub fn render_tiles_workgroup_count(&self) -> Size<u32> {
        self.size.map(|x| (x.0 as u32).div_ceil(TILE_SIZE))
    }
}

pub struct BlitProgram {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

impl BlitProgram {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
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
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
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

        Self {
            pipeline,
            bind_group_layout,
        }
    }

    pub fn create_bind_group(
        &self,
        device: &wgpu::Device,
        tex_view: &wgpu::TextureView,
        sampler: &wgpu::Sampler,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Blit bind group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(tex_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        })
    }
}

pub struct RenderTilesProgram {
    pub pipeline: wgpu::ComputePipeline,
    pub bind_group_layout0: wgpu::BindGroupLayout,
    pub bind_group_layout1: wgpu::BindGroupLayout,
}

impl RenderTilesProgram {
    pub fn new(device: &wgpu::Device) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("shaders/render_tiles.wgsl"));
        let bind_group_layout0 =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("render_tiles bind group layout0"),
                entries: &[
                    // Params
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
        let bind_group_layout1 =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("render_tiles bind group layout1"),
                entries: &[
                    // Shape data
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: Some(
                                NonZero::new(std::mem::size_of::<f32>() as _).unwrap(),
                            ),
                        },
                        count: None,
                    },
                    // Fills
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: Some(NonZero::new(8).unwrap()),
                        },
                        count: None,
                    },
                ],
            });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render tiles layout"),
            bind_group_layouts: &[&bind_group_layout0, &bind_group_layout1],
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

        Self {
            pipeline,
            bind_group_layout0,
            bind_group_layout1,
        }
    }

    fn create_bind_group0(
        &self,
        device: &wgpu::Device,
        tex_view: &wgpu::TextureView,
        params_buffer: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Render tiles bind group"),
            layout: &self.bind_group_layout0,
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

    fn create_bind_group1(
        &self,
        device: &wgpu::Device,
        shapes_data_buffer: &wgpu::Buffer,
        fill_ops_buffer: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Render tiles bind group"),
            layout: &self.bind_group_layout1,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: shapes_data_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: fill_ops_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
        })
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
