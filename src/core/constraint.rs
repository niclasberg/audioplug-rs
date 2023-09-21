use super::Size;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Constraint {
    pub min_size: Size,
    pub max_size: Size
}

impl Constraint {
    pub const NONE: Self = Self {
        min_size: Size::ZERO,
        max_size: Size::INFINITY,
    };

    pub fn min(&self) -> Size {
        self.min_size
    }

    pub fn max(&self) -> Size {
        self.min_size
    }

    pub fn exact(size: Size) -> Self {
        Self { min_size: size, max_size: size }
    }

    pub fn shrink(&self, size: Size) -> Self {
        let min_size = Size::new(
            (self.min_size.width - size.width).max(0.0),
            (self.min_size.height - size.height).max(0.0)
        );

        let max_size = Size::new(
            (self.max_size.width - size.width).max(0.0),
            (self.max_size.height - size.height).max(0.0)
        );

        Self { min_size, max_size }
    }

    pub fn clamp(&self, size: Size) -> Size {
        size.clamp(&self.min(), &self.max())
    }
}