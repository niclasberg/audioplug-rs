use windows::{core::{PCWSTR, w, Result, Error}, 
    Win32::{Graphics::Direct2D, Foundation::{HWND, RECT}, UI::WindowsAndMessaging::GetClientRect}};

use crate::core::{Rectangle, Color};

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

pub struct Renderer {
    render_target: Direct2D::ID2D1HwndRenderTarget,
    brush: Direct2D::ID2D1SolidColorBrush,
}

impl Renderer {
    pub fn new(hwnd: HWND, d2d_factory: &Direct2D::ID2D1Factory) -> Result<Self> {
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
            d2d_factory.CreateHwndRenderTarget(
                &render_target_properties as *const _, 
                &hwnd_render_target_properies as *const _)?
        };

        let brush = unsafe {
            render_target.CreateSolidColorBrush(&Color::GREEN.into(), None)?
        };

        Ok(Renderer {
            render_target,
            brush
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

    pub fn fill_rounded_rectangle(&mut self, rect: Rectangle, radius_x: f32, radius_y: f32, color: Color) {
        unsafe {
            self.brush.SetColor(&color.into());
            let rounded_rect = Direct2D::D2D1_ROUNDED_RECT {
                rect: rect.into(),
                radiusX: radius_x,
                radiusY: radius_y,
            };
            self.render_target.FillRoundedRectangle(&rounded_rect, &self.brush)
        }
    }
}