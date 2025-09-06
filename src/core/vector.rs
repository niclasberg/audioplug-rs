use std::ops::{Add, Div, Mul, Neg, Sub};

use bytemuck::{Pod, Zeroable};

use super::{Interpolate, Point, Size};

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Vec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    pub const X: Self = Self { x: 1.0, y: 0.0 };
    pub const Y: Self = Self { x: 0.0, y: 1.0 };
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct Vec2f {
    pub x: f32,
    pub y: f32,
}

impl Vec2f {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    pub const X: Self = Self { x: 1.0, y: 0.0 };
    pub const Y: Self = Self { x: 0.0, y: 1.0 };
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct Vec2i {
    pub x: i32,
    pub y: i32,
}

impl Vec2i {
    pub const ZERO: Self = Self { x: 0, y: 0 };
    pub const X: Self = Self { x: 1, y: 0 };
    pub const Y: Self = Self { x: 0, y: 1 };
}

impl Neg for Vec2i {
    type Output = Self;

    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct Vec2u {
    pub x: u32,
    pub y: u32,
}

impl Vec2u {
    pub const ZERO: Self = Self { x: 0, y: 0 };
    pub const X: Self = Self { x: 1, y: 0 };
    pub const Y: Self = Self { x: 0, y: 1 };
}

macro_rules! impl_vec2_base {
    ($name: ident, $t: tt) => {
        impl $name {
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

            pub fn scale(self, val: $t) -> Self {
                Self {
                    x: self.x * val,
                    y: self.y * val,
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

        impl Mul for $name {
            type Output = Self;

            fn mul(self, rhs: Self) -> Self::Output {
                Self {
                    x: rhs.x * self.x,
                    y: rhs.y * self.y,
                }
            }
        }

        impl Mul<$t> for $name {
            type Output = Self;

            fn mul(self, rhs: $t) -> Self::Output {
                Self {
                    x: rhs * self.x,
                    y: rhs * self.y,
                }
            }
        }

        impl Mul<$name> for $t {
            type Output = $name;

            fn mul(self, rhs: $name) -> Self::Output {
                Self::Output {
                    x: self * rhs.x,
                    y: self * rhs.y,
                }
            }
        }

        impl Div for $name {
            type Output = Self;

            fn div(self, rhs: Self) -> Self::Output {
                Self {
                    x: self.x / rhs.x,
                    y: self.y / rhs.y,
                }
            }
        }

        unsafe impl Zeroable for $name {}
        unsafe impl Pod for $name {}
    };
}

macro_rules! impl_vec2_float {
    ($name: ident, $t: tt) => {
        impl $name {
            pub const MIN: Self = Self {
                x: $t::MIN,
                y: $t::MIN,
            };
            pub const MAX: Self = Self {
                x: $t::MAX,
                y: $t::MAX,
            };

            pub fn dot(self, other: Self) -> $t {
                self.x * other.x + self.y * other.y
            }

            pub fn length(self) -> $t {
                self.x.hypot(self.y)
            }

            pub fn length_squared(self) -> $t {
                self.dot(self)
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

        impl Neg for $name {
            type Output = Self;

            fn neg(self) -> Self {
                Self {
                    x: -self.x,
                    y: -self.y,
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
    };
}

impl_vec2_base!(Vec2i, i32);
impl_vec2_base!(Vec2u, u32);
impl_vec2_base!(Vec2, f64);
impl_vec2_float!(Vec2, f64);
impl_vec2_base!(Vec2f, f32);
impl_vec2_float!(Vec2f, f32);

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq)]
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
