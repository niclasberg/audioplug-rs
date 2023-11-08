mod color;
mod context;
mod path;
mod affine_transform;
pub use color::CGColor;
pub use context::CGContext;
pub use icrate::Foundation::{CGPoint, CGRect, CGSize, CGFloat};
pub use path::CGPath;
pub use affine_transform::CGAffineTransform;

use crate::core::{Size, Point, Rectangle};

impl Into<CGPoint> for Point {
    fn into(self) -> CGPoint {
        CGPoint { x: self.x, y: self.y }
    }
}

impl From<CGPoint> for Point {
    fn from(value: CGPoint) -> Self {
        Point::new(value.x, value.y)
    }
}

impl Into<CGSize> for Size {
    fn into(self) -> CGSize {
        CGSize { width: self.width, height: self.height }
    }
}

impl From<CGSize> for Size {
    fn from(value: CGSize) -> Self {
        Size::new(value.width, value.height)
    }
}

impl Into<CGRect> for Rectangle {
    fn into(self) -> CGRect {
        CGRect { origin: self.position().into(), size: self.size().into() }
    }
}

impl From<CGRect> for Rectangle {
    fn from(value: CGRect) -> Self {
        Rectangle::new(value.origin.into(), value.size.into())
    }
}