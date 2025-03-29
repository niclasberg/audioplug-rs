use std::cell::RefCell;

use crate::core::{ColorMap, Rectangle, UnitPoint};
use windows::Win32::Graphics::Direct2D;
use windows::{core::Result, Win32::Graphics::Direct2D::Common::D2D1_GRADIENT_STOP};

use super::renderer::RendererGeneration;

struct CachedBrush<T> {
    brush: T,
    generation: RendererGeneration,
}

pub struct NativeLinearGradient {
    pub color_map: ColorMap,
    pub start: UnitPoint,
    pub end: UnitPoint,
    cached_brush: RefCell<Option<CachedBrush<Direct2D::ID2D1LinearGradientBrush>>>,
}

impl NativeLinearGradient {
    pub fn new(color_map: ColorMap, start: UnitPoint, end: UnitPoint) -> Self {
        Self {
            color_map,
            start,
            end,
            cached_brush: RefCell::new(None),
        }
    }

    pub(super) fn use_brush(
        &self,
        render_target: &Direct2D::ID2D1HwndRenderTarget,
        generation: RendererGeneration,
        bounds: Rectangle,
        f: impl FnOnce(&Direct2D::ID2D1HwndRenderTarget, &Direct2D::ID2D1Brush),
    ) -> Result<()> {
        if !self
            .cached_brush
            .borrow()
            .as_ref()
            .is_some_and(|cached_brush| cached_brush.generation == generation)
        {
            let gradient_stops = create_gradient_stop_collection(render_target, &self.color_map)?;

            let gradient_properties = Direct2D::D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES {
                startPoint: self.start.resolve(bounds).into(),
                endPoint: self.end.resolve(bounds).into(),
            };

            let brush_properties = Direct2D::D2D1_BRUSH_PROPERTIES {
                opacity: 1.0,
                transform: windows_numerics::Matrix3x2::identity(),
            };

            let brush = unsafe {
                render_target.CreateLinearGradientBrush(
                    &gradient_properties as *const _,
                    Some(&brush_properties as *const _),
                    &gradient_stops,
                )
            }?;

            self.cached_brush
                .replace(Some(CachedBrush { brush, generation }));

            f(
                render_target,
                &self.cached_brush.borrow().as_ref().unwrap().brush,
            )
        } else {
            let cached_brush = self.cached_brush.borrow();
            let brush = &cached_brush.as_ref().unwrap().brush;
            unsafe { brush.SetStartPoint(self.start.resolve(bounds).into()) };
            unsafe { brush.SetEndPoint(self.end.resolve(bounds).into()) };
            f(render_target, &brush)
        }

        Ok(())
    }
}

fn create_gradient_stop_collection(
    render_target: &Direct2D::ID2D1HwndRenderTarget,
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

pub struct NativeRadialGradient {
    pub color_map: ColorMap,
    pub start: UnitPoint,
    pub end: UnitPoint,
    cached_brush: RefCell<Option<CachedBrush<Direct2D::ID2D1RadialGradientBrush>>>,
}
