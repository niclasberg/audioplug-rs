use super::{Point, Rectangle, Size};

impl<T> From<taffy::Point<T>> for Point<T> {
    fn from(value: taffy::Point<T>) -> Self {
        Point::new(value.x, value.y)
    }
}

impl<T> Into<taffy::Point<T>> for Point<T> {
    fn into(self) -> taffy::Point<T> {
        taffy::Point {
            x: self.x,
            y: self.y,
        }
    }
}

impl<T> From<taffy::Size<T>> for Size<T> {
    fn from(value: taffy::Size<T>) -> Self {
        Size::new(value.width, value.height)
    }
}

impl<U, T: Into<U>> Into<taffy::Size<U>> for Size<T> {
    fn into(self) -> taffy::Size<U> {
        taffy::Size {
            width: self.width.into(),
            height: self.height.into(),
        }
    }
}
