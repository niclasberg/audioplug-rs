use super::{Point, Rect, Size, Vec2};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ellipse {
    pub center: Point,
    pub radii: Size,
}

impl Ellipse {
    pub const fn new(center: Point, radii: Size) -> Self {
        Self { center, radii }
    }

    pub fn from_rectangle(rect: Rect) -> Self {
        Self {
            center: rect.center(),
            radii: rect.size() / 2.0,
        }
    }

    pub fn offset(&self, delta: impl Into<Vec2>) -> Self {
        Self::new(self.center + delta.into(), self.radii)
    }

    pub fn scale(&self, scale: f32) -> Self {
        Self::new(self.center.scale(scale), self.radii.scale(scale))
    }

    pub fn inflate(self, amount: f32) -> Self {
        Self {
            center: self.center,
            radii: self.radii + Size::splat(amount),
        }
    }

    pub fn contains(&self, pos: Point) -> bool {
        if self.radii.width < f32::EPSILON || self.radii.height < f32::EPSILON {
            false
        } else {
            ((pos.x - self.center.x) / self.radii.width).powi(2)
                + ((pos.y - self.center.y) / self.radii.height).powi(2)
                <= 1.0
        }
    }

    pub fn bounds(&self) -> Rect {
        Rect::from_center(self.center, self.radii.scale(2.0))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Circle {
    pub center: Point,
    pub radius: f32,
}

impl Circle {
    pub const fn new(center: Point, radius: f32) -> Self {
        Self { center, radius }
    }

    pub fn with_radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    pub fn contains(&self, pos: Point) -> bool {
        (pos.x - self.center.x).powi(2) + (pos.y - self.center.y).powi(2) <= self.radius.powi(2)
    }
}

impl From<Circle> for Ellipse {
    fn from(value: Circle) -> Self {
        Self::new(value.center, Size::new(value.radius, value.radius))
    }
}
