use std::ops::{Add, Mul};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Size<T = f64> {
    pub width: T,
    pub height: T,
}

impl<T> Size<T> {
    pub fn new(width: T, height: T) -> Self {
        Self { width, height }
    }
}

impl Size<f64> {
    pub const ZERO: Self = Self {
        width: 0.0,
        height: 0.0
    };

    pub const INFINITY: Self = Self {
        width: f64::INFINITY,
        height: f64::INFINITY
    };

    pub fn max(&self, other: &Self) -> Self {
        Self::new(self.width.max(other.width), self.height.max(other.height))
    }

    pub fn min(&self, other: &Self) -> Self {
        Self::new(self.width.min(other.width), self.height.min(other.height))
    }

    pub fn scale(mut self, factor: f64) -> Self {
        self.height *= factor;
        self.width *= factor;
        self
    }

    pub fn clamp(&self, min: &Self, max: &Self) -> Self {
        let height = self.height.clamp(min.height, max.height);
        let width = self.width.clamp(min.width, max.width);
        Self { width, height }
    }
}

impl From<[u32; 2]> for Size {
    fn from([width, height]: [u32; 2]) -> Self {
        Self { width: width.into(), height: height.into() }
    }
}

impl From<[u16; 2]> for Size {
    fn from([width, height]: [u16; 2]) -> Self {
        Self { width: width.into(), height: height.into() }
    }
}

impl Add for Size {
    type Output = Size;

    fn add(self, rhs: Self) -> Self::Output {
        Self { width: self.width + rhs.width, height: self.height + self.height }
    }
}