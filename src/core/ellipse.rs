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

    pub fn from_rectangle(rect: Rect<f64>) -> Self {
        Self {
            center: rect.center(),
            radii: rect.size() / 2.0,
        }
    }

    pub fn offset(&self, delta: impl Into<Vec2>) -> Self {
        Self::new(self.center + delta.into(), self.radii)
    }

    pub fn contains(&self, pos: Point) -> bool {
        if self.radii.width < f64::EPSILON || self.radii.height < f64::EPSILON {
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
    pub radius: f64,
}

impl Circle {
    pub const fn new(center: Point, radius: f64) -> Self {
        Self { center, radius }
    }

    pub fn with_radius(mut self, radius: f64) -> Self {
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
