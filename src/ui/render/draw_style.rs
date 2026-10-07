use std::ops::Mul;

use crate::{
    core::{Color, ColorMap, LinearGradient, Point, Rect, Size, TranslateScale, Vec2},
    ui::style::Fill,
};

#[derive(Debug, Default, Clone)]
pub struct DrawStyle {
    pub fill: Option<Paint>,
    pub stroke: Option<StrokeStyle>,
    pub shadow: Option<ShadowOptions>,
}

impl DrawStyle {
    pub const fn new() -> Self {
        Self {
            fill: None,
            stroke: None,
            shadow: None,
        }
    }

    pub fn fill(mut self, fill: impl Into<Paint>) -> Self {
        self.fill = Some(fill.into());
        self
    }
}

#[derive(Debug, Clone)]
pub struct StrokeStyle {
    pub paint: Paint,
    pub line_width: f32,
}

#[derive(Debug, Clone)]
pub enum Paint {
    Solid(Color),
    LinearGradient {
        start: Point,
        end: Point,
        colors: ColorMap,
    },
    RadialGradient {
        center: Point,
        radius: f32,
        color_map: ColorMap,
    },
}

impl From<Color> for Paint {
    fn from(value: Color) -> Self {
        Self::Solid(value)
    }
}

impl From<LinearGradient> for Paint {
    fn from(value: LinearGradient) -> Self {
        Self::LinearGradient {
            start: value.start,
            end: value.end,
            colors: value.color_map,
        }
    }
}

impl<'a> Mul<PaintRef<'a>> for TranslateScale {
    type Output = PaintRef<'a>;

    fn mul(self, rhs: PaintRef<'a>) -> Self::Output {
        match rhs {
            PaintRef::Solid(color) => PaintRef::Solid(color),
            PaintRef::LinearGradient { start, end, colors } => PaintRef::LinearGradient {
                start: self * start,
                end: self * end,
                colors,
            },
            PaintRef::RadialGradient {
                center,
                radius,
                colors,
            } => PaintRef::RadialGradient {
                center: self * center,
                radius: self.scale * radius,
                colors,
            },
        }
    }
}

#[derive(Clone, Copy)]
pub enum PaintRef<'a> {
    Solid(Color),
    LinearGradient {
        start: Point,
        end: Point,
        colors: &'a ColorMap,
    },
    RadialGradient {
        center: Point,
        radius: f32,
        colors: &'a ColorMap,
    },
}

impl<'a> PaintRef<'a> {
    pub fn from_fill(fill: &'a Fill, window_size: Size, bounds: Rect) -> Self {
        match fill {
            Fill::Solid(color) => PaintRef::Solid(*color),
            Fill::LinearGradient { colors, start, end } => PaintRef::LinearGradient {
                start: start.resolve_into_point(window_size, bounds),
                end: end.resolve_into_point(window_size, bounds),
                colors,
            },
            Fill::RadialGradient {
                colors,
                center,
                radius,
            } => Self::RadialGradient {
                center: center.resolve_into_point(window_size, bounds),
                radius: radius.resolve_into_px(window_size, bounds.size().max_element() / 2.0),
                colors,
            },
        }
    }
}

impl<'a> From<&'a Paint> for PaintRef<'a> {
    fn from(value: &'a Paint) -> Self {
        match value {
            Paint::Solid(color) => Self::Solid(*color),
            Paint::LinearGradient { start, end, colors } => Self::LinearGradient {
                start: *start,
                end: *end,
                colors,
            },
            Paint::RadialGradient {
                center,
                radius,
                color_map,
            } => Self::RadialGradient {
                center: *center,
                radius: *radius,
                colors: color_map,
            },
        }
    }
}

impl From<Color> for PaintRef<'_> {
    fn from(value: Color) -> Self {
        Self::Solid(value)
    }
}

impl<'a> From<&'a LinearGradient> for PaintRef<'a> {
    fn from(value: &'a LinearGradient) -> Self {
        Self::LinearGradient {
            start: value.start,
            end: value.end,
            colors: &value.color_map,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShadowKind {
    DropShadow,
    InnerShadow,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShadowOptions {
    pub radius: f32,
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
