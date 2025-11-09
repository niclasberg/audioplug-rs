pub struct GradientRef {
    offset: u32,
    size: u32,
}

pub struct GradientAtlas {
    texture: wgpu::Texture,
    allocator: GradientAllocator,
}

impl GradientAtlas {
    pub fn new(
        device: &wgpu::Device,
        width: u32,
        max_gradients: u32,
        format: wgpu::TextureFormat,
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Gradient atlas texture"),
            size: wgpu::Extent3d {
                width,
                height: max_gradients,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let allocator = GradientAllocator::new(max_gradients);
        Self { texture, allocator }
    }
}

struct GradientAllocator {
    max_rows: u32,
    next_row: u32,
    freed_rows: Vec<u32>,
}

impl GradientAllocator {
    pub fn new(max_rows: u32) -> Self {
        Self {
            max_rows,
            next_row: 0,
            freed_rows: Vec::new(),
        }
    }

    pub fn allocate(&mut self) -> Option<u32> {
        if let Some(row) = self.freed_rows.pop() {
            Some(row)
        } else if self.next_row < self.max_rows {
            let row = self.next_row;
            self.next_row += 1;
            Some(row)
        } else {
            None
        }
    }

    pub fn free(&mut self, row: u32) {
        assert!(row < self.next_row);
        self.freed_rows.push(row);
    }
}
