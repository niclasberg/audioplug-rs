use bytemuck::{bytes_of, cast_slice};
use windows::{
    core::*,
    Win32::{
        Foundation::{HMODULE, HWND},
        Graphics::{
            Direct2D::{self, ID2D1Brush},
            Direct3D, Direct3D11, Dxgi,
        },
    },
};

use super::{
    com::direct2d_factory,
    filters::{
        set_effect_property_f32, CustomEffect, EffectWrapper, RoundedRectShadowEffectShader,
    },
    util::get_scale_factor_for_window,
    Bitmap, TextLayout,
};
use crate::core::{
    Color, Ellipse, Point, Rectangle, RoundedRectangle, ShadowOptions, Size, SpringPhysics,
    Transform, Vec2,
};
use crate::platform::{BrushRef, ShapeRef};
use std::{
    cell::{Ref, RefCell},
    mem::MaybeUninit,
    rc::Rc,
};

impl From<Color> for Direct2D::Common::D2D1_COLOR_F {
    fn from(val: Color) -> Self {
        Direct2D::Common::D2D1_COLOR_F {
            r: val.r,
            g: val.g,
            b: val.b,
            a: val.a,
        }
    }
}

impl From<Rectangle> for Direct2D::Common::D2D_RECT_F {
    fn from(val: Rectangle) -> Self {
        Direct2D::Common::D2D_RECT_F {
            left: val.left() as f32,
            top: val.top() as f32,
            right: val.right() as f32,
            bottom: val.bottom() as f32,
        }
    }
}

impl From<Direct2D::Common::D2D_RECT_F> for Rectangle {
    fn from(value: Direct2D::Common::D2D_RECT_F) -> Self {
        Self::from_ltrb(
            value.left.into(),
            value.top.into(),
            value.right.into(),
            value.bottom.into(),
        )
    }
}

impl From<RoundedRectangle> for Direct2D::D2D1_ROUNDED_RECT {
    fn from(val: RoundedRectangle) -> Self {
        Direct2D::D2D1_ROUNDED_RECT {
            rect: val.rect.into(),
            radiusX: val.corner_radius.width as f32,
            radiusY: val.corner_radius.height as f32,
        }
    }
}

impl From<Ellipse> for Direct2D::D2D1_ELLIPSE {
    fn from(val: Ellipse) -> Self {
        Direct2D::D2D1_ELLIPSE {
            point: val.center.into(),
            radiusX: val.radii.width as f32,
            radiusY: val.radii.height as f32,
        }
    }
}

impl From<Point> for windows_numerics::Vector2 {
    fn from(val: Point) -> Self {
        windows_numerics::Vector2 {
            X: val.x as f32,
            Y: val.y as f32,
        }
    }
}

impl From<Vec2> for windows_numerics::Vector2 {
    fn from(val: Vec2) -> Self {
        windows_numerics::Vector2 {
            X: val.x as f32,
            Y: val.y as f32,
        }
    }
}

impl From<Transform> for windows_numerics::Matrix3x2 {
    fn from(val: Transform) -> Self {
        windows_numerics::Matrix3x2 {
            M11: val.m11 as f32,
            M12: val.m12 as f32,
            M21: val.m21 as f32,
            M22: val.m22 as f32,
            M31: val.tx as f32,
            M32: val.ty as f32,
        }
    }
}

impl From<windows_numerics::Matrix3x2> for Transform {
    fn from(value: windows_numerics::Matrix3x2) -> Self {
        Transform {
            m11: value.M11.into(),
            m12: value.M12.into(),
            m21: value.M21.into(),
            m22: value.M22.into(),
            tx: value.M31.into(),
            ty: value.M32.into(),
        }
    }
}

