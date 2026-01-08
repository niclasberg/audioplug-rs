use crate::core::Zero;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Edge {
    Left,
    Top,
    Right,
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Edges<T> {
    pub left: T,
    pub top: T,
    pub right: T,
    pub bottom: T,
}

impl<T> Edges<T>
where
    T: Copy,
{
    pub fn all(value: T) -> Self {
        Self {
            left: value,
            top: value,
            right: value,
            bottom: value,
        }
    }
}

impl<T: Zero> Zero for Edges<T> {
    const ZERO: Self = Self {
        left: T::ZERO,
        top: T::ZERO,
        right: T::ZERO,
        bottom: T::ZERO,
    };
}
