use std::ops::{Add, Div, Mul, Sub};

use crate::core::{ScaleFactor, Zero};

use super::Lerp;

#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    pub const INFINITY: Self = Self {
        width: f32::INFINITY,
        height: f32::INFINITY,
    };

    #[inline(always)]
    #[must_use]
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    /// Create a Size with both `width` and `height` set to `v`
    #[inline]
    #[must_use]
    pub const fn splat(v: f32) -> Self {
        Self {
            width: v,
            height: v,
        }
    }

    /// Returns a new Size with the `height` and `width` modified by the mapping function `f`
    #[inline]
    #[must_use]
    pub fn map(self, f: impl Fn(f32) -> f32) -> Size {
        Size::new(f(self.width), f(self.height))
    }

    /// Returns a new Size with the `width` modified by the mapping function `f`
    #[inline]
    #[must_use]
    pub fn map_width(self, f: impl Fn(f32) -> f32) -> Self {
        Self::new(f(self.width), self.height)
    }

    /// Returns a new Size with the `height` modified by the mapping function `f`
    #[inline]
    #[must_use]
    pub fn map_height(self, f: impl Fn(f32) -> f32) -> Self {
        Self::new(self.width, f(self.height))
    }

    #[inline]
    #[must_use]
    pub const fn with_width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    #[inline]
    #[must_use]
    pub const fn with_height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    pub const fn max(&self, other: &Self) -> Self {
        Self::new(self.width.max(other.width), self.height.max(other.height))
    }

    pub const fn min(&self, other: &Self) -> Self {
        Self::new(self.width.min(other.width), self.height.min(other.height))
    }

    pub const fn scale(mut self, factor: f32) -> Self {
        self.height *= factor;
        self.width *= factor;
        self
    }

    pub const fn scale_x(mut self, factor: f32) -> Self {
        self.width *= factor;
        self
    }

    pub const fn scale_y(mut self, factor: f32) -> Self {
        self.height *= factor;
        self
    }

    pub const fn clamp(&self, min: Self, max: Self) -> Self {
        let height = self.height.clamp(min.height, max.height);
        let width = self.width.clamp(min.width, max.width);
        Self { width, height }
    }

    pub const fn max_element(&self) -> f32 {
        self.width.max(self.height)
    }

    pub const fn min_element(&self) -> f32 {
        self.width.min(self.height)
    }

    /*pub fn expand_to_u32(&self) -> Size<u32> {
        assert!(self.width >= 0.0);
        assert!(self.height >= 0.0);
        Size {
            width: self.width.ceil() as u32,
            height: self.height.ceil() as u32,
        }
    }*/
}

impl Zero for Size {
    const ZERO: Self = Self {
        width: 0.0,
        height: 0.0,
    };
}

impl<U: Into<f32>> From<[U; 2]> for Size {
    fn from([width, height]: [U; 2]) -> Self {
        Self {
            width: width.into(),
            height: height.into(),
        }
    }
}

impl<U: Into<f32>> From<(U, U)> for Size {
    fn from((width, height): (U, U)) -> Self {
        Self {
            width: width.into(),
            height: height.into(),
        }
    }
}

impl Add for Size {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            width: self.width + rhs.width,
            height: self.height + rhs.height,
        }
    }
}

impl Sub for Size {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            width: self.width - rhs.width,
            height: self.height - rhs.height,
        }
    }
}

impl Mul<f32> for Size {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Size::new(self.width * rhs, self.height * rhs)
    }
}

impl Div<f32> for Size {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Size::new(self.width / rhs, self.height / rhs)
    }
}

impl Lerp for Size {
    fn lerp(&self, other: &Self, scalar: f64) -> Self {
        Self {
            width: self.width.lerp(&other.width, scalar),
            height: self.height.lerp(&other.height, scalar),
        }
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct PhysicalSize {
    pub width: i32,
    pub height: i32,
}

impl PhysicalSize {
    pub fn new(width: i32, height: i32) -> Self {
        Self { width, height }
    }

    pub fn from_logical(logical_size: Size, scale_factor: ScaleFactor) -> Self {
        let size = logical_size.scale(scale_factor.0).map(|x| x.round());
        Self {
            width: size.width as i32,
            height: size.height as i32,
        }
    }

    pub fn into_logical(self, scale_factor: ScaleFactor) -> Size {
        Size::new(
            self.width as f32 / scale_factor.0,
            self.height as f32 / scale_factor.0,
        )
    }
}

pub struct PartialSize {
    pub width: Option<f32>,
    pub height: Option<f32>,
}

impl PartialSize {
    pub fn unwrap_or(self, other: Size) -> Size {
        Size {
            width: self.width.unwrap_or(other.width),
            height: self.height.unwrap_or(other.height),
        }
    }
}
