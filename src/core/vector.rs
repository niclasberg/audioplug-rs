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
#[derive(Debug, Default, Copy, Clone, PartialEq, Pod, Zeroable)]
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

            pub const fn scale(self, val: $t) -> Self {
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

        impl Div<$t> for $name {
            type Output = Self;

            fn div(self, rhs: $t) -> Self::Output {
                Self {
                    x: self.x / rhs,
                    y: self.y / rhs,
                }
            }
        }
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

            pub const fn floor(self) -> Self {
                Self {
                    x: self.x.floor(),
                    y: self.y.floor(),
                }
            }

            pub const fn ceil(self) -> Self {
                Self {
                    x: self.x.ceil(),
                    y: self.y.ceil(),
                }
            }

            pub const fn round(self) -> Self {
                Self {
                    x: self.x.round(),
                    y: self.y.round(),
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

impl_vec2_base!(Vec2, f64);
impl_vec2_base!(Vec2f, f32);
impl_vec2_base!(Vec2i, i32);
impl_vec2_base!(Vec2u, u32);
impl_vec2_float!(Vec2, f64);
impl_vec2_float!(Vec2f, f32);

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq, Pod, Zeroable)]
pub struct Vec3f {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3f {
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    pub const X: Self = Self {
        x: 1.0,
        y: 0.0,
        z: 0.0,
    };
    pub const Y: Self = Self {
        x: 0.0,
        y: 1.0,
        z: 0.0,
    };
    pub const Z: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 1.0,
    };
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Pod, Zeroable)]
pub struct Vec4f {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vec4f {
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 0.0,
    };
    pub const X: Self = Self {
        x: 1.0,
        y: 0.0,
        z: 0.0,
        w: 0.0,
    };
    pub const Y: Self = Self {
        x: 0.0,
        y: 1.0,
        z: 0.0,
        w: 0.0,
    };
    pub const Z: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 1.0,
        w: 0.0,
    };
    pub const W: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 1.0,
    };
}
