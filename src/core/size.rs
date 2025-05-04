use std::ops::{Add, Div, Mul, Sub};

use num::Zero;

use super::{Interpolate, SpringPhysics};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Size<T = f64> {
    pub width: T,
    pub height: T,
}

impl<T> Size<T> {
    #[inline(always)]
    #[must_use]
    pub const fn new(width: T, height: T) -> Self {
        Self { width, height }
    }

    /// Create a Size with both `width` and `height` set to `v`
    #[inline]
    #[must_use]
    pub fn splat(v: T) -> Self
    where
        T: Clone,
    {
        Self {
            width: v.clone(),
            height: v,
        }
    }

    /// Returns a new Size with the `height` and `width` modified by the mapping function `f`
    #[inline]
    #[must_use]
    pub fn map<U>(self, f: impl Fn(T) -> U) -> Size<U> {
        Size::new(f(self.width), f(self.height))
    }

    /// Returns a new Size with the `width` modified by the mapping function `f`
    #[inline]
    #[must_use]
    pub fn map_width(self, f: impl Fn(T) -> T) -> Self {
        Self::new(f(self.width), self.height)
    }

    /// Returns a new Size with the `height` modified by the mapping function `f`
    #[inline]
    #[must_use]
    pub fn map_height(self, f: impl Fn(T) -> T) -> Self {
        Self::new(self.width, f(self.height))
    }

    #[inline]
    #[must_use]
    pub fn with_width(mut self, width: T) -> Self {
        self.width = width;
        self
    }

    #[inline]
    #[must_use]
    pub fn with_height(mut self, height: T) -> Self {
        self.height = height;
        self
    }

    pub fn max(&self, other: &Self) -> Self
    where
        T: Ord + Copy,
    {
        Self::new(self.width.max(other.width), self.height.max(other.height))
    }

    pub fn min(&self, other: &Self) -> Self
    where
        T: Ord + Copy,
    {
        Self::new(self.width.min(other.width), self.height.min(other.height))
    }
}

impl Size<f64> {
    pub const ZERO: Self = Self {
        width: 0.0,
        height: 0.0,
    };

    pub const INFINITY: Self = Self {
        width: f64::INFINITY,
        height: f64::INFINITY,
    };

    pub fn scale(mut self, factor: f64) -> Self {
        self.height *= factor;
        self.width *= factor;
        self
    }

    pub fn scale_x(mut self, factor: f64) -> Self {
        self.width *= factor;
        self
    }

    pub fn scale_y(mut self, factor: f64) -> Self {
        self.height *= factor;
        self
    }

    pub fn clamp(&self, min: Self, max: Self) -> Self {
        let height = self.height.clamp(min.height, max.height);
        let width = self.width.clamp(min.width, max.width);
        Self { width, height }
    }

    pub fn max_element(&self) -> f64 {
        self.width.max(self.height)
    }

    pub fn min_element(&self) -> f64 {
        self.width.min(self.height)
    }
}

impl<T> Size<Option<T>> {
    pub fn unwrap_or(self, other: Size<T>) -> Size<T> {
        Size {
            width: self.width.unwrap_or(other.width),
            height: self.height.unwrap_or(other.height),
        }
    }
}

impl From<Size<i32>> for Size {
    fn from(value: Size<i32>) -> Self {
        Self {
            width: value.width as f64,
            height: value.height as f64,
        }
    }
}

impl<T, U: Into<T>> From<[U; 2]> for Size<T> {
    fn from([width, height]: [U; 2]) -> Self {
        Self {
            width: width.into(),
            height: height.into(),
        }
    }
}

impl<T, U: Into<T>> From<(U, U)> for Size<T> {
    fn from((width, height): (U, U)) -> Self {
        Self {
            width: width.into(),
            height: height.into(),
        }
    }
}

impl<T: Add<Output = T>> Add for Size<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            width: self.width + rhs.width,
            height: self.height + rhs.height,
        }
    }
}

impl<T: Sub<Output = T>> Sub for Size<T> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            width: self.width - rhs.width,
            height: self.height - rhs.height,
        }
    }
}

impl Mul<f64> for Size {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Size::new(self.width * rhs, self.height * rhs)
    }
}

impl Div<f64> for Size {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        if rhs.is_zero() {
            panic!("Cannot divide size by zero");
        }

        Size::new(self.width / rhs, self.height / rhs)
    }
}

impl<T: Default> Default for Size<T> {
    fn default() -> Self {
        Self {
            width: Default::default(),
            height: Default::default(),
        }
    }
}

impl<T: Interpolate> Interpolate for Size<T> {
    fn lerp(&self, other: &Self, scalar: f64) -> Self {
        Self {
            width: self.width.lerp(&other.width, scalar),
            height: self.height.lerp(&other.height, scalar),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn size_add() {
        assert_eq!(
            Size::new(5.0, 10.0) + Size::new(1.0, 2.0),
            Size::new(6.0, 12.0)
        );
    }
}
