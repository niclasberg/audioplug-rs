use std::ops::{Add, Sub};

use bytemuck::{Pod, Zeroable};

use super::{Interpolate, Point, Size};

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct Vec2f {
    pub x: f32,
    pub y: f32,
}

macro_rules! impl_vec2 {
    ($name: ident, $t: tt) => {
        impl $name {
            pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
            pub const X: Self = Self { x: 1.0, y: 0.0 };
            pub const Y: Self = Self { x: 0.0, y: 1.0 };
            pub const MIN: Self = Self {
                x: $t::MIN,
                y: $t::MIN,
            };
            pub const MAX: Self = Self {
                x: $t::MAX,
                y: $t::MAX,
            };

            pub const fn new(x: $t, y: $t) -> Self {
                Self { x, y }
            }

            pub const fn splat(val: $t) -> Self {
                Self { x: val, y: val }
            }

            pub const fn into_point(self) -> Point<$t> {
                Point {
                    x: self.x,
                    y: self.y,
                }
            }

            pub const fn into_size(self) -> Size<$t> {
                Size {
                    width: self.x,
                    height: self.y,
                }
            }

            pub fn dot(self, other: Self) -> $t {
                self.x * other.x + self.y * other.y
            }

            pub fn length(self) -> $t {
                self.x.hypot(self.y)
            }

            pub fn length_squared(self) -> $t {
                self.dot(self)
            }

            pub fn min(self, other: Self) -> Self {
                Self {
                    x: self.x.min(other.x),
                    y: self.y.min(other.y),
                }
            }

            pub fn max(self, other: Self) -> Self {
                Self {
                    x: self.x.max(other.x),
                    y: self.y.max(other.y),
                }
            }

            pub fn floor(self) -> Self {
                Self {
                    x: self.x.floor(),
                    y: self.y.floor(),
                }
            }

            pub fn ceil(self) -> Self {
                Self {
                    x: self.x.ceil(),
                    y: self.y.ceil(),
                }
            }
        }

        impl From<Point<$t>> for $name {
            fn from(value: Point<$t>) -> Self {
                Self {
                    x: value.x,
                    y: value.y,
                }
            }
        }

        impl From<Size<$t>> for $name {
            fn from(value: Size<$t>) -> Self {
                Self {
                    x: value.width,
                    y: value.height,
                }
            }
        }

        impl Add for $name {
            type Output = Self;

            fn add(self, rhs: Self) -> Self::Output {
                Self {
                    x: self.x + rhs.x,
                    y: self.y + rhs.y,
                }
            }
        }

        impl Sub for $name {
            type Output = Self;

            fn sub(self, rhs: Self) -> Self::Output {
                Self {
                    x: self.x - rhs.x,
                    y: self.y - rhs.y,
                }
            }
        }

        impl Interpolate for $name {
            fn lerp(&self, other: &Self, scalar: f64) -> Self {
                Self {
                    x: self.x.lerp(&other.x, scalar),
                    y: self.y.lerp(&other.y, scalar),
                }
            }
        }

        unsafe impl Zeroable for $name {}
        unsafe impl Pod for $name {}
    };
}

impl_vec2!(Vec2, f64);
impl_vec2!(Vec2f, f32);

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vec3f {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

unsafe impl Zeroable for Vec3f {}
unsafe impl Pod for Vec3f {}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vec4f {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

unsafe impl Zeroable for Vec4f {}
unsafe impl Pod for Vec4f {}
