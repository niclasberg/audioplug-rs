use windows::core::Result;
use windows::Win32::Graphics::Direct2D;

use super::renderer::{DeviceResource, RendererGeneration};

pub struct Image {
    bitmap: DeviceResource<Direct2D::ID2D1Bitmap>,
}

pub struct GaussianBlur {
    filter: DeviceResource<Direct2D::ID2D1Effect>,
    radius: f64,
}

impl GaussianBlur {
    pub fn new(radius: f64) -> Self {
        Self {
            filter: DeviceResource::new(),
            radius,
        }
    }

    pub fn set_radius(&mut self, radius: f64) {
        self.radius = radius;
    }

    pub fn with_output(
        &self,
        render_target: &Direct2D::ID2D1DeviceContext,
        generation: RendererGeneration,
        f: impl FnOnce(&Direct2D::ID2D1DeviceContext, &Direct2D::ID2D1Image),
    ) -> Result<()> {
        let filter = self.filter.get_or_insert(generation, || unsafe {
            render_target.CreateEffect(&Direct2D::CLSID_D2D1GaussianBlur)
        })?;

        unsafe {
            set_effect_property_f32(
                &filter,
                Direct2D::D2D1_GAUSSIANBLUR_PROP_STANDARD_DEVIATION.0,
                self.radius as _,
            )
        }?;

        f(render_target, &unsafe { filter.GetOutput() }?);

        Ok(())
        //self.filter.get_or_insert(generation, || {})
    }
}

pub struct DropShadow {}

impl DropShadow {}

pub unsafe fn set_effect_property_f32(
    properties: &Direct2D::ID2D1Properties,
    index: i32,
    value: f32,
) -> Result<()> {
    properties.SetValue(
        index as u32,
        Direct2D::D2D1_PROPERTY_TYPE_FLOAT,
        value.to_ne_bytes().as_slice(),
    )
}

pub unsafe fn set_effect_property_u32(
    properties: &Direct2D::ID2D1Properties,
    index: i32,
    value: u32,
) -> Result<()> {
    properties.SetValue(
        index as u32,
        Direct2D::D2D1_PROPERTY_TYPE_UINT32,
        value.to_ne_bytes().as_slice(),
    )
}

pub fn set_effect_property_i32(
    properties: &Direct2D::ID2D1Properties,
    index: i32,
    value: i32,
) -> Result<()> {
    unsafe {
        properties.SetValue(
            index as u32,
            Direct2D::D2D1_PROPERTY_TYPE_INT32,
            value.to_ne_bytes().as_slice(),
        )
    }
}