impl From<Size<u32>> for Direct2D::Common::D2D_SIZE_U {
    fn from(value: Size<u32>) -> Self {
        Direct2D::Common::D2D_SIZE_U {
            width: value.width,
            height: value.height,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct RendererGeneration(pub(super) usize);
impl RendererGeneration {
    pub fn next(&self) -> Self {
        Self(self.0 + 1)
    }
}

pub type RendererRef<'a> = &'a mut Renderer;

enum SavedAction {
    Clip,
    Layer,
}

struct SavedState {
    transform: windows_numerics::Matrix3x2,
    actions: Vec<SavedAction>,
}
pub struct Renderer {
    render_target: Direct2D::ID2D1DeviceContext,
    swapchain: Dxgi::IDXGISwapChain1,
    brush: Direct2D::ID2D1SolidColorBrush,
    saved_states: Vec<SavedState>,
    generation: RendererGeneration,
    scale_factor: f32,
}

impl Renderer {
    pub fn new(hwnd: HWND) -> Result<Self> {
        let scale_factor = get_scale_factor_for_window(hwnd) as f32;

        let device = unsafe {
            let mut device = None;
            Direct3D11::D3D11CreateDevice(
                None,
                Direct3D::D3D_DRIVER_TYPE_HARDWARE,
                HMODULE::default(),
                Direct3D11::D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                None,
                Direct3D11::D3D11_SDK_VERSION,
                Some(&mut device),
                None,
                None,
            )?;
            device.unwrap()
        };

        let dxgi_device = device.cast::<Dxgi::IDXGIDevice>()?;
        let direct2d_device = unsafe { direct2d_factory().CreateDevice(&dxgi_device) }?;
        let render_target = unsafe {
            direct2d_device.CreateDeviceContext(Direct2D::D2D1_DEVICE_CONTEXT_OPTIONS_NONE)
        }?;
        unsafe { render_target.SetDpi(96.0 * scale_factor, 96.0 * scale_factor) };

        let dxgi_factory: Dxgi::IDXGIFactory2 = unsafe { dxgi_device.GetAdapter()?.GetParent() }?;
        let swapchain = unsafe {
            dxgi_factory.CreateSwapChainForHwnd(
                &device,
                hwnd,
                &Dxgi::DXGI_SWAP_CHAIN_DESC1 {
                    Width: 0,  // Automatic scaling
                    Height: 0, // Automatic scaling
                    Format: Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM,
                    Stereo: false.into(),
                    SampleDesc: Dxgi::Common::DXGI_SAMPLE_DESC {
                        Count: 1, // don't use multi-sampling
                        Quality: 0,
                    },
                    BufferUsage: Dxgi::DXGI_USAGE_RENDER_TARGET_OUTPUT,
                    BufferCount: 2, // Double buffering
                    Scaling: Dxgi::DXGI_SCALING_NONE,
                    SwapEffect: Dxgi::DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL,
                    AlphaMode: Dxgi::Common::DXGI_ALPHA_MODE_IGNORE,
                    ..Default::default()
                },
                None,
                None,
            )
        }?;

        bind_swapchain_bitmap_to_render_target(&render_target, &swapchain, scale_factor)?;

        let brush = unsafe { render_target.CreateSolidColorBrush(&Color::GREEN.into(), None)? };

        EffectWrapper::<RoundedRectShadowEffectShader>::register(direct2d_factory())?;
        let rounded_shadow_effect =
            unsafe { render_target.CreateEffect(&RoundedRectShadowEffectShader::EFFECT_GUID) }?;

        Ok(Renderer {
            render_target,
            swapchain,
            brush,
            saved_states: Vec::new(),
            generation: RendererGeneration(0),
            scale_factor,
        })
    }

    pub fn resize(&self, width: u32, height: u32) -> Result<()> {
        // We need to clear all references to the swapchain buffers before resizing.
        unsafe { self.render_target.SetTarget(None) };

        unsafe {
            self.swapchain.ResizeBuffers(
                0,
                width,
                height,
                Dxgi::Common::DXGI_FORMAT_UNKNOWN,
                Dxgi::DXGI_SWAP_CHAIN_FLAG(0),
            )
        }?;

        bind_swapchain_bitmap_to_render_target(
            &self.render_target,
            &self.swapchain,
            self.scale_factor,
        )?;

        Ok(())
    }

    pub fn update_scale_factor(&mut self, scale_factor: f32) -> Result<()> {
        unsafe {
            self.render_target
                .SetDpi(96.0 * scale_factor, 96.0 * scale_factor)
        };
        unsafe { self.render_target.SetTarget(None) };
        bind_swapchain_bitmap_to_render_target(&self.render_target, &self.swapchain, scale_factor)?;
        self.scale_factor = scale_factor;
        Ok(())
    }

    pub fn begin_draw(&self) {
        unsafe { self.render_target.BeginDraw() };
    }

    pub fn end_draw(&self) -> Result<()> {
        unsafe { self.render_target.EndDraw(None, None) }?;
        unsafe { self.swapchain.Present(1, Dxgi::DXGI_PRESENT(0)) }.ok()
    }

    pub fn clear(&self, color: Color) {
        unsafe { self.render_target.Clear(Some(&color.into())) };
    }

    pub fn get_transform(&mut self) -> Transform {
        self.get_d2d1_transform().into()
    }

    fn get_d2d1_transform(&self) -> windows_numerics::Matrix3x2 {
        let mut transform = MaybeUninit::uninit();
        unsafe {
            self.render_target.GetTransform(transform.as_mut_ptr());
            transform.assume_init()
        }
    }

    pub fn transform(&mut self, transform: Transform) {
        let new_transform = (transform * self.get_transform()).into();
        unsafe {
            self.render_target.SetTransform(&new_transform);
        }
    }

    pub fn clip(&mut self, bounds: Rectangle) {
        unsafe {
            self.render_target
                .PushAxisAlignedClip(&bounds.into(), Direct2D::D2D1_ANTIALIAS_MODE_PER_PRIMITIVE)
        };
        if let Some(saved_state) = self.saved_states.last_mut() {
            saved_state.actions.push(SavedAction::Clip);
        }
    }

    pub fn save(&mut self) {
        let saved_state = SavedState {
            transform: self.get_d2d1_transform(),
            actions: Vec::new(),
        };
        self.saved_states.push(saved_state);
    }

    pub fn restore(&mut self) {
        if let Some(saved_state) = self.saved_states.pop() {
            unsafe { self.render_target.SetTransform(&saved_state.transform) }
            for action in saved_state.actions {
                match action {
                    SavedAction::Clip => unsafe { self.render_target.PopAxisAlignedClip() },
                    SavedAction::Layer => unsafe { self.render_target.PopLayer() },
                }
            }
        }
    }

    pub fn draw_line(&mut self, p0: Point, p1: Point, brush: BrushRef, line_width: f32) {
        let bounds = Rectangle::from_points(p0, p1);
        self.use_brush(bounds, brush, |render_target, brush| unsafe {
            render_target.DrawLine(p0.into(), p1.into(), brush, line_width, None);
        });
    }

    #[inline(always)]
    fn use_brush(
        &mut self,
        rect: Rectangle,
        brush: BrushRef,
        f: impl FnOnce(&Direct2D::ID2D1RenderTarget, &Direct2D::ID2D1Brush),
    ) {
        match brush {
            BrushRef::Solid(color) => unsafe {
                self.brush.SetColor(&color.into());
                f(&self.render_target, &self.brush);
            },
            BrushRef::LinearGradient(linear_gradient) => {
                let start = linear_gradient.start.resolve(rect);
                let end = linear_gradient.end.resolve(rect);
                linear_gradient
                    .native
                    .use_brush(
                        &self.render_target,
                        self.generation,
                        start,
                        end,
                        move |render_target, brush| f(render_target, brush),
                    )
                    .unwrap()
            }
        }
    }

    pub fn draw_shadow(&mut self, shape: ShapeRef, options: ShadowOptions) {
        let rect = shape.bounds();
        let bitmap_size = (rect.size() + Size::splat(options.radius * 2.0))
            .scale(self.scale_factor as _)
            .expand_to_u32();
        let blur_std_dev = options.radius / 3.0;

        let props = Direct2D::D2D1_BITMAP_PROPERTIES1 {
            pixelFormat: Direct2D::Common::D2D1_PIXEL_FORMAT {
                format: Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM,
                alphaMode: Direct2D::Common::D2D1_ALPHA_MODE_PREMULTIPLIED,
            },
            dpiX: 96.0 * self.scale_factor,
            dpiY: 96.0 * self.scale_factor,
            bitmapOptions: Direct2D::D2D1_BITMAP_OPTIONS_TARGET,
            ..Default::default()
        };

        let Ok(mask_bitmap) = (unsafe {
            self.render_target
                .CreateBitmap(bitmap_size.into(), None, 0, &props)
        }) else {
            return;
        };

        let offset_effect = unsafe {
            self.render_target
                .CreateEffect(&Direct2D::CLSID_D2D12DAffineTransform)
        }
        .unwrap();
        unsafe {
            offset_effect.SetInput(0, &mask_bitmap, false);
            let transform = [
                1.0,
                0.0,
                0.0,
                1.0,
                options.offset.x as f32,
                options.offset.y as f32,
            ];
            offset_effect.SetValue(
                Direct2D::D2D1_2DAFFINETRANSFORM_PROP_TRANSFORM_MATRIX.0 as u32,
                Direct2D::D2D1_PROPERTY_TYPE_MATRIX_3X2,
                bytes_of(&transform),
            )
        }
        .unwrap();

        let blur_effect = unsafe {
            self.render_target
                .CreateEffect(&Direct2D::CLSID_D2D1GaussianBlur)
        }
        .unwrap();
        unsafe {
            blur_effect.SetInput(0, &offset_effect.GetOutput().unwrap(), false);
            set_effect_property_f32(
                &blur_effect,
                Direct2D::D2D1_GAUSSIANBLUR_PROP_STANDARD_DEVIATION.0,
                blur_std_dev as f32,
            )
        }
        .unwrap();

        let colorize_effect = unsafe {
            self.render_target
                .CreateEffect(&Direct2D::CLSID_D2D1ColorMatrix)
        }
        .unwrap();
        unsafe { colorize_effect.SetInput(0, &blur_effect.GetOutput().unwrap(), false) };
        let color_matrix = [
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            options.color.a,
            options.color.r,
            options.color.g,
            options.color.b,
            0.0,
        ];
        unsafe {
            colorize_effect.SetValue(
                Direct2D::D2D1_COLORMATRIX_PROP_COLOR_MATRIX.0 as u32,
                Direct2D::D2D1_PROPERTY_TYPE_MATRIX_5X4,
                cast_slice(&color_matrix),
            )
        }
        .unwrap();
        unsafe {
            colorize_effect.SetValue(
                Direct2D::D2D1_COLORMATRIX_PROP_ALPHA_MODE.0 as u32,
                Direct2D::D2D1_PROPERTY_TYPE_ENUM,
                bytes_of(&Direct2D::Common::D2D1_ALPHA_MODE_PREMULTIPLIED.0),
            )
        }
        .unwrap();

        let composite_effect = unsafe {
            self.render_target
                .CreateEffect(&Direct2D::CLSID_D2D1Composite)
        }
        .unwrap();
        unsafe {
            composite_effect.SetInput(0, &colorize_effect.GetOutput().unwrap(), false);
            composite_effect.SetInput(1, &mask_bitmap, false);
            composite_effect
                .SetValue(
                    Direct2D::D2D1_COMPOSITE_PROP_MODE.0 as u32,
                    Direct2D::D2D1_PROPERTY_TYPE_ENUM,
                    bytes_of(&Direct2D::Common::D2D1_COMPOSITE_MODE_DESTINATION_OUT.0),
                )
                .unwrap();
        }

        let prev_target = unsafe { self.render_target.GetTarget() }.unwrap();
        let prev_transform = self.get_d2d1_transform();
        unsafe {
            self.render_target.SetTarget(&mask_bitmap);
            self.render_target.Clear(Some(&Color::ZERO.into()));

            let transform = Transform::from_translation(
                Vec2::splat(options.radius) - rect.top_left().as_vector(),
            );
            self.render_target.SetTransform(&transform.into());

            self.brush.SetColor(&Color::BLACK.into());
            fill_shape_impl(&self.render_target, shape, &self.brush);

            self.render_target.SetTransform(&prev_transform);
            self.render_target.SetTarget(&prev_target);
        }

        let offset = rect.top_left() - Vec2::splat(options.radius);
        unsafe {
            self.render_target.DrawImage(
                &composite_effect.GetOutput().unwrap(),
                Some(&offset.into()),
                None,
                Direct2D::D2D1_INTERPOLATION_MODE_LINEAR,
                Direct2D::Common::D2D1_COMPOSITE_MODE_SOURCE_OVER,
            )
        };
    }

    pub fn fill_shape(&mut self, shape: ShapeRef, brush: BrushRef) {
        let rect = shape.bounds();
        self.use_brush(rect, brush, |render_target, brush| unsafe {
            fill_shape_impl(render_target, shape, brush)
        });
    }

    pub fn stroke_shape(&mut self, shape: ShapeRef, brush: BrushRef, line_width: f32) {
        let rect = shape.bounds();
        self.use_brush(rect, brush, |render_target, brush| unsafe {
            stroke_shape_impl(render_target, shape, brush, line_width);
        });
    }

    pub fn draw_text(&mut self, text_layout: &TextLayout, position: Point) {
        unsafe {
            self.brush.SetColor(&text_layout.color.into());
            self.render_target.DrawTextLayout(
                position.into(),
                &text_layout.text_layout,
                &self.brush,
                Direct2D::D2D1_DRAW_TEXT_OPTIONS_NONE,
            )
        }
    }

    pub fn draw_bitmap(&mut self, source: &Bitmap, rect: Rectangle) {
        source.draw(&self.render_target, self.generation, rect.into())
    }

    pub fn dirty_rect(&self) -> Rectangle {
        Rectangle::EMPTY
    }
}

fn bind_swapchain_bitmap_to_render_target(
    render_target: &Direct2D::ID2D1DeviceContext,
    swapchain: &Dxgi::IDXGISwapChain1,
    scale_factor: f32,
) -> Result<()> {
    let back_buffer: Dxgi::IDXGISurface = unsafe { swapchain.GetBuffer(0) }?;
    let bitmap_props = Direct2D::D2D1_BITMAP_PROPERTIES1 {
        pixelFormat: Direct2D::Common::D2D1_PIXEL_FORMAT {
            format: Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM,
            alphaMode: Direct2D::Common::D2D1_ALPHA_MODE_PREMULTIPLIED,
        },
        dpiX: scale_factor * 96.0,
        dpiY: scale_factor * 96.0,
        bitmapOptions: Direct2D::D2D1_BITMAP_OPTIONS_TARGET
            | Direct2D::D2D1_BITMAP_OPTIONS_CANNOT_DRAW,
        ..Default::default()
    };

    unsafe {
        let bitmap =
            render_target.CreateBitmapFromDxgiSurface(&back_buffer, Some(&bitmap_props))?;
        render_target.SetTarget(&bitmap);
    };
    Ok(())
}

unsafe fn fill_shape_impl(
    render_target: &Direct2D::ID2D1RenderTarget,
    shape: ShapeRef,
    brush: &ID2D1Brush,
) {
    match shape {
        ShapeRef::Rect(rectangle) => render_target.FillRectangle(&rectangle.into(), brush),
        ShapeRef::Rounded(rounded_rectangle) => {
            render_target.FillRoundedRectangle(&rounded_rectangle.into(), brush)
        }
        ShapeRef::Ellipse(ellipse) => render_target.FillEllipse(&ellipse.into(), brush),
        ShapeRef::Geometry(path_geometry) => {
            render_target.FillGeometry(&path_geometry.0 .0, brush, None)
        }
    }
}

unsafe fn stroke_shape_impl(
    render_target: &Direct2D::ID2D1RenderTarget,
    shape: ShapeRef,
    brush: &ID2D1Brush,
    line_width: f32,
) {
    match shape {
        ShapeRef::Rect(rectangle) => {
            render_target.DrawRectangle(&rectangle.into(), brush, line_width, None)
        }
        ShapeRef::Rounded(rounded_rectangle) => {
            render_target.DrawRoundedRectangle(&rounded_rectangle.into(), brush, line_width, None)
        }
        ShapeRef::Ellipse(ellipse) => {
            render_target.DrawEllipse(&ellipse.into(), brush, line_width, None)
        }
        ShapeRef::Geometry(path_geometry) => {
            render_target.DrawGeometry(&path_geometry.0 .0, brush, line_width, None)
        }
    }
}

#[derive(Clone)]
struct RenderResourceInner<T> {
    generation: RendererGeneration,
    resource: T,
}

/// Utility class that stores a device dependent resource. The current value
/// is tagged by a renderer generation. The generation is stepped whenever the
/// render target is recreated, and in that case we can force a rebuild of the
/// underlying resource.
#[derive(Clone)]
pub struct DeviceResource<T> {
    inner: Rc<RefCell<Option<RenderResourceInner<T>>>>,
}

impl<T> DeviceResource<T> {
    pub fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(None)),
        }
    }

    pub fn get_or_insert(
        &self,
        generation: RendererGeneration,
        f: impl FnOnce() -> Result<T>,
    ) -> Result<Ref<T>> {
        if !self
            .inner
            .borrow()
            .as_ref()
            .is_some_and(|inner| inner.generation == generation)
        {
            let resource = f()?;
            self.inner.replace(Some(RenderResourceInner {
                resource,
                generation,
            }));
        }
        Ok(Ref::map(self.inner.borrow(), |inner| {
            &inner.as_ref().unwrap().resource
        }))
    }
}
