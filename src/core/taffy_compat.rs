use super::{Point, Size};

impl<T> From<taffy::Point<T>> for Point<T> {
    fn from(value: taffy::Point<T>) -> Self {
        Point::new(value.x, value.y)
    }
}

impl<T> From<Point<T>> for taffy::Point<T> {
    fn from(val: Point<T>) -> Self {
        taffy::Point { x: val.x, y: val.y }
    }
}

impl<T> From<taffy::Size<T>> for Size<T> {
    fn from(value: taffy::Size<T>) -> Self {
        Size::new(value.width, value.height)
    }
}

impl<U, T: Into<U>> From<Size<T>> for taffy::Size<U> {
    fn from(val: Size<T>) -> Self {
        taffy::Size {
            width: val.width.into(),
            height: val.height.into(),
        }
    }
}
