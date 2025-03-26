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

    pub fn bounds(&self) -> Rectangle {
        Rectangle::from_center(self.center, self.radii.scale(2.0))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Circle<T = f64> {
    pub center: Point<T>, 
    pub radius: T,
}

impl<T> Circle<T> {
    pub const fn new(center: Point<T>, radius: T) -> Self {
        Self { center, radius }
    }

    pub fn with_radius(mut self, radius: T) -> Self {
        self.radius = radius;
        self
    }
}

impl Circle<f64> {
    pub fn contains(&self, pos: Point) -> bool {
        (pos.x - self.center.x).powi(2) + (pos.y - self.center.y).powi(2) <= self.radius.powi(2)
    }
}

impl<T: Clone> From<Circle<T>> for Ellipse<T> {
    fn from(value: Circle<T>) -> Self {
        Self::new(value.center, Size::new(value.radius.clone(), value.radius))
    }
}