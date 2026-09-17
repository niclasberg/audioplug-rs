use crate::core::Rect;

use super::{Point, Size};

impl From<taffy::Point<f32>> for Point {
    fn from(value: taffy::Point<f32>) -> Self {
        Point::new(value.x, value.y)
    }
}

impl From<Point> for taffy::Point<f32> {
    fn from(val: Point) -> Self {
        taffy::Point { x: val.x, y: val.y }
    }
}

impl From<taffy::Size<f32>> for Size {
    fn from(value: taffy::Size<f32>) -> Self {
        Size::new(value.width, value.height)
    }
}

impl<U: From<f32>> From<Size> for taffy::Size<U> {
    fn from(val: Size) -> Self {
        taffy::Size {
            width: val.width.into(),
            height: val.height.into(),
        }
    }
}

impl<T: From<f32>> From<Rect> for taffy::Rect<T> {
    fn from(value: Rect) -> Self {
        taffy::Rect {
            left: value.left.into(),
            right: value.right.into(),
            top: value.top.into(),
            bottom: value.bottom.into(),
        }
    }
}

impl From<taffy::Rect<f32>> for Rect {
    fn from(value: taffy::Rect<f32>) -> Self {
        Rect {
            left: value.left,
            top: value.top,
            right: value.right,
            bottom: value.bottom,
        }
    }
}
