use windows::core::Result;
use windows::Win32::Graphics::Direct2D;

use super::renderer::{DeviceDependentResource, RendererGeneration};

pub struct GaussianBlur {
    filter: DeviceDependentResource<Direct2D::ID2D1Effect>,
    radius: f64,
}

impl GaussianBlur {
    pub fn new(radius: f64) -> Self {
        Self {
            filter: DeviceDependentResource::new(),
            radius,
        }
    }

    pub fn apply(
        &self,
        render_target: &Direct2D::ID2D1DeviceContext,
        generation: RendererGeneration,
    ) -> Result<()> {
        let filter = self.filter.get_or_insert(generation, || unsafe {
            render_target.CreateEffect(&Direct2D::CLSID_D2D1GaussianBlur)
        })?;

        set_property_f32(
            &filter,
            Direct2D::D2D1_GAUSSIANBLUR_PROP_STANDARD_DEVIATION.0,
            self.radius as _,
        )?;

        Ok(())
        //self.filter.get_or_insert(generation, || {})
    }
}

pub struct DropShadow {}

impl DropShadow {}

fn set_property_f32(properties: &Direct2D::ID2D1Properties, index: i32, value: f32) -> Result<()> {
    unsafe {
        properties.SetValue(
            index as u32,
            Direct2D::D2D1_PROPERTY_TYPE_FLOAT,
            value.to_ne_bytes().as_slice(),
        )
    }
}
