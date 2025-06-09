use crate::core::{ColorMap, Point, Size, Vec2};
use windows::Win32::Graphics::Direct2D;
use windows::{core::Result, Win32::Graphics::Direct2D::Common::D2D1_GRADIENT_STOP};

use super::renderer::{DeviceResource, RendererGeneration};

#[derive(Clone)]
pub struct NativeLinearGradient {
    pub color_map: ColorMap,
    cached_brush: DeviceResource<Direct2D::ID2D1LinearGradientBrush>,
}

impl NativeLinearGradient {
    pub fn new(color_map: ColorMap) -> Self {
        Self {
            color_map,
            cached_brush: DeviceResource::new(),
        }
    }

    pub(super) fn use_brush(
        &self,
        render_target: &Direct2D::ID2D1RenderTarget,
        generation: RendererGeneration,
        start: Point,
        end: Point,
        f: impl FnOnce(&Direct2D::ID2D1RenderTarget, &Direct2D::ID2D1Brush),
    ) -> Result<()> {
        let brush = self.cached_brush.get_or_insert(generation, || {
            let gradient_stops = create_gradient_stop_collection(render_target, &self.color_map)?;

            let gradient_properties = Direct2D::D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES {
                startPoint: start.into(),
                endPoint: end.into(),
            };

            let brush_properties = Direct2D::D2D1_BRUSH_PROPERTIES {
                opacity: 1.0,
                transform: windows_numerics::Matrix3x2::identity(),
            };

            unsafe {
                render_target.CreateLinearGradientBrush(
                    &gradient_properties as *const _,
                    Some(&brush_properties as *const _),
                    &gradient_stops,
                )
            }
        })?;

        unsafe {
            brush.SetStartPoint(start.into());
            brush.SetEndPoint(end.into());
        }

        f(render_target, &brush);

        Ok(())
    }
}

fn create_gradient_stop_collection(
    render_target: &Direct2D::ID2D1RenderTarget,
    color_map: &ColorMap,
) -> Result<Direct2D::ID2D1GradientStopCollection> {
    let stops: Vec<_> = color_map
        .stops
        .iter()
        .map(|stop| D2D1_GRADIENT_STOP {
            position: stop.position,
            color: stop.color.into(),
        })
        .collect();
    unsafe {
        render_target.CreateGradientStopCollection(
            stops.as_slice(),
            Direct2D::D2D1_GAMMA_2_2,
            Direct2D::D2D1_EXTEND_MODE_CLAMP,
        )
    }
}

#[derive(Clone)]
pub struct NativeRadialGradient {
    pub color_map: ColorMap,
    cached_brush: DeviceResource<Direct2D::ID2D1RadialGradientBrush>,
}

impl NativeRadialGradient {
    pub fn new(color_map: ColorMap) -> Self {
        Self {
            color_map,
            cached_brush: DeviceResource::new(),
        }
    }

    pub(super) fn use_brush(
        &self,
        render_target: &Direct2D::ID2D1RenderTarget,
        generation: RendererGeneration,
        center: Point,
        radius: Size,
        f: impl FnOnce(&Direct2D::ID2D1RenderTarget, &Direct2D::ID2D1Brush),
    ) -> Result<()> {
        let brush = self.cached_brush.get_or_insert(generation, move || {
            let gradient_stops = create_gradient_stop_collection(render_target, &self.color_map)?;

            let gradient_properties = Direct2D::D2D1_RADIAL_GRADIENT_BRUSH_PROPERTIES {
                center: center.into(),
                gradientOriginOffset: Vec2::ZERO.into(),
                radiusX: radius.width as _,
                radiusY: radius.height as _,
            };

            let brush_properties = Direct2D::D2D1_BRUSH_PROPERTIES {
                opacity: 1.0,
                transform: windows_numerics::Matrix3x2::identity(),
            };

            unsafe {
                render_target.CreateRadialGradientBrush(
                    &gradient_properties as *const _,
                    Some(&brush_properties as *const _),
                    &gradient_stops,
                )
            }
        })?;

        unsafe {
            brush.SetCenter(center.into());
            brush.SetRadiusX(radius.width as _);
            brush.SetRadiusY(radius.height as _);
        }

        f(render_target, &brush);

        Ok(())
    }
}
