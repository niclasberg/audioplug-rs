use windows::{
    core::Result,
    Win32::{
        Foundation::{HWND, RECT},
        Graphics::Direct2D,
        UI::WindowsAndMessaging::GetClientRect,
    },
};

use super::{com::direct2d_factory, geometry::NativeGeometry, NativeImage, TextLayout};
use crate::{
    app::BrushRef,
    core::{Color, Ellipse, Point, Rectangle, RoundedRectangle, Transform},
};
use std::mem::MaybeUninit;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct RendererGeneration(pub(super) usize);

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
    render_target: Direct2D::ID2D1HwndRenderTarget,
    brush: Direct2D::ID2D1SolidColorBrush,
    saved_states: Vec<SavedState>,
    generation: RendererGeneration,
}

impl Renderer {
    pub fn new(hwnd: HWND) -> Result<Self> {
        let mut rect: RECT = RECT::default();
        unsafe { GetClientRect(hwnd, &mut rect) }?;

        let size = Direct2D::Common::D2D_SIZE_U {
            width: (rect.right - rect.left) as u32,
            height: (rect.bottom - rect.top) as u32,
        };
        let hwnd_render_target_properies = Direct2D::D2D1_HWND_RENDER_TARGET_PROPERTIES {
            hwnd,
            pixelSize: size,
            ..Default::default()
        };
        let render_target_properties = Direct2D::D2D1_RENDER_TARGET_PROPERTIES::default();

        let render_target = unsafe {
            direct2d_factory().CreateHwndRenderTarget(
                &render_target_properties as *const _,
                &hwnd_render_target_properies as *const _,
            )?
        };

        let brush = unsafe { render_target.CreateSolidColorBrush(&Color::GREEN.into(), None)? };

        Ok(Renderer {
            render_target,
            brush,
            saved_states: Vec::new(),
            generation: RendererGeneration(0),
        })
    }

    pub fn resize(&self, width: u32, height: u32) -> Result<()> {
        let new_size = Direct2D::Common::D2D_SIZE_U { width, height };
        unsafe { self.render_target.Resize(&new_size) }
    }

    pub fn begin_draw(&self) {
        unsafe { self.render_target.BeginDraw() };
    }

    pub fn end_draw(&self) -> Result<()> {
        unsafe { self.render_target.EndDraw(None, None) }
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

    pub fn draw_rectangle(&mut self, rect: Rectangle, brush: BrushRef, line_width: f32) {
        self.use_brush(rect, brush, |render_target, brush| unsafe {
            render_target.DrawRectangle(&rect.into(), brush, line_width, None)
        });
    }

    #[inline(always)]
    fn use_brush(
        &mut self,
        rect: Rectangle,
        brush: BrushRef,
        f: impl FnOnce(&Direct2D::ID2D1HwndRenderTarget, &Direct2D::ID2D1Brush),
    ) {
        match brush {
            BrushRef::Solid(color) => unsafe {
                self.brush.SetColor(&color.into());
                f(&self.render_target, &self.brush);
            },
            BrushRef::LinearGradient(linear_gradient) => linear_gradient
                .0
                .use_brush(
                    &self.render_target,
                    self.generation,
                    rect,
                    move |render_target, brush| f(render_target, brush),
                )
                .unwrap(),
        }
    }

    pub fn fill_rectangle(&mut self, rect: Rectangle, brush: BrushRef) {
        self.use_brush(rect, brush, |render_target, brush| unsafe {
            render_target.FillRectangle(&rect.into(), brush)
        });
    }

    pub fn draw_rounded_rectangle(
        &mut self,
        rect: RoundedRectangle,
        brush: BrushRef,
        line_width: f32,
    ) {
        self.use_brush(rect.rect, brush, |render_target, brush| unsafe {
            render_target.DrawRoundedRectangle(&rect.into(), brush, line_width, None);
        });
    }

    pub fn fill_rounded_rectangle(&mut self, rect: RoundedRectangle, brush: BrushRef) {
        self.use_brush(rect.rect, brush, |render_target, brush| unsafe {
            render_target.FillRoundedRectangle(&rect.into(), brush);
        });
    }

    pub fn draw_ellipse(&mut self, ellipse: Ellipse, brush: BrushRef, line_width: f32) {
        self.use_brush(ellipse.bounds(), brush, |render_target, brush| unsafe {
            render_target.DrawEllipse(&ellipse.into(), brush, line_width, None);
        });
    }

    pub fn fill_ellipse(&mut self, ellipse: Ellipse, brush: BrushRef) {
        self.use_brush(ellipse.bounds(), brush, |render_target, brush| unsafe {
            render_target.FillEllipse(&ellipse.into(), brush);
        });
    }

    pub fn draw_geometry(&mut self, geometry: &NativeGeometry, brush: BrushRef, line_width: f32) {
        let bounds =
            unsafe { geometry.0.GetBounds(None) }.expect("Getting transform should not fail");
        self.use_brush(bounds.into(), brush, |render_target, brush| unsafe {
            render_target.DrawGeometry(&geometry.0, brush, line_width, None);
        });
    }

    pub fn fill_geometry(&mut self, geometry: &NativeGeometry, brush: BrushRef) {
        let bounds =
            unsafe { geometry.0.GetBounds(None) }.expect("Getting transform should not fail");
        self.use_brush(bounds.into(), brush, |render_target, brush| unsafe {
            render_target.FillGeometry(&geometry.0, brush, None);
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

    pub fn draw_bitmap(&mut self, source: &NativeImage, rect: Rectangle) {
        source.draw(&self.render_target, self.generation, rect.into())
    }
}
