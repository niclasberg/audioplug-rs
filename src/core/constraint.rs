use crate::core::Zero;

use super::Size;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Constraint {
    pub min_size: Size,
    pub max_size: Size,
}

impl Constraint {
    pub const NONE: Self = Self {
        min_size: Size::ZERO,
        max_size: Size::INFINITY,
    };

    pub fn exact(size: Size) -> Self {
        Self {
            min_size: size,
            max_size: size,
        }
    }

    pub fn is_tight(&self) -> bool {
        self.min_size == self.max_size
    }

    pub fn has_unbounded_width(&self) -> bool {
        self.max_size.width.is_infinite()
    }

    pub fn has_unbounded_height(&self) -> bool {
        self.max_size.height.is_infinite()
    }

    pub fn min(&self) -> Size {
        self.min_size
    }

    pub fn with_min(mut self, size: Size) -> Self {
        self.min_size = size;
        self
    }

    pub fn max(&self) -> Size {
        self.max_size
    }

    pub fn with_max(mut self, size: Size) -> Self {
        self.max_size = size;
        self
    }

    pub fn shrink(&self, size: Size) -> Self {
        let min_size = Size::new(
            (self.min_size.width - size.width).max(0.0),
            (self.min_size.height - size.height).max(0.0),
        );

        let max_size = Size::new(
            (self.max_size.width - size.width).max(0.0),
            (self.max_size.height - size.height).max(0.0),
        );

        Self { min_size, max_size }
    }

    pub fn clamp(&self, size: Size) -> Size {
        size.clamp(self.min(), self.max())
    }
}
