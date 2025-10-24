use super::{Point, Rect};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct UnitValue(f64);

impl UnitValue {
    pub const MIN: Self = Self(0.0);
    pub const MAX: Self = Self(1.0);
}

impl UnitValue {
    pub const fn new(value: f64) -> Self {
        assert!(value >= 0.0 && value <= 1.0);
        Self(value)
    }

    pub const fn new_unchecked(value: f64) -> Self {
        Self(value)
    }

    pub fn resolve(&self, min: f64, max: f64) -> f64 {
        min + (max - min) * self.0
    }
}

impl From<UnitValue> for f64 {
    fn from(value: UnitValue) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct UnitPoint {
    pub x: f64,
    pub y: f64,
}

impl UnitPoint {
    pub const TOP_LEFT: Self = Self { x: 0.0, y: 0.0 };
    pub const TOP_CENTER: Self = Self { x: 0.5, y: 0.0 };
    pub const TOP_RIGHT: Self = Self { x: 1.0, y: 0.0 };
    pub const CENTER_LEFT: Self = Self { x: 0.0, y: 0.5 };
    pub const CENTER: Self = Self { x: 0.0, y: 0.5 };
    pub const CENTER_RIGHT: Self = Self { x: 1.0, y: 0.5 };
    pub const BOTTOM_LEFT: Self = Self { x: 0.0, y: 1.0 };
    pub const BOTTOM_CENTER: Self = Self { x: 0.5, y: 1.0 };
    pub const BOTTOM_RIGHT: Self = Self { x: 1.0, y: 1.0 };

    pub fn new(x: UnitValue, y: UnitValue) -> Self {
        Self { x: x.0, y: y.0 }
    }

    pub fn resolve(&self, bounds: Rect) -> Point {
        bounds.get_relative_point(self.x, self.y)
    }
}
