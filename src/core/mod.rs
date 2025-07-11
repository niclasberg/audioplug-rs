mod alignment;
mod axis;
mod border;
mod color;
mod color_map;
mod constraint;
mod cursor;
mod ellipse;
mod interpolation;
mod keyboard;
mod point;
mod rectangle;
mod rounded_rectangle;
mod size;
mod taffy_compat;
mod text;
mod transform;
mod unit_point;
mod vector;

pub use alignment::{Align, HAlign, VAlign};
pub use axis::Axis;
pub use border::Border;
pub use color::Color;
pub use color_map::*;
pub use constraint::*;
pub use cursor::Cursor;
pub use ellipse::{Circle, Ellipse};
pub use interpolation::{Interpolate, SpringPhysics, SpringProperties};
pub use keyboard::{Key, Modifiers};
pub use point::Point;
pub use rectangle::Rectangle;
pub use rounded_rectangle::RoundedRectangle;
pub use size::Size;
pub use text::*;
pub use transform::Transform;
pub use unit_point::UnitPoint;
pub use vector::{Vec2, Vec2f, Vec3f, Vec4f};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WindowTheme {
    /// Light mode
    Light,
    /// Dark mode
    Dark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShadowKind {
    DropShadow,
    InnerShadow,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShadowOptions {
    pub radius: f64,
    pub offset: Vec2,
    pub color: Color,
    pub kind: ShadowKind,
}

impl ShadowOptions {
    pub const DEFAULT: Self = Self {
        radius: 0.0,
        offset: Vec2::ZERO,
        color: Color::BLACK.with_alpha(0.3),
        kind: ShadowKind::DropShadow,
    };
}

impl Default for ShadowOptions {
    fn default() -> Self {
        Self {
            radius: 0.0,
            offset: Vec2::ZERO,
            color: Color::BLACK.with_alpha(0.3),
            kind: ShadowKind::DropShadow,
        }
    }
}
