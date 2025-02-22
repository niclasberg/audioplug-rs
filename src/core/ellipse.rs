use super::{Point, Rectangle, Size, Vector};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ellipse<T = f64> {
    pub center: Point<T>, 
    pub radii: Size<T>
}

impl<T> Ellipse<T> {
    pub const fn new(center: Point<T>, radii: Size<T>) -> Self {
        Self { center, radii }
    }
}

impl Ellipse<f64> {
    pub fn from_rectangle(rect: Rectangle<f64>) -> Self {
        Self {
            center: rect.center(),
            radii: rect.size() / 2.0
        }   
    }

    pub fn offset(&self, delta: impl Into<Vector>) -> Self {
        Self::new(self.center + delta.into(), self.radii)
    }

    pub fn contains(&self, pos: Point) -> bool {
        if self.radii.width < f64::EPSILON || self.radii.height < f64::EPSILON {
            false
        } else {
            ((pos.x - self.center.x) / self.radii.width).powi(2) + ((pos.y - self.center.y) / self.radii.height).powi(2) <= 1.0
        }
    }
}