use windows::{core::Result, Foundation::Numerics::Matrix3x2, Win32::{Foundation::{HWND, RECT}, Graphics::Direct2D, UI::WindowsAndMessaging::GetClientRect}};

use crate::core::{Rectangle, Color, Size, Transform, Point};
use std::mem::MaybeUninit;
use super::{com::direct2d_factory, ImageSource, TextLayout};

impl Into<Direct2D::Common::D2D1_COLOR_F> for Color {
    fn into(self) -> Direct2D::Common::D2D1_COLOR_F {
        Direct2D::Common::D2D1_COLOR_F {
            r: self.r,
            g: self.g,
            b: self.b,
            a: self.a
        }
    }
}

impl Into<Direct2D::Common::D2D_RECT_F> for Rectangle {
    fn into(self) -> Direct2D::Common::D2D_RECT_F {
        Direct2D::Common::D2D_RECT_F {
            left: self.left() as f32,
            top: self.top() as f32,
            right: self.right() as f32,
            bottom: self.bottom() as f32
        }
    }
}

impl Into<Direct2D::Common::D2D_POINT_2F> for Point {
    fn into(self) -> Direct2D::Common::D2D_POINT_2F {
        Direct2D::Common::D2D_POINT_2F {
            x: self.x as f32,
            y: self.y as f32
        }
    }
}

impl Into<Matrix3x2> for Transform {
    fn into(self) -> Matrix3x2 {
        Matrix3x2 { 
            M11: self.m11 as f32, 
            M12: self.m12 as f32, 
            M21: self.m21 as f32, 
            M22: self.m22 as f32, 
            M31: self.tx as f32, 
            M32: self.ty as f32 
        }
    }
}

impl From<Matrix3x2> for Transform {
    fn from(value: Matrix3x2) -> Self {
        Transform { 
            m11: value.M11.into(), 
            m12: value.M12.into(), 
            m21: value.M21.into(), 
            m22: value.M22.into(), 
            tx: value.M31.into(), 
            ty: value.M32.into() 
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct RendererGeneration(usize);

pub type RendererRef<'a> = &'a mut Renderer;

enum SavedAction {
    Clip,
    Layer
}

struct SavedState {
    transform: Matrix3x2,
    actions: Vec<SavedAction>
}

pub struct Renderer {
    render_target: Direct2D::ID2D1HwndRenderTarget,
    brush: Direct2D::ID2D1SolidColorBrush,
    saved_states: Vec<SavedState>,
    generation: RendererGeneration
}

impl Renderer {
    pub fn new(hwnd: HWND) -> Result<Self> {
        let mut rect: RECT = RECT::default();
        unsafe { GetClientRect(hwnd, &mut rect) };

        let size = Direct2D::Common::D2D_SIZE_U {
            width: (rect.right - rect.left) as u32,
            height: (rect.bottom - rect.top) as u32
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
                &hwnd_render_target_properies as *const _)?
        };

        let brush = unsafe {
            render_target.CreateSolidColorBrush(&Color::GREEN.into(), None)?
        };

        Ok(Renderer {
            render_target,
            brush,
            saved_states: Vec::new(),
            generation: RendererGeneration(0)
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

    fn get_d2d1_transform(&self) -> Matrix3x2 {
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
        unsafe { self.render_target.PushAxisAlignedClip(&bounds.into(), Direct2D::D2D1_ANTIALIAS_MODE_PER_PRIMITIVE) };
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
            unsafe { self.render_target.SetTransform(&saved_state.transform)}
            for action in saved_state.actions {
                match action {
                    SavedAction::Clip => unsafe { self.render_target.PopAxisAlignedClip() },
                    SavedAction::Layer => unsafe { self.render_target.PopLayer() },
                }
            }
        }
    }

    pub fn draw_line(&mut self, p0: Point, p1: Point, color: Color, line_width: f32) {
		unsafe {
            self.brush.SetColor(&color.into());
            self.render_target.DrawLine(p0.into(), p1.into(), &self.brush, line_width, None)
        }
	}

    pub fn draw_rectangle(&mut self, rect: Rectangle, color: Color, line_width: f32) {
        unsafe {
            self.brush.SetColor(&color.into());
            self.render_target.DrawRectangle(
                &rect.into(), 
                &self.brush,
                line_width,
                None);
        };
    }

    pub fn fill_rectangle(&mut self, rect: Rectangle, color: Color) {
        unsafe {
            self.brush.SetColor(&color.into());
            self.render_target.FillRectangle(
                &rect.into(), 
                &self.brush)
        };
    }

    pub fn draw_rounded_rectangle(&mut self, rect: Rectangle, radius: Size, color: Color, line_width: f32) {
        unsafe {
            self.brush.SetColor(&color.into());
            let rounded_rect = Direct2D::D2D1_ROUNDED_RECT {
                rect: rect.into(),
                radiusX: radius.width as f32,
                radiusY: radius.height as f32,
            };
            self.render_target.DrawRoundedRectangle(
                &rounded_rect, 
                &self.brush, 
                line_width, 
                None);
        }
    }

    pub fn fill_rounded_rectangle(&mut self, rect: Rectangle, radius: Size, color: Color) {
        unsafe {
            self.brush.SetColor(&color.into());
            let rounded_rect = Direct2D::D2D1_ROUNDED_RECT {
                rect: rect.into(),
                radiusX: radius.width as f32,
                radiusY: radius.height as f32,
            };
            self.render_target.FillRoundedRectangle(&rounded_rect, &self.brush)
        }
    }

    pub fn draw_ellipse(&mut self, origin: Point, radii: Size, color: Color, line_width: f32) {
        unsafe {
            self.brush.SetColor(&color.into());
            let ellipse = Direct2D::D2D1_ELLIPSE {
                point: origin.into(),
                radiusX: radii.width as f32,
                radiusY: radii.height as f32,
            };
            self.render_target.DrawEllipse(&ellipse, &self.brush, line_width, None);
        }
    }

    pub fn fill_ellipse(&mut self, origin: Point, radii: Size, color: Color) {
        unsafe {
            self.brush.SetColor(&color.into());
            let ellipse = Direct2D::D2D1_ELLIPSE {
                point: origin.into(),
                radiusX: radii.width as f32,
                radiusY: radii.height as f32,
            };
            self.render_target.FillEllipse(&ellipse, &self.brush)
        }
    }

    pub fn draw_text(&mut self, text_layout: &TextLayout, position: Point) {
        unsafe {
            self.brush.SetColor(&text_layout.color.into());
            self.render_target.DrawTextLayout(
                position.into(), 
                &text_layout.text_layout, 
                &self.brush, 
                Direct2D::D2D1_DRAW_TEXT_OPTIONS_NONE)
        }
    }

    pub fn draw_bitmap(&mut self, source: &ImageSource, rect: Rectangle) {
        source.draw(&self.render_target, self.generation, rect.into())
    }
}