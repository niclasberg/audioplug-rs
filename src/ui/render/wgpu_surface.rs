use std::num::NonZero;

use bytemuck::{Pod, Zeroable};
use thiserror::Error;
use wgpu::util::DeviceExt;

use super::tiles::TILE_SIZE;
use crate::{
    core::{PhysicalCoord, PhysicalSize, Size, Zero},
    ui::render::gpu_scene::GpuScene,
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

pub struct SurfaceState {
    pub blit_bind_group: wgpu::BindGroup,
    pub render_tiles_bind_group0: wgpu::BindGroup,
    pub output_texture: wgpu::Texture,
    pub params_buffer: wgpu::Buffer,
    pub output_sampler: wgpu::Sampler,
    pub last_size: PhysicalSize,
}

impl SurfaceState {
    pub fn new(
        device: &wgpu::Device,
        blit_program: &BlitProgram,
        render_tiles_program: &RenderTilesProgram,
        size: PhysicalSize,
    ) -> Self {
        let width = size.width.0 as _;
        let height = size.height.0 as _;

        let params = Params { width, height };
        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Params buffer"),
            contents: bytemuck::bytes_of(&params),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let output_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Output texture sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        // Output texture for compute shader
        let output_texture = create_output_texture(device, width, height);
        let tex_view = output_texture.create_view(&Default::default());
        let blit_bind_group = blit_program.create_bind_group(device, &tex_view, &output_sampler);

        let render_tiles_bind_group0 =
            render_tiles_program.create_bind_group0(device, &tex_view, &params_buffer);

        SurfaceState {
            blit_bind_group,
            render_tiles_bind_group0,
            output_texture,
            params_buffer,
            output_sampler,
            last_size: size,
        }
    }

    pub fn resize_if_needed(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        blit_program: &BlitProgram,
        render_tiles_program: &RenderTilesProgram,
        size: PhysicalSize,
    ) {
        if size != self.last_size {
            let width = size.width.0 as _;
            let height = size.height.0 as _;

            self.output_texture = create_output_texture(device, width, height);
            let texture_view = self
                .output_texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            self.blit_bind_group =
                blit_program.create_bind_group(device, &texture_view, &self.output_sampler);
            self.render_tiles_bind_group0 =
                render_tiles_program.create_bind_group0(device, &texture_view, &self.params_buffer);
            queue.write_buffer(
                &self.params_buffer,
                0,
                bytemuck::bytes_of(&Params { height, width }),
            );
        }
    }
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
    // Render tiles pipeline
    pub render_tiles_program: RenderTilesProgram,
    pub render_tiles_bind_group1: wgpu::BindGroup,
    pub shapes_data_buffer: wgpu::Buffer,
    pub fill_ops_buffer: wgpu::Buffer,
    pub state: Option<SurfaceState>,
    pub is_configured: bool,
}

impl WGPUSurface {
    pub async fn new(
        instance: &wgpu::Instance,
        handle: &crate::platform::Handle,
    ) -> Result<Self, GraphicsInitError> {
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
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
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

        let shapes_data_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Shapes data buffer"),
            contents: bytemuck::cast_slice(&[0]),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let fill_ops_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("FillOps buffer"),
            contents: bytemuck::cast_slice(&GpuScene::NOOP_FILL),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let blit_program = BlitProgram::new(&device, format);
        let render_tiles_program = RenderTilesProgram::new(&device);
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
            render_tiles_program,
            render_tiles_bind_group1,
            shapes_data_buffer,
            fill_ops_buffer,
            state: None,
            is_configured: false,
        })
    }

    pub fn configure_if_needed(&mut self, new_size: PhysicalSize) {
        if new_size.height > PhysicalCoord::ZERO && new_size.width > PhysicalCoord::ZERO {
            let Self {
                device,
                queue,
                blit_program,
                render_tiles_program,
                ..
            } = self;

            if !self.is_configured || self.size != new_size {
                self.size = new_size;
                self.config.width = new_size.width.0 as _;
                self.config.height = new_size.height.0 as _;

                self.surface.configure(device, &self.config);
                self.is_configured = true;
            }

            let state = self.state.get_or_insert_with(|| {
                SurfaceState::new(device, blit_program, render_tiles_program, new_size)
            });
            state.resize_if_needed(device, queue, blit_program, render_tiles_program, new_size);
        }
    }

    pub fn upload_scene(&mut self, scene: &GpuScene) {
        let fill_ops_recreated = {
            let fill_ops = if scene.fill_ops.is_empty() {
                &[0]
            } else {
                bytemuck::cast_slice(scene.fill_ops.as_slice())
            };

            update_buffer(
                &mut self.device,
                &mut self.queue,
                &mut self.fill_ops_buffer,
                fill_ops,
                "FillOps buffer",
            )
        };

        let shape_data_recreated = {
            let shape_data = if scene.shape_data.is_empty() {
                bytemuck::cast_slice(&GpuScene::NOOP_FILL)
            } else {
                bytemuck::cast_slice(scene.shape_data.as_slice())
            };
            update_buffer(
                &mut self.device,
                &mut self.queue,
                &mut self.shapes_data_buffer,
                shape_data,
                "ShapeData buffer",
            )
        };

        if shape_data_recreated || fill_ops_recreated {
            self.render_tiles_bind_group1 = self.render_tiles_program.create_bind_group1(
                &self.device,
                &self.shapes_data_buffer,
                &self.fill_ops_buffer,
            );
        }
    }

    pub fn resized(&mut self) {
        self.is_configured = false;
    }

    pub fn render_tiles_workgroup_count(&self) -> Size<u32> {
        self.size.map(|x| (x.0 as u32).div_ceil(TILE_SIZE))
    }
}

/// Update or reallocate a buffer and fill with the provided data
/// Returns true if a new buffer was created
fn update_buffer(
    device: &mut wgpu::Device,
    queue: &mut wgpu::Queue,
    buffer: &mut wgpu::Buffer,
    data: &[u8],
    label: &str,
) -> bool {
    if data.len() < buffer.size() as usize {
        queue.write_buffer(buffer, 0, data);
        false
    } else {
        *buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: data,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });
        true
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
                immediate_size: 0,
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
            cache: None,
            multiview_mask: None,
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
            immediate_size: 0,
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
