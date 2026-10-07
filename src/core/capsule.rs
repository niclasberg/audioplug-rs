use crate::core::{Axis, Point, Rect, Vec2};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Capsule {
    pub start: Point,
    pub end: Point,
    pub radius: f32,
}

impl Capsule {
    /// Creates a capsule that fits tightly within a rect
    pub const fn from_rect(rect: Rect, axis: Axis) -> Self {
        let radius = rect.size().min_element() / 2.0;
        let start = match axis {
            Axis::Vertical => Point::new(rect.center().x, rect.top + radius),
            Axis::Horizontal => Point::new(rect.left + radius, rect.center().y),
        };
        let end = match axis {
            Axis::Vertical => Point::new(rect.center().x, rect.bottom - radius),
            Axis::Horizontal => Point::new(rect.right - radius, rect.center().y),
        };

        Self { start, end, radius }
    }

    pub fn bounds(&self) -> Rect {
        // We can imagine the capsule as the union of two circles and a rect:
        // o=====o
        // We can thus find the tight bounding box as the rect containing the
        // two circles.
        let min = self.start.min(self.end);
        let max = self.start.max(self.end);
        Rect {
            left: min.x - self.radius,
            top: min.y - self.radius,
            right: max.x + self.radius,
            bottom: max.y + self.radius,
        }
    }

    pub fn offset(&self, delta: impl Into<Vec2>) -> Self {
        let delta = delta.into();
        Self {
            start: self.start + delta,
            end: self.end + delta,
            radius: self.radius,
        }
    }

    pub fn inflate(&self, amount: f32) -> Self {
        Self {
            radius: self.radius + amount,
            ..*self
        }
    }

    pub fn scale(&self, s: f32) -> Self {
        Self {
            start: self.start.scale(s),
            end: self.end.scale(s),
            radius: self.radius * s,
        }
    }

    pub fn contains(&self, point: Point) -> bool {
        self.bounds().contains(point)
    }
}
